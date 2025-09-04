use rand::{RngCore, SeedableRng};
use rand_xoshiro::SplitMix64;

use crate::{generator::DatasetItemGenerator, render::RenderCfg};

mod generator;
mod render;

const TRAIN_DATA_NUM: i32 = 5;
fn main() -> anyhow::Result<()> {
    let mut t = DatasetItemGenerator::<9, 3, 3> { m: [[0; 9]; 9] };
    let cfg = &RenderCfg::default();
    for i in 0..TRAIN_DATA_NUM {
        let mut sm = SplitMix64::seed_from_u64(i as u64);
        let seed = sm.next_u64();

        t.generate_with_seed(seed);
        let (boxes, hl) = t.save_img(i as u32, seed, cfg)?;
        t.save_matrix(i as u32, seed, &boxes, hl, cfg)?;
        t.reset_matrix();
    }

    Ok(())
}
