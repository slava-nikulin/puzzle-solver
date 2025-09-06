use crate::{
    generator::DatasetItemGenerator,
    geom::{CellBox, CellGrid},
    record::JsonRecord,
    render::Highlight,
};
use image::RgbaImage;
use serde_json;
use std::{
    fs::File,
    io::{BufWriter, Error, Write},
    path::Path,
};

impl<'a, const N: usize, const BR: usize, const BC: usize>
    DatasetItemGenerator<'a, N, BR, BC>
{
    pub fn init_output(&mut self) -> std::io::Result<()> {
        let dir = std::path::Path::new(self.config.out_dir).join("images");
        std::fs::create_dir_all(&dir)?;
        if self.writer.is_none() {
            let path = std::path::Path::new(self.config.out_dir).join("labels.jsonl");
            let file = File::create(path)?;
            self.writer = Some(BufWriter::with_capacity(8 << 20, file));
        }
        Ok(())
    }

    pub fn write_labels_jsonl(
        &mut self,
        id: u32,
        seed: u64,
        boxes: &CellGrid<N>,
        hl: Option<Highlight>,
    ) -> Result<(), Error> {
        let image_rel = format!("images/{id:06}.png");
        let flat_boxes: Vec<CellBox> = boxes.iter().flatten().copied().collect();
        let rec = JsonRecord {
            schema: "v1",
            image: image_rel,
            labels: self.m.iter().flatten().cloned().collect(),
            boxes: flat_boxes,
            seed,
            highlight: hl,
            dim: N as u8,
        };
        let json = serde_json::to_string(&rec).unwrap();

        if let Some(ref mut writer) = self.writer {
            writeln!(writer, "{}", json)?;
        }

        Ok(())
    }

    pub fn finalize_output(&mut self) -> Result<(), Error> {
        if let Some(writer) = self.writer.take() {
            writer.into_inner()?.sync_all()?;
        }
        Ok(())
    }

    pub(crate) fn save_png(&self, img: &RgbaImage, id: u32) -> image::ImageResult<()> {
        let out_path = Path::new(self.config.out_dir)
            .join("images")
            .join(format!("{id:06}.png"));
        img.save(out_path)
    }
}
