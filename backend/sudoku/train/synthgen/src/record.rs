use serde::Serialize;

use crate::render::Highlight;
use crate::geom::CellBox;

#[derive(Serialize, Debug)]
pub struct JsonRecord {
    pub schema: &'static str,
    pub image: String,
    pub labels: Vec<u8>,
    pub boxes: Vec<CellBox>,
    pub dim: u8,
    pub seed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight: Option<Highlight>,
}

