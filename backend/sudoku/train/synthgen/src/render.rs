pub struct RenderCfg {
    pub out_dir: String, // "dataset"
    pub img_w: u32,
    pub margin: u32,
    pub line_thin: u32,
    pub line_thick: u32,
    pub font_px: f32,
    pub do_cell_grid: bool,   // рисовать тонкие линии внутри 3x3
    pub with_highlight: bool, // включать подсветки
}

impl Default for RenderCfg {
    fn default() -> Self {
        Self {
            out_dir: "dataset9".to_string(),
            img_w: 512,
            margin: 16,
            line_thin: 2,
            line_thick: 5,
            font_px: 48.0,
            do_cell_grid: true,
            with_highlight: true,
        }
    }
}
