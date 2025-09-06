use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct CellBox {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl CellBox {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }
}

pub type CellGrid<const N: usize> = [[CellBox; N]; N];

