use image::Rgba;
use rand::{Rng, rngs::SmallRng};
use serde::Serialize;

pub struct ImageConfig<'a> {
    pub width: u32,
    pub height: u32,
    pub margin: u32,
    pub board_size: u32,
    pub cell_size: u32,
    pub with_highlight: bool,
    pub line_thick: u32,
    pub line_thin: u32,
    pub do_cell_grid: bool,
    pub out_dir: &'a str,
    pub font_px: f32,
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

pub struct ColorPalette {
    pub background: Rgba<u8>,
    pub border: Rgba<u8>,
    pub highlight_row: Rgba<u8>,
    pub highlight_col: Rgba<u8>,
    pub highlight_box: Rgba<u8>,
    pub highlight_cell: Rgba<u8>,
}

impl ColorPalette {
    pub fn new(rng: &mut SmallRng) -> Self {
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

#[derive(Serialize, Clone, Copy, Debug)]
pub struct Highlight {
    pub row: usize,
    pub col: usize,
    pub sbox: (usize, usize),
    pub cell: (usize, usize),
}
