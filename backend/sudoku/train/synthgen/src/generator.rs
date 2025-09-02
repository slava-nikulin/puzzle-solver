use std::path::Path;

use image::{Rgba, RgbaImage};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use rusttype::{Font, Scale};
use serde::Serialize;

use crate::render::RenderCfg;

#[derive(Serialize)]
struct JsonRecord {
    image: String,
    labels: Vec<u8>,
    boxes: Vec<[u32; 4]>,
    seed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    highlight: Option<Highlight>,
}

#[derive(Serialize, Clone, Copy)]
struct Highlight {
    row: usize,
    col: usize,
    cell: (usize, usize),
    sbox: (usize, usize),
}

pub struct DatasetItemGenerator<const N: usize, const BR: usize, const BC: usize> {
    pub m: [[u8; N]; N],
}

impl<const N: usize, const BR: usize, const BC: usize> DatasetItemGenerator<N, BR, BC> {
    pub fn reset_matrix(&mut self) {
        self.m.iter_mut().for_each(|row| row.fill(0));
    }

    pub fn generate(&mut self) {
        let mut rng = SmallRng::from_os_rng();
        let probability_of_zero = rng.random_range(60..=75);

        for row in 0..N {
            for col in 0..N {
                let cell = if rng.random_range(0..100) < probability_of_zero {
                    0
                } else {
                    rng.random_range(0..=N)
                };
                self.m[row][col] = cell;
            }
        }
    }

    fn save_img(
        &self,
        id: u32,
        seed: u64,
        cfg: &RenderCfg,
    ) -> image::ImageResult<[[u32; 4]; N * N]> {
        // геометрия
        let w = cfg.img_w;
        let h = cfg.img_w;
        let margin = cfg.margin;
        let board = w - 2 * margin;
        let cell: u32 = board / N;

        let mut img = RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]));
        let black = Rgba([20, 30, 40, 255]);
        let hl_row = Rgba([120, 170, 255, 70]);
        let hl_col = Rgba([120, 170, 255, 70]);
        let hl_box = Rgba([120, 170, 255, 50]);
        let hl_cell = Rgba([120, 170, 255, 120]);

        // bbox'ы клеток
        let mut boxes = [[0u32; 4]; N * N];
        for r in 0..N {
            for c in 0..N {
                let x = margin + c as u32 * cell;
                let y = margin + r as u32 * cell;
                boxes[r * N + c] = [x, y, cell, cell];
            }
        }

        // случайные подсветки (детерминированы seed'ом)
        let mut rng = SmallRng::seed_from_u64(seed);
        let hl = if cfg.with_highlight {
            Some(Highlight {
                row: rng.random_range(0..N),
                col: rng.random_range(0..N),
                sbox: (rng.random_range(0..BR), rng.random_range(0..BC)),
                cell: (rng.random_range(0..N), rng.random_range(0..N)),
            })
        } else {
            None
        };

        // рендер подсветок
        if let Some(hh) = hl {
            // row
            let y = boxes[hh.row * N].1();
            draw_filled_rect_mut(
                &mut img,
                Rect::at(margin as i32, y as i32).of_size(board, cell),
                hl_row,
            );
            // col
            let x = boxes[hh.col].0();
            draw_filled_rect_mut(
                &mut img,
                Rect::at(x as i32, margin as i32).of_size(cell, board),
                hl_col,
            );
            // box 3x3
            let x0: u32 = margin + (hh.sbox.1 as u32) * BR * cell;
            let y0: u32 = margin + (hh.sbox.0 as u32) * BC * cell;
            draw_filled_rect_mut(
                &mut img,
                Rect::at(x0 as i32, y0 as i32).of_size(BR * cell, BC * cell),
                hl_box,
            );
            // cell
            let (r, c) = hh.cell;
            let [x, y, wc, hc] = boxes[r * N + c];
            draw_filled_rect_mut(
                &mut img,
                Rect::at(x as i32, y as i32).of_size(wc, hc),
                hl_cell,
            );
        }

        // толстые границы 3x3
        for i in 0..=N {
            let t = if i % 3 == 0 {
                cfg.line_thick
            } else if cfg.do_cell_grid {
                cfg.line_thin
            } else {
                0
            };
            if t > 0 {
                let x = margin + i as u32 * cell;
                let y = margin + i as u32 * cell;
                // вертикаль
                draw_filled_rect_mut(
                    &mut img,
                    Rect::at((x as i32 - (t / 2) as i32), margin as i32).of_size(t, board),
                    black,
                );
                // горизонталь
                draw_filled_rect_mut(
                    &mut img,
                    Rect::at(margin as i32, (y as i32 - (t / 2) as i32)).of_size(board, t),
                    black,
                );
            }
        }

        // цифры
        // Вшитый шрифт (любой TTF, тут DejaVuSansMono)
        static FONT_BYTES: &[u8] = include_bytes!("DejaVuSansMono.ttf");
        let font = Font::try_from_bytes(FONT_BYTES).unwrap();
        let scale = Scale::uniform(cfg.font_px);

        for r in 0..N {
            for c in 0..N {
                let v = self.m[r][c];
                if v == 0 {
                    continue;
                }
                let [x, y, wc, hc] = boxes[r * N + c];
                // центрирование приблизительное
                let tx = x + wc / 3;
                let ty = y + (2 * hc) / 3;
                draw_text_mut(
                    &mut img,
                    black,
                    tx as i32,
                    ty as i32,
                    scale,
                    &font,
                    &v.to_string(),
                );
            }
        }

        // сохранить
        let out_path = Path::new(&cfg.out_dir)
            .join("images")
            .join(format!("{id:06}.png"));
        std::fs::create_dir_all(out_path.parent().unwrap()).ok();
        img.save(out_path)?;

        Ok(boxes)
    }

    fn save_matrix(
        &self,
        id: u32,
        seed: u64,
        boxes: &[[u32; 4]; N * N],
        hl: Option<Highlight>,
        cfg: &RenderCfg,
    ) -> std::io::Result<()> {
        let image_rel = format!("images/{id:06}.png");
        let rec = JsonRecord {
            image: image_rel,
            labels: self.m.iter().flatten().cloned().collect(),
            boxes: boxes.iter().cloned().collect(),
            seed,
            highlight: hl,
        };
        let json = serde_json::to_string(&rec).unwrap();

        let path = Path::new(&cfg.out_dir).join("labels.jsonl");
        let mut f = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(f, "{json}")?;
        Ok(())
    }
}

trait XYWH {
    fn x(&self) -> u32;
    fn y(&self) -> u32;
}
impl XYWH for [u32; 4] {
    fn x(&self) -> u32 {
        self[0]
    }
    fn y(&self) -> u32 {
        self[1]
    }
}
