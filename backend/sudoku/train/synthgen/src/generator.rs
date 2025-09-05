use ab_glyph::PxScale;
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
    fonts::{self, FontCache, get_metrics_pack},
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

#[derive(Serialize)]
struct JsonRecord {
    schema: &'static str,
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

    // fn render_numbers(
    //     &self,
    //     img: &mut RgbaImage,
    //     boxes: &[[CellBox; N]; N],
    //     colors: &ColorPalette,
    //     rng: &mut SmallRng,
    // ) {
    //     let cache = FontCache::global();
    //     let small = rng.random_range(0..100) < 35;
    //     let scale = if small {
    //         PxScale {
    //             x: self.config.font_px / 2.0,
    //             y: self.config.font_px / 2.0,
    //         }
    //     } else {
    //         PxScale {
    //             x: self.config.font_px,
    //             y: self.config.font_px,
    //         }
    //     };

    //     let font = cache.get_random(rng);
    //     let base_jitter_px: f32 = if small { 1.0 } else { 2.0 };
    //     // Масштабированное лицо шрифта для метрик ascent/descent/height
    //     let sf = font.as_scaled(scale); // дает доступ к ascent(), descent(), height() в пикселях [7]

    //     for r in 0..N {
    //         for c in 0..N {
    //             let value = self.m[r][c];
    //             if value == 0 {
    //                 continue;
    //             }

    //             let cell = boxes[r][c];
    //             let text = fonts::DIGITS[value as usize];

    //             // 1) Горизонтальный центр — по реальной ширине контура одного глифа
    //             let gid = font.glyph_id(text.chars().next().unwrap());
    //             let glyph0 = gid.with_scale_and_position(scale, point(0.0, 0.0));
    //             let outlined0 = font.outline_glyph(glyph0).expect("font must have outlines"); // [2]
    //             let b = outlined0.px_bounds(); // точные пиксельные границы контура [2][1]
    //             let bw = (b.max.x - b.min.x) as f32;

    //             // 2) Вертикальный центр — по макетной высоте (ascent - descent),
    //             // без line_gap для одиночной строки
    //             let layout_h = sf.height(); // ascent - descent (descent отрицателен) [7]
    //             let cx = cell.x as f32 + 0.5 * cell.w as f32;
    //             let cy = cell.y as f32 + 0.5 * cell.h as f32;

    //             let mut tl_x = (cx - 0.5 * bw).round();
    //             let mut tl_y = (cy - 0.5 * layout_h).round(); // верх макетной рамки [8][7]

    //             let jx = rng.random_range(-base_jitter_px..=base_jitter_px);
    //             let jy = rng.random_range(-base_jitter_px..=base_jitter_px);

    //             tl_x += jx;
    //             tl_y += jy;

    //             // draw_text_mut ожидает верхний‑левый угол макетной рамки
    //             draw_text_mut(
    //                 img,
    //                 colors.border,
    //                 tl_x as i32,
    //                 tl_y as i32,
    //                 scale,
    //                 font,
    //                 text,
    //             );
    //         }
    //     }
    // }

    fn render_numbers(
        &self,
        img: &mut RgbaImage,
        boxes: &[[CellBox; N]; N],
        colors: &ColorPalette,
        rng: &mut SmallRng,
    ) {
        let small = rng.random_range(0..100) < 35;
        let px = if small {
            (self.config.font_px / 2.0).round() as u32
        } else {
            self.config.font_px.round() as u32
        };
        let scale = PxScale {
            x: px as f32,
            y: px as f32,
        };

        let (font, font_name) = FontCache::global().get_random_named(rng);
        let pack = get_metrics_pack(font_name, font, px);
        let layout_h = pack.height;

        for r in 0..N {
            for c in 0..N {
                let v = self.m[r][c];
                if v == 0 {
                    continue;
                }
                let cell = boxes[r][c];

                // предрасчитанные метрики
                let dm = &pack.digits[v as usize];
                let cx = cell.x as f32 + 0.5 * cell.w as f32;
                let cy = cell.y as f32 + 0.5 * cell.h as f32;

                let jx: f32 = if small {
                    rng.random_range(-1.0..=1.0)
                } else {
                    rng.random_range(-2.0..=2.0)
                };
                let jy: f32 = if small {
                    rng.random_range(-1.0..=1.0)
                } else {
                    rng.random_range(-2.0..=2.0)
                };

                let tl_x = (cx - 0.5 * dm.bw + jx).round() as i32;
                let tl_y = (cy - 0.5 * layout_h + jy).round() as i32;

                draw_text_mut(
                    img,
                    colors.border,
                    tl_x,
                    tl_y,
                    scale,
                    font,
                    fonts::DIGITS[v as usize],
                );
            }
        }
    }

    fn save_image(&self, img: &RgbaImage, id: u32) -> image::ImageResult<()> {
        debug_assert_eq!(N, BR * BC);
        debug_assert!(self.config.board_size >= self.config.cell_size * (N as u32));
        debug_assert!(
            self.config.board_size - self.config.cell_size * (N as u32) < self.config.cell_size
        );

        let out_path = Path::new(self.config.out_dir)
            .join("images")
            .join(format!("{id:06}.png"));

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
            schema: "v1",
            image: image_rel,
            labels: self.m.iter().flatten().cloned().collect(),
            boxes: boxes.iter().flatten().cloned().collect(),
            seed,
            highlight: hl,
            dim: N as u8,
        };
        let json = serde_json::to_string(&rec).unwrap();

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

    pub fn init_io(&mut self) -> std::io::Result<()> {
        let dir = std::path::Path::new(self.config.out_dir).join("images");
        std::fs::create_dir_all(&dir)?;
        if self.writer.is_none() {
            let path = std::path::Path::new(self.config.out_dir).join("labels.jsonl");
            let file = std::fs::File::create(path)?;
            self.writer = Some(std::io::BufWriter::with_capacity(8 << 20, file));
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
