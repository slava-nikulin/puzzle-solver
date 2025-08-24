use crate::{solver::SolveError, sudoku::Sudoku};

struct SudokuState {
    grid: [[u8; 9]; 9],
    // constraints: [[u16; 9]; 9],
    taken_stat: [u8; 9],
    last_edit_pos: (usize, usize),
    last_edit_val: u8,
    row_taken: [u16; 9],
    col_taken: [u16; 9],
    box_taken: [u16; 9],
}

impl SudokuState {
    fn new(grid: [[u8; 9]; 9]) -> Self {
        Self {
            grid,
            // constraints: [[0; 9]; 9],
            taken_stat: [0; 9],
            last_edit_pos: (0, 0),
            last_edit_val: 0,
            row_taken: [0; 9],
            col_taken: [0; 9],
            box_taken: [0; 9],
        }
    }

    fn avail_cand_count(bits: u16) -> u8 {
        (9 - bits.count_ones()) as u8
    }

    fn fill_bitsets(&mut self) {
        for i in 0..9 {
            for j in 0..9 {
                if self.grid[i][j] > 0 {
                    let taken_bit = self.grid[i][j] - 1;
                    self.col_taken[j] |= 1u16 << taken_bit;
                    self.row_taken[i] |= 1u16 << taken_bit;
                    self.box_taken[i / 3 * 3 + j / 3] |= 1u16 << taken_bit;
                }
            }
        }
    }

    fn taken_candidates(&mut self, i: usize, j: usize) -> u16 {
        self.box_taken[i / 3 * 3 + j / 3] | self.col_taken[j] | self.row_taken[i]
    }

    fn mark_candidate(&mut self, i: usize, j: usize, n: u8) {
        self.box_taken[i / 3 * 3 + j / 3] |= 1 << (n - 1);
        self.col_taken[j] |= 1 << (n - 1);
        self.row_taken[i] |= 1 << (n - 1);
    }
    // Minimum Remaining Values
    fn mrv(&mut self) -> Option<(usize, usize)> {
        let mut cell_to_fill: (usize, usize) = (10, 10);
        let mut min_avail_count = 10;
        let mut candidates: u16 = 0;

        // mark taken candidates
        'outer: loop {
            self.taken_stat = [0; 9];

            for i in 0..9 {
                for j in 0..9 {
                    if self.grid[i][j] == 0 {
                        // let i_small = i / 3;
                        // let j_small = j / 3;

                        // for k in 0..9 {
                        //     if self.grid[k][j] > 0 {
                        //         self.mark_candidate(i, j, self.grid[k][j]);
                        //     }

                        //     if self.grid[i][k] > 0 {
                        //         self.mark_candidate(i, j, self.grid[i][k]);
                        //     }
                        // }

                        // for l in 0..3 {
                        //     for m in 0..3 {
                        //         if self.grid[i_small * 3 + l][j_small * 3 + m] > 0 {
                        //             self.mark_candidate(
                        //                 i,
                        //                 j,
                        //                 self.grid[i_small * 3 + l][j_small * 3 + m],
                        //             );
                        //         }
                        //     }
                        // }

                        candidates = self.taken_candidates(i, j);

                        // self.taken_values[i][j] |= self.constraints[i][j];
                        if Self::avail_cand_count(candidates) == 0 {
                            // forward check
                            return None;
                        } else if Self::avail_cand_count(candidates) == 1 {
                            for k in 0..9 {
                                // singleton push
                                if candidates & (1 << k) == 0 {
                                    self.grid[i][j] = k + 1;
                                    self.mark_candidate(i, j, k);
                                    continue 'outer;
                                }
                            }
                        }
                    } else {
                        self.taken_stat[(self.grid[i][j] - 1) as usize] += 1;
                    }
                }
            }
            break 'outer;
        }

        // select candidate cell
        for i in 0..9 {
            for j in 0..9 {
                if self.grid[i][j] > 0 {
                    continue;
                }
                let available_count = Self::avail_cand_count(candidates);
                if min_avail_count >= available_count {
                    min_avail_count = available_count;
                    cell_to_fill = (i, j);
                } else if min_avail_count == available_count {
                    // tie break - TODO - перепроверить алгоритм, возможно, другой критерий лучше
                    // let mut curr_cell_freedom = 0;
                    // let mut selected_cell_freedom = 0;

                    // for k in 0..9 {
                    //     // число-кандидат на запись
                    //     if candidates & (1 << k) == 0 {
                    //         curr_cell_freedom += self.taken_stat[k];
                    //     }
                    //     if self.taken_values[cell_to_fill.0][cell_to_fill.1] & (1 << k) == 0 {
                    //         selected_cell_freedom += self.taken_stat[k];
                    //     }
                    // }

                    // if curr_cell_freedom > selected_cell_freedom {
                    //     cell_to_fill = (i, j)
                    // }
                }
            }
        }

        Some(cell_to_fill)
    }

    // Least Constraining Value
    fn lcv(&mut self, i: usize, j: usize) -> u8 {
        for k in 0..9 {
            if self.taken_candidates(i, j) & (1 << k) == 0 {
                return k + 1;
            }
        }

        0
    }

    // fn lcv(&self, i: usize, j: usize) -> u8 {
    //     let mut selected_val_density = 0;
    //     let mut selected_val = 0;

    //     for k in 0..9 {
    //         if (self.taken_values[i][j] & (1 << k) == 0)
    //             && selected_val_density <= self.taken_stat[k]
    //         {
    //             selected_val_density = self.taken_stat[k];
    //             selected_val = k + 1;
    //         }
    //     }

    //     selected_val as u8
    // }

    // fn mark_candidate(&mut self, i: usize, j: usize, n: u8) {
    //     self.taken_values[i][j] |= 1u16 << (n - 1)
    // }
}

pub struct DfsBacktracking;

impl DfsBacktracking {
    pub fn solve(&mut self, s: &mut Sudoku) -> Result<(), SolveError> {
        let mut init_state = SudokuState::new(s.init);
        init_state.fill_bitsets();
        let mut dfs_stack = vec![init_state];

        while !dfs_stack.is_empty() {
            if let Some(last_state) = dfs_stack.last_mut() {
                if let Some(cand_pos) = last_state.mrv() {
                    if cand_pos.0 > 9 {
                        s.solution = last_state.grid;
                        return Ok(());
                    }
                    let candidate = last_state.lcv(cand_pos.0, cand_pos.1);
                    if candidate == 0 {
                        return Err(SolveError::Unsolvable);
                    }
                    let mut new_state = SudokuState::new(last_state.grid);
                    new_state.grid[cand_pos.0][cand_pos.1] = candidate;
                    new_state.last_edit_pos = cand_pos;
                    new_state.last_edit_val = candidate;
                    // new_state.constraints = last_state.constraints;

                    dfs_stack.push(new_state);
                } else {
                    let incorrect_state = {
                        let this = dfs_stack.pop();
                        match this {
                            Some(val) => val,
                            None => return Err(SolveError::Unsolvable),
                        }
                    };

                    if let Some(last_correct_state) = dfs_stack.last_mut() {
                        last_correct_state.mark_candidate(
                            incorrect_state.last_edit_pos.0,
                            incorrect_state.last_edit_pos.1,
                            incorrect_state.last_edit_val,
                        );
                    } else {
                        return Err(SolveError::Unsolvable);
                    }
                }
            }
        }

        Err(SolveError::Unsolvable)
    }
}

// I don't utilize bitset operations features
