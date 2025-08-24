use crate::{solver::SolveError, sudoku::Sudoku};
use rand::{Rng, SeedableRng, rngs::SmallRng};
pub struct StochasticBacktracking;

impl StochasticBacktracking {
    fn reset_square(&mut self, s: &mut Sudoku, i: usize, j: usize) {
        for p in 0..3 {
            for r in 0..3 {
                if s.init[i * 3 + p][j * 3 + r] == 0 {
                    s.solution[i * 3 + p][j * 3 + r] = 0;
                }
            }
        }
    }

    pub fn solve(&mut self, s: &mut Sudoku) -> Result<(), SolveError> {
        let mut rng = SmallRng::from_os_rng();
        let mut cands_mask: [u8; 9] = [0; 9];
        let mut cand: u8;
        let mut taken: u8;
        let mut i: usize = 0;
        let mut j: usize = 0;
        let mut attempts: [u8; 9] = [0; 9];
        let max_attemts: u8 = 10;

        'outer: loop {
            for t in (0..=(i * 3 + j)).rev() {
                self.reset_square(s, t / 3, t % 3);

                if attempts[t] < max_attemts {
                    attempts[t] += 1;
                    if t == 0 && attempts[t] >= max_attemts {
                        attempts.fill(0);
                        i = 0;
                        j = 0;
                        continue 'outer;
                    }
                    attempts[(t + 1)..9].fill(0);
                    i = t / 3;
                    j = t % 3;
                    break;
                }
            }
            while i < 3 {
                while j < 3 {
                    for k in 0..3 {
                        for l in 0..3 {
                            if s.solution[i * 3 + k][j * 3 + l] == 0 {
                                cands_mask.fill(1);
                                for q in 0..9 {
                                    taken = s.solution[i * 3 + k][q];
                                    if taken > 0 {
                                        cands_mask[(taken - 1) as usize] = 0;
                                    }

                                    taken = s.solution[q][j * 3 + l];
                                    if taken > 0 {
                                        cands_mask[(taken - 1) as usize] = 0;
                                    }
                                }

                                for p in 0..3 {
                                    for r in 0..3 {
                                        taken = s.solution[i * 3 + p][j * 3 + r];
                                        if taken > 0 {
                                            cands_mask[(taken - 1) as usize] = 0;
                                        }
                                    }
                                }

                                let cands: Vec<u8> =
                                    cands_mask
                                        .iter()
                                        .enumerate()
                                        .filter_map(|(idx, &val)| {
                                            if val > 0 { Some((idx + 1) as u8) } else { None }
                                        })
                                        .collect();

                                if cands.is_empty() {
                                    continue 'outer;
                                }

                                cand = cands[rng.random_range(0..cands.len())];

                                s.solution[i * 3 + k][j * 3 + l] = cand;
                            }
                        }
                    }
                    j += 1;
                }
                i += 1;
                j = 0;
            }
            break 'outer;
        }

        if s.check() {
            Ok(())
        } else {
            Err(SolveError::Unsolvable)
        }
    }
}
