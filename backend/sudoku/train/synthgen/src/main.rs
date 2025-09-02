use crate::generator::DatasetItemGenerator;

mod generator;
mod render;

const TRAIN_DATA_NUM: i32 = 3;
fn main() -> anyhow::Result<()> {
    let mut t = DatasetItemGenerator::<9, 3, 3> { m: [[0; 9]; 9] };

    let mut global = 0u32;
    for i in 0..TRAIN_DATA_NUM {
        t.generate();

        // let seed = (global as u64) * 1_000_003 + 0x9E3779B97F4A7C15;
        // let boxes = t.save_img(global, seed, &cfg)?;

        // t.save_matrix();
        // t.save_img();

        t.reset_matrix();
    }

    Ok(())
}
