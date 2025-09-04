use std::{
    fs::{File, read_dir},
    io::{BufWriter, Error, Write},
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use image::{Rgba, RgbaImage};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng, rngs::SmallRng};
use serde::Serialize;

static FONT_CACHE: OnceLock<FontCache> = OnceLock::new();
const DIGITS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

struct FontCache {
    fonts: Vec<FontArc>,
}

impl FontCache {
    fn new() -> Self {
        let fonts = Self::load_fonts();
        // детерминированный порядок
        // fonts.sort_by(|a, b| a.as_font().full_name().cmp(&b.as_font().full_name()));
        if fonts.is_empty() {
            panic!("No fonts found in assets/fonts");
        }
        FontCache { fonts }
    }

    fn load_fonts() -> Vec<FontArc> {
        let font_dir = Path::new("assets/fonts");

        read_dir(font_dir)
            .ok()
            .into_iter()
            .flat_map(|rd| rd.filter_map(|e| e.ok()))
            .map(|e| e.path())
            .filter(|p| {
                matches!(
                    p.extension().and_then(|s| s.to_str()),
                    Some("ttf") | Some("otf")
                )
            })
            .filter_map(|path| {
                std::fs::read(&path)
                    .ok()
                    .and_then(|bytes| FontArc::try_from_vec(bytes).ok())
                    .filter(|f| ('0'..='9').all(|ch| f.glyph_id(ch).0 != 0))
            })
            .collect()
    }

    fn get_random<'a>(&'a self, rng: &mut SmallRng) -> &'a FontArc {
        &self.fonts[rng.random_range(0..self.fonts.len())]
    }

    fn global() -> &'static FontCache {
        FONT_CACHE.get_or_init(FontCache::new)
    }
}

#[derive(Serialize, Clone, Copy, Debug)]
pub struct Highlight {
    row: usize,
    col: usize,
    sbox: (usize, usize),
    cell: (usize, usize),
}

#[derive(Clone, Copy)]
struct CellBox {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl CellBox {
    fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    fn as_array(&self) -> [u32; 4] {
        [self.x, self.y, self.w, self.h]
    }
}

pub struct ImageConfig<'a> {
    width: u32,
    height: u32,
    margin: u32,
    board_size: u32,
    cell_size: u32,
    with_highlight: bool,
    line_thick: u32,
    line_thin: u32,
    do_cell_grid: bool,
    out_dir: &'a str,
    font_px: f32,
}

impl ImageConfig<'_> {
    pub fn new(n: u32) -> Self {
        let width = 512;
        let height = 512;
        let margin = 16;
        let board_size = width - 2 * margin;
        let cell_size = board_size / n;
        let with_highlight = true;
        let line_thick = 5;
        let line_thin = 2;
        let do_cell_grid = true;
        let out_dir = "../dataset9";
        let font_px = 48.0;

        Self {
            width,
            height,
            margin,
            board_size,
            cell_size,
            with_highlight,
            line_thick,
            line_thin,
            do_cell_grid,
            out_dir,
            font_px,
        }
    }
}

struct ColorPalette {
    background: Rgba<u8>,
    border: Rgba<u8>,
    highlight_row: Rgba<u8>,
    highlight_col: Rgba<u8>,
    highlight_box: Rgba<u8>,
    highlight_cell: Rgba<u8>,
}

impl ColorPalette {
    fn new(rng: &mut SmallRng) -> Self {
        Self {
            background: Rgba([255, 255, 255, 255]),
            border: Rgba([20, 30, 40, 255]),
            highlight_row: Self::jittered_color([120, 170, 255, 70], [12, 12, 12, 28], rng),
            highlight_col: Self::jittered_color([120, 170, 255, 70], [12, 12, 12, 28], rng),
            highlight_box: Self::jittered_color([120, 170, 255, 50], [10, 10, 10, 22], rng),
            highlight_cell: Self::jittered_color([120, 170, 255, 120], [10, 10, 10, 30], rng),
        }
    }

    fn jittered_color(base: [u8; 4], ranges: [i16; 4], rng: &mut SmallRng) -> Rgba<u8> {
        Rgba([
            Self::jitter(base[0], ranges[0], rng),
            Self::jitter(base[1], ranges[1], rng),
            Self::jitter(base[2], ranges[2], rng),
            Self::jitter(base[3], ranges[3], rng),
        ])
    }

    fn jitter(base: u8, range: i16, rng: &mut SmallRng) -> u8 {
        ((base as i16 + rng.random_range(-range..=range)).clamp(0, 255)) as u8
    }
}

// ==== Константы вместо RenderCfg ====

#[derive(Serialize)]
struct JsonRecord {
    image: String,
    labels: Vec<u8>,
    boxes: Vec<[u32; 4]>,
    dim: u8,
    seed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    highlight: Option<Highlight>,
}

pub struct DatasetItemGenerator<'a, const N: usize, const BR: usize, const BC: usize> {
    pub m: [[u8; N]; N],
    pub(crate) writer: Option<BufWriter<File>>,
    pub config: ImageConfig<'a>,
}

