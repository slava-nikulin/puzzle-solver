use ab_glyph::{Font, PxScale, ScaleFont, point};
use image::RgbaImage;
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use serde::Serialize;
use std::{
    fs::File,
    io::{BufWriter, Error, Write},
    path::Path,
};

use crate::{
    fonts::{self, FontCache},
    render::{ColorPalette, Highlight, ImageConfig},
};

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
        // 1) Шрифт и масштаб
        let font = FontCache::global().get_random(rng);
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

        // 2) Рендер каждой цифры
        for r in 0..N {
            for c in 0..N {
                let value = self.m[r][c];
                if value == 0 {
                    continue;
                }

                let cell = boxes[r][c];
                let ch = fonts::DIGITS[value as usize];

                // Построить глиф в (0,0), чтобы померить px_bounds
                let gid = font.glyph_id(ch);
                let glyph0 = gid.with_scale_and_position(scale, point(0.0, 0.0));
                if let Some(outlined0) = font.outline_glyph(glyph0) {
                    let b = outlined0.px_bounds();
                    let bw = b.max.x - b.min.x;
                    let bh = b.max.y - b.min.y;

                    let cx = cell.x as f32 + 0.5 * cell.w as f32;
                    let cy = cell.y as f32 + 0.5 * cell.h as f32;

                    let tl_x = (cx - 0.5 * bw).round() as i32;
                    let tl_y = (cy - 0.7 * bh).round() as i32;

                    draw_text_mut(img, colors.border, tl_x, tl_y, scale, font, &ch.to_string());
                }
                // Никакого return здесь — продолжаем рендер сетки
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
