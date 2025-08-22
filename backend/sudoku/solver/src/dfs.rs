use crate::sudoku::Sudoku;

struct SudokuState {
    grid: [[u8; 9]; 9],
    taken_values: [[u16; 9]; 9],
    taken_stat: [u8; 9],
}

impl SudokuState {
    fn new(grid: [[u8; 9]; 9]) -> Self {
        Self {
            grid,
            taken_values: [[0; 9]; 9],
            taken_stat: [0; 9],
        }
    }
    // Minimum Remaining Values
    fn mrv(&mut self) -> Option<(usize, usize)> {
        let mut cell_to_fill: (usize, usize) = (0, 0);
        let mut min_cand_count = 10;

        'outer: loop {
            self.taken_values = [[0; 9]; 9];
            self.taken_stat = [0; 9];

            for i in 0..9 {
                for j in 0..9 {
                    if self.grid[i][j] == 0 {
                        let i_small = i / 3;
                        let j_small = j / 3;

                        for k in 0..9 {
                            if self.grid[k][j] > 0 {
                                self.mark_candidate(i, j, self.grid[k][j]);
                            }

                            if self.grid[i][k] > 0 {
                                self.mark_candidate(i, j, self.grid[i][k]);
                            }
                        }

                        for l in 0..3 {
                            for m in 0..3 {
                                if self.grid[i_small * 3 + l][j_small * 3 + m] > 0 {
                                    self.mark_candidate(
                                        i,
                                        j,
                                        self.grid[i_small * 3 + l][j_small * 3 + m],
                                    );
                                }
                            }
                        }
                        if self.taken_values[i][j].count_ones() >= 9 {
                            // forward check
                            return None;
                        } else if self.taken_values[i][j].count_ones() == 1 {
                            for k in 0..9 {
                                // singleton push
                                if self.taken_values[i][j] & (1 << k) == 0 {
                                    self.grid[i][j] = k + 1;
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

        for i in 0..9 {
            for j in 0..9 {
                let taken_count = self.taken_values[i][j].count_ones();
                if min_cand_count > taken_count {
                    min_cand_count = taken_count;
                    cell_to_fill = (i, j);
                } else if min_cand_count == taken_count {
                    // tie break - TODO - перепроверить алгоритм, возможно, другой критерий лучше
                    let mut curr_cell_freedom = 0;
                    let mut selected_cell_freedom = 0;

                    for k in 0..9 {
                        // число-кандидат на запись
                        if self.taken_values[i][j] & (1 << k) == 0 {
                            curr_cell_freedom += self.taken_stat[k];
                        }
                        if self.taken_values[cell_to_fill.0][cell_to_fill.1] & (1 << k) == 0 {
                            selected_cell_freedom += self.taken_stat[k];
                        }
                    }

                    if curr_cell_freedom > selected_cell_freedom {
                        cell_to_fill = (i, j)
                    }
                }
            }
        }

        Some(cell_to_fill)
    }

    // Least Constraining Value
    fn lcv(&self, i: usize, j: usize) -> u8 {
        let mut selected_val_freedom = 0;
        let mut selected_val = 0;

        for k in 0..9 {
            if self.taken_values[i][j] & (1 << k) == 0 && selected_val_freedom > self.taken_stat[k]
            {
                selected_val_freedom = self.taken_stat[k];
                selected_val = k + 1;
            }
        }

        return selected_val as u8;
    }

    fn mark_candidate(&mut self, i: usize, j: usize, n: u8) {
        self.taken_values[i][j] |= 1u16 << (n - 1)
    }
}

pub struct DfsBacktracking;

impl DfsBacktracking {
    pub fn solve(&mut self, s: &mut Sudoku) -> bool {
        let mut dfs_stack = vec![SudokuState::new(s.init)];

        while dfs_stack.is_empty() {
            if let Some(last_state) = dfs_stack.last_mut() {
                if let Some(cell_to_fill) = last_state.mrv() {
                    let candidate = last_state.lcv(cell_to_fill.0, cell_to_fill.1);
                    let mut new_state = SudokuState::new(last_state.grid);
                    new_state.grid[cell_to_fill.0][cell_to_fill.1] = candidate;

                    // TODO: доработать
                    dfs_stack.push(new_state);
                    // last_state.grid[cell_to_fill.0][cell_to_fill.1] =
                } else {
                }
            }
        }

        false
    }
}
