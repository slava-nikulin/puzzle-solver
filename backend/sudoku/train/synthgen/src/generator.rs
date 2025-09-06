use crate::{
    fonts::{self, FontCache, get_metrics_pack},
    geom::{CellBox, CellGrid},
    render::{ColorPalette, Highlight, ImageConfig},
};
use ab_glyph::PxScale;
use image::RgbaImage;
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::{
    fs::File,
    io::BufWriter,
};

pub struct DatasetItemGenerator<'a, const N: usize, const BR: usize, const BC: usize> {
    pub m: [[u8; N]; N],
    pub(crate) writer: Option<BufWriter<File>>,
    pub config: ImageConfig<'a>,
}

impl<const N: usize, const BR: usize, const BC: usize> DatasetItemGenerator<'_, N, BR, BC> {
    // Public API
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
    pub fn render_and_save_image(
        &self,
        id: u32,
        seed: u64,
    ) -> image::ImageResult<(CellGrid<N>, Option<Highlight>)> {
        debug_assert_eq!(N, BR * BC);

        let mut rng = SmallRng::seed_from_u64(seed);
        let colors = ColorPalette::new(&mut rng);

        let mut img =
            RgbaImage::from_pixel(self.config.width, self.config.height, colors.background);
        let boxes = self.calculate_cell_boxes();
        let highlight = self.generate_highlight(&mut rng);

        if let Some(hl) = highlight {
            self.render_highlights(&mut img, &hl, &boxes, &colors, &mut rng);
        }
        self.render_borders(&mut img, &colors);
        self.render_numbers(&mut img, &boxes, &colors, &mut rng);

        // Image geometry invariants
        debug_assert!(self.config.board_size >= self.config.cell_size * (N as u32));
        debug_assert!(
            self.config.board_size - self.config.cell_size * (N as u32) < self.config.cell_size
        );

        self.save_png(&img, id)?;

        Ok((boxes, highlight))
    }

    // Helpers
    fn calculate_cell_boxes(&self) -> CellGrid<N> {
        let mut boxes = [[CellBox::new(0, 0, 0, 0); N]; N];
        let w = self.config.board_size;
        let h = self.config.board_size;
        let cs = self.config.cell_size;

        for (r, row) in boxes.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
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
                *cell = CellBox::new(x, y, ww, hh);
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
        boxes: &CellGrid<N>,
        colors: &ColorPalette,
        rng: &mut SmallRng,
    ) {
        // Highlight the selected row
        if rng.random_range(0..100) < 70 {
            let y = boxes[highlight.row][0].y;
            draw_filled_rect_mut(
                img,
                Rect::at(self.config.margin as i32, y as i32)
                    .of_size(self.config.board_size, self.config.cell_size),
                colors.highlight_row,
            );
        }

        // Highlight the selected column
        if rng.random_range(0..100) < 70 {
            let x = boxes[0][highlight.col].x;
            draw_filled_rect_mut(
                img,
                Rect::at(x as i32, self.config.margin as i32)
                    .of_size(self.config.cell_size, self.config.board_size),
                colors.highlight_col,
            );
        }

        // Highlight the selected sub-box
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

        // Highlight the selected cell
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

            // Vertical grid lines
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

            // Horizontal grid lines
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
        boxes: &CellGrid<N>,
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

        for (r, row) in self.m.iter().enumerate() {
            for (c, m_val) in row.iter().enumerate() {
                if *m_val == 0 {
                    continue;
                }
                let cell = boxes[r][c];

                // Precomputed digit metrics
                let dm = &pack.digits[*m_val as usize];
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
                    fonts::DIGITS[*m_val as usize],
                );
            }
        }
    }

}

impl<const N: usize, const BR: usize, const BC: usize> Drop
    for DatasetItemGenerator<'_, N, BR, BC>
{
    fn drop(&mut self) {
        let _ = self.finalize_output();
    }
}
