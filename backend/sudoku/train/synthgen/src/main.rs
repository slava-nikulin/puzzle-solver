use rand::{RngCore, SeedableRng};
use rand_xoshiro::SplitMix64;

use crate::{generator::DatasetItemGenerator, render::ImageConfig};

mod fonts;
mod geom;
mod generator;
mod io;
mod record;
mod render;

const TRAIN_DATA_NUM: i32 = 5;
fn main() -> anyhow::Result<()> {
    let mut t = DatasetItemGenerator::<9, 3, 3> {
        m: [[0; 9]; 9],
        writer: None,
        config: ImageConfig::new(9),
    };
    t.init_output()?;
    for i in 0..TRAIN_DATA_NUM {
        let mut sm = SplitMix64::seed_from_u64(i as u64);
        let seed = sm.next_u64();

        t.generate_with_seed(seed);
        let (boxes, hl) = t.render_and_save_image(i as u32, seed)?;
        t.write_labels_jsonl(i as u32, seed, &boxes, hl)?;
        t.reset_matrix();
    }

    Ok(())
}
