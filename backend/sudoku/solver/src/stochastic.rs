use crate::sudoku::Sudoku;
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

    pub fn solve(&mut self, s: &mut Sudoku) -> bool {
        let mut rng = SmallRng::from_os_rng();
        let mut cands_mask: [i8; 9] = [0; 9];
        let mut cand: i8;
        let mut taken: i8;
        let mut i: usize = 0;
        let mut j: usize = 0;
        let mut attempts: [i8; 9] = [0; 9];
        let max_attemts: i8 = 10;

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

                                let cands: Vec<i8> =
                                    cands_mask
                                        .iter()
                                        .enumerate()
                                        .filter_map(|(idx, &val)| {
                                            if val > 0 { Some((idx + 1) as i8) } else { None }
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

        s.check()
    }
}

// impl Solver for StochasticBacktracking {

// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let init_sudoku = [
//             [9, 0, 6, 3, 4, 0, 8, 1, 0],
//             [0, 5, 1, 7, 0, 0, 3, 0, 0],
//             [4, 7, 0, 0, 9, 1, 0, 0, 5],
//             [0, 0, 0, 9, 0, 3, 0, 0, 2],
//             [0, 0, 2, 0, 8, 7, 0, 0, 0],
//             [1, 0, 7, 2, 0, 0, 6, 0, 0],
//             [0, 8, 5, 0, 0, 9, 1, 0, 0],
//             [0, 3, 4, 0, 6, 0, 0, 0, 9],
//             [0, 1, 0, 5, 0, 8, 7, 0, 6],
//         ];
//         let expected = [
//             [9, 2, 6, 3, 4, 5, 8, 1, 7],
//             [8, 5, 1, 7, 2, 6, 3, 9, 4],
//             [4, 7, 3, 8, 9, 1, 2, 6, 5],
//             [5, 6, 8, 9, 1, 3, 4, 7, 2],
//             [3, 4, 2, 6, 8, 7, 9, 5, 1],
//             [1, 9, 7, 2, 5, 4, 6, 3, 8],
//             [6, 8, 5, 4, 7, 9, 1, 2, 3],
//             [7, 3, 4, 1, 6, 2, 5, 8, 9],
//             [2, 1, 9, 5, 3, 8, 7, 4, 6],
//         ];
//         let mut sudoku = Sudoku::new(init_sudoku);
//         let res = sudoku.solve();

//         assert_eq!(res, expected);
//         assert!(Sudoku::check(res))
//     }
// }
