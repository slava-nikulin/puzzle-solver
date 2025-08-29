use crate::generator::DatasetItemGenerator;

mod generator;

fn main() {
    let mut t = DatasetItemGenerator::new();
    for i in 0..200_000 {
        t.generate();

        t.save_matrix();
        t.save_img();

        t.reset_matrix();
    }
}
