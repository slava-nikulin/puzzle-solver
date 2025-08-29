use rand::{Rng, SeedableRng, rngs::SmallRng};

pub struct DatasetItemGenerator {
    m: [[u8; 9]; 9],
}

impl DatasetItemGenerator {
    pub fn new() -> Self {
        Self { m: [[0; 9]; 9] }
    }

    pub fn reset_matrix(&mut self) {
        for r in 0..9 {
            for c in 0..9 {
                self.m[r][c] = 0;
            }
        }
    }

    pub fn generate(&mut self) {
        let mut rng = SmallRng::from_os_rng();
        let probability_of_zero = rng.random_range(60..=75);

        for r in 0..9 {
            for c in 0..9 {
                if rng.random_range(0..100) < probability_of_zero {
                    self.m[r][c] = 0;
                } else {
                    self.m[r][c] = rng.random_range(0..=9);
                }
            }
        }
    }

    pub fn save_matrix(&self) {}

    pub fn save_img(&self) {}
}