impl<const N: usize, const BR: usize, const BC: usize> DatasetItemGenerator<'_, N, BR, BC> {
    pub fn reset_matrix(&mut self) {
        self.m.iter_mut().for_each(|row| row.fill(0));
    }

    pub fn generate_with_seed(&mut self, seed: u64) {
        let mut rng = SmallRng::seed_from_u64(seed);
        let p0 = rng.random_range(60..=75);
        for r in 0..N {
            for c in 0..N {
                self.m[r][c] = if rng.random_range(0..100) < p0 {
                    0
                } else {
                    rng.random_range(1..=N) as u8
                };
            }
        }
    }
    pub fn save_img(
        &self,
        id: u32,
        seed: u64,
    ) -> image::ImageResult<([[[u32; 4]; N]; N], Option<Highlight>)> {
        debug_assert_eq!(N, BR * BC);

        let mut rng = SmallRng::seed_from_u64(seed);
        let colors = ColorPalette::new(&mut rng);

        // Инициализация изображения
        let mut img =
            RgbaImage::from_pixel(self.config.width, self.config.height, colors.background);

        // Вычисление bbox'ов клеток
        let boxes = self.calculate_cell_boxes();

        // Генерация подсветок
        let highlight = self.generate_highlight(&mut rng);

        // Рендеринг
        if let Some(hl) = highlight {
            self.render_highlights(&mut img, &hl, &boxes, &colors, &mut rng);
        }

        self.render_borders(&mut img, &colors);
        self.render_numbers(&mut img, &boxes, &colors, &mut rng);

        // Сохранение
        self.save_image(&img, id)?;

        // Преобразование boxes в нужный формат
        let result_boxes = self.convert_boxes(&boxes);
        Ok((result_boxes, highlight))
    }

    fn calculate_cell_boxes(&self) -> [[CellBox; N]; N] {
        let mut boxes = [[CellBox::new(0, 0, 0, 0); N]; N];
        let w = self.config.board_size;
        let h = self.config.board_size;
        let cs = self.config.cell_size;
        for r in 0..N {
            for c in 0..N {
                let x = self.config.margin + c as u32 * cs;
                let y = self.config.margin + r as u32 * cs;
                let ww = if c == N - 1 {
                    w - cs * (N as u32 - 1)
                } else {
                    cs
                };
                let hh = if r == N - 1 {
                    h - cs * (N as u32 - 1)
                } else {
                    cs
                };
                boxes[r][c] = CellBox::new(x, y, ww, hh);
            }
        }
        boxes
    }

    fn generate_highlight(&self, rng: &mut SmallRng) -> Option<Highlight> {
        if !self.config.with_highlight {
            return None;
        }

        Some(Highlight {
            row: rng.random_range(0..N),
            col: rng.random_range(0..N),
            sbox: (rng.random_range(0..N / BR), rng.random_range(0..N / BC)),
            cell: (rng.random_range(0..N), rng.random_range(0..N)),
        })
    }

    fn render_highlights(
        &self,
        img: &mut RgbaImage,
        highlight: &Highlight,
        boxes: &[[CellBox; N]; N],
        colors: &ColorPalette,
        rng: &mut SmallRng,
    ) {
        // Подсветка строки
        if rng.random_range(0..100) < 70 {
            let y = boxes[highlight.row][0].y;
            draw_filled_rect_mut(
                img,
                Rect::at(self.config.margin as i32, y as i32)
                    .of_size(self.config.board_size, self.config.cell_size),
                colors.highlight_row,
            );
        }

        // Подсветка колонки
        if rng.random_range(0..100) < 70 {
            let x = boxes[0][highlight.col].x;
            draw_filled_rect_mut(
                img,
                Rect::at(x as i32, self.config.margin as i32)
                    .of_size(self.config.cell_size, self.config.board_size),
                colors.highlight_col,
            );
        }

        // Подсветка блока
        if rng.random_range(0..100) < 70 {
            let (by, bx) = highlight.sbox;
            let x0 = self.config.margin + (bx as u32) * (BC as u32) * self.config.cell_size;
            let y0 = self.config.margin + (by as u32) * (BR as u32) * self.config.cell_size;
            draw_filled_rect_mut(
                img,
                Rect::at(x0 as i32, y0 as i32).of_size(
                    (BC as u32) * self.config.cell_size,
                    (BR as u32) * self.config.cell_size,
                ),
                colors.highlight_box,
            );
        }

        // Подсветка ячейки
        if rng.random_range(0..100) < 70 {
            let (r, c) = highlight.cell;
            let cell = boxes[r][c];
            draw_filled_rect_mut(
                img,
                Rect::at(cell.x as i32, cell.y as i32).of_size(cell.w, cell.h),
                colors.highlight_cell,
            );
        }
    }

    fn render_borders(&self, img: &mut RgbaImage, colors: &ColorPalette) {
        for i in 0..=N {
            let x = self.config.margin + i as u32 * self.config.cell_size;
            let y = self.config.margin + i as u32 * self.config.cell_size;

            // Вертикальные линии
            let v_thickness = self.get_line_thickness(i, BC);
            if v_thickness > 0 {
                draw_filled_rect_mut(
                    img,
                    Rect::at(
                        x as i32 - (v_thickness / 2) as i32,
                        self.config.margin as i32,
                    )
                    .of_size(v_thickness, self.config.board_size),
                    colors.border,
                );
            }

            // Горизонтальные линии
            let h_thickness = self.get_line_thickness(i, BR);
            if h_thickness > 0 {
                draw_filled_rect_mut(
                    img,
                    Rect::at(
                        self.config.margin as i32,
                        y as i32 - (h_thickness / 2) as i32,
                    )
                    .of_size(self.config.board_size, h_thickness),
                    colors.border,
                );
            }
        }
    }

    fn get_line_thickness(&self, i: usize, block: usize) -> u32 {
        if i == 0 || i == N || i % block == 0 {
            self.config.line_thick
        } else if self.config.do_cell_grid {
            self.config.line_thin
        } else {
            0
        }
    }

    fn render_numbers(
        &self,
        img: &mut RgbaImage,
        boxes: &[[CellBox; N]; N],
        colors: &ColorPalette,
        rng: &mut SmallRng,
    ) {
        // Получаем шрифт из кэша
        let font = FontCache::global().get_random(rng);

        // Определяем размер шрифта
        let small = rng.random_range(0..100) < 35;
        let scale = if small {
            PxScale {
                x: self.config.font_px / 2.0,
                y: self.config.font_px / 2.0,
            }
        } else {
            PxScale {
                x: self.config.font_px,
                y: self.config.font_px,
            }
        };

        // Рендерим числа
        for r in 0..N {
            for c in 0..N {
                let value = self.m[r][c];
                if value == 0 {
                    continue;
                }

                let cell = boxes[r][c];
                // без аллокаций строк:

                let s = DIGITS[value as usize];

                let vm = font.as_scaled(scale);
                let ch = s.chars().next().unwrap();
                let gid = font.glyph_id(ch);
                let adv = font.h_advance_unscaled(gid.with_scale(scale).id); // ширина глифа

                let jx: i32 = rng.random_range(-2..=2);
                let jy: i32 = rng.random_range(-2..=2);

                // центрирование по X по advance, по Y по ascent (базовая линия):
                // let tx = (cell.x as f32 + (cell.w as f32 - adv) * 0.5).round() as i32 + jx;
                // let ty = (cell.y as f32 + (cell.h as f32 + vm.ascent()) * 0.5).round() as i32 + jy;
                let tx = (cell.x as f32 + (cell.w as f32) * 0.5).round() as i32 + jx;
                let ty = (cell.y as f32 + (cell.h as f32) * 0.5).round() as i32 + jy;

                draw_text_mut(img, colors.border, tx, ty, scale, font, s);
            }
        }
    }

    fn save_image(&self, img: &RgbaImage, id: u32) -> image::ImageResult<()> {
        let out_path = Path::new(self.config.out_dir)
            .join("images")
            .join(format!("{id:06}.png"));

        if self.writer.is_none() {
            let dir = Path::new(self.config.out_dir).join("images");
            std::fs::create_dir_all(&dir)?;
        }

        img.save(out_path)
    }

    fn convert_boxes(&self, boxes: &[[CellBox; N]; N]) -> [[[u32; 4]; N]; N] {
        let mut result = [[[0u32; 4]; N]; N];

        for r in 0..N {
            for c in 0..N {
                result[r][c] = boxes[r][c].as_array();
            }
        }

        result
    }

    pub fn save_matrix(
        &mut self,
        id: u32,
        seed: u64,
        boxes: &[[[u32; 4]; N]; N],
        hl: Option<Highlight>,
    ) -> Result<(), Error> {
        let image_rel = format!("images/{id:06}.png");
        let rec = JsonRecord {
            image: image_rel,
            labels: self.m.iter().flatten().cloned().collect(),
            boxes: boxes.iter().flatten().cloned().collect(),
            seed,
            highlight: hl,
            dim: N as u8,
        };
        let json = serde_json::to_string(&rec).unwrap();

        // Инициализируем writer при первом вызове
        if self.writer.is_none() {
            let path = Path::new(self.config.out_dir).join("labels.jsonl");
            let file = File::create(path)?;
            self.writer = Some(BufWriter::with_capacity(8 << 20, file));
        }

        // Записываем в уже открытый файл
        if let Some(ref mut writer) = self.writer {
            writeln!(writer, "{}", json)?;
        }

        Ok(())
    }

    pub fn close_writer(&mut self) -> Result<(), Error> {
        if let Some(writer) = self.writer.take() {
            writer.into_inner()?.sync_all()?;
        }
        Ok(())
    }
}

impl<const N: usize, const BR: usize, const BC: usize> Drop
    for DatasetItemGenerator<'_, N, BR, BC>
{
    fn drop(&mut self) {
        let _ = self.close_writer();
    }
}
