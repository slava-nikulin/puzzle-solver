use core::fmt;
use std::fs::DirBuilder;

use rand::{Rng, SeedableRng, rngs::SmallRng};

fn main() {
    let mut init_sudoku = [[0; 9]; 9];

    init_sudoku = [
        [9, 0, 6, 3, 4, 0, 8, 1, 0],
        [0, 5, 1, 7, 0, 0, 3, 0, 0],
        [4, 7, 0, 0, 9, 1, 0, 0, 5],
        [0, 0, 0, 9, 0, 3, 0, 0, 2],
        [0, 0, 2, 0, 8, 7, 0, 0, 0],
        [1, 0, 7, 2, 0, 0, 6, 0, 0],
        [0, 8, 5, 0, 0, 9, 1, 0, 0],
        [0, 3, 4, 0, 6, 0, 0, 0, 9],
        [0, 1, 0, 5, 0, 8, 7, 0, 6],
    ];

    let mut sudoku = Sudoku::new(init_sudoku);

    sudoku.solve();

    println!("{}", sudoku);
    println!("{}", sudoku.check());
}

struct Sudoku {
    data: [[i8; 9]; 9],
    init: [[i8; 9]; 9],
}

impl Sudoku {
    fn new(init: [[i8; 9]; 9]) -> Self {
        Sudoku { data: init, init }
    }

    fn solve(&mut self) {
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
                self.reset_square(t / 3, t % 3);

                if attempts[t] < max_attemts {
                    attempts[t] += 1;
                    if t == 0 && attempts[t] >= max_attemts {
                        attempts.fill(0);
                        i = 0;
                        j = 0;
                        continue 'outer;
                    }
                    for t1 in (t + 1)..9 {
                        attempts[t1] = 0;
                    }
                    i = t / 3;
                    j = t % 3;
                    break;
                }
            }
            'square_i: while i < 3 {
                // j = 0;
                'square_j: while j < 3 {
                    for k in 0..3 {
                        for l in 0..3 {
                            if self.data[i * 3 + k][j * 3 + l] == 0 {
                                cands_mask.fill(1);
                                for q in 0..9 {
                                    taken = self.data[i * 3 + k][q];
                                    if taken > 0 {
                                        cands_mask[(taken - 1) as usize] = 0;
                                    }

                                    taken = self.data[q][j * 3 + l];
                                    if taken > 0 {
                                        cands_mask[(taken - 1) as usize] = 0;
                                    }
                                }

                                for p in 0..3 {
                                    for r in 0..3 {
                                        taken = self.data[i * 3 + p][j * 3 + r];
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

                                self.data[i * 3 + k][j * 3 + l] = cand;
                            }
                        }
                    }
                    j += 1;
                }
                i += 1;
                j = 0;
            }
            dbg!("test");
            break 'outer;
        }
    }

    fn check(&self) -> bool {
        let mut small_square_row_num: usize;
        let mut small_square_col_num: usize;
        let mut small_square_row_num_check: usize;
        let mut small_square_col_num_check: usize;
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    small_square_row_num = i * 3 + k;
                    for l in 0..3 {
                        small_square_col_num = j * 3 + l;

                        for n in 0..3 {
                            small_square_row_num_check = i * 3 + n;
                            for m in 0..3 {
                                small_square_col_num_check = j * 3 + m;

                                if small_square_row_num != small_square_row_num_check
                                    && small_square_col_num != small_square_col_num_check
                                    && self.data[small_square_row_num][small_square_col_num]
                                        == self.data[small_square_row_num_check]
                                            [small_square_col_num_check]
                                {
                                    return false;
                                }
                            }
                        }

                        for n in 0..9 {
                            if (n != small_square_row_num
                                && self.data[n][small_square_col_num]
                                    == self.data[small_square_row_num][small_square_col_num])
                                || (n != small_square_col_num
                                    && self.data[small_square_row_num][n]
                                        == self.data[small_square_row_num][small_square_col_num])
                            {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        true
    }

    fn reset_square(&mut self, i: usize, j: usize) {
        for p in 0..3 {
            for r in 0..3 {
                if self.init[i * 3 + p][j * 3 + r] == 0 {
                    self.data[i * 3 + p][j * 3 + r] = 0;
                }
            }
        }
    }

    // fn check_cand(i: usize, j: usize, sudoku: [[i8; 9]; 9], cand: i8) -> bool {
    //     let i_small = i / 3;
    //     let j_small = j / 3;

    //     for k in 0..3 {
    //         for l in 0..3 {
    //             if sudoku[i_small * 3 + k][j_small * 3 + l] == cand {
    //                 return false;
    //             }
    //         }
    //     }

    //     for k in 0..9 {
    //         if (k != i && sudoku[k][j] == cand) || (k != j && sudoku[i][k] == cand) {
    //             return false;
    //         }
    //     }

    //     true
    // }
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.data {
            for val in row {
                write!(f, "{} ", val)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
