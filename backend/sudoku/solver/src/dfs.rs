use crate::{solver::SolveError, sudoku::Sudoku};

const N: usize = 9;
const FULL_MASK_N: u16 = (1u16 << N) - 1;
const BOX: usize = 3;

#[derive(Clone)]
struct SudokuState {
    grid: [[u8; N]; N], // grid[r][c] is 0 for empty or 1..=9 for value
    last_edit_pos: (usize, usize),
    last_edit_val: u8,
    row_taken: [u16; N], //row_taken/col_taken/box_taken have bit k-1 set if k is used
    col_taken: [u16; N],
    box_taken: [u16; N],
    cell_constraints: [[u16; N]; N], // cell_constraints[r][c] forbids candidate bits that have been backtracked already for that specific cell and state.
}

struct PeerStat {
    peers_count: u8,
    peers_domains_sum: u16,
}

enum MrvRes {
    Solved,
    Cell(usize, usize),
}

impl SudokuState {
    fn new(grid: [[u8; N]; N]) -> Self {
        let mut state = Self {
            grid,
            last_edit_pos: (0, 0),
            last_edit_val: 0,
            row_taken: [0; N],
            col_taken: [0; N],
            box_taken: [0; N],
            cell_constraints: [[0; N]; N],
        };

        for row in 0..N {
            for col in 0..N {
                if state.grid[row][col] > 0 {
                    let taken_bit = 1u16 << (state.grid[row][col] - 1);
                    state.col_taken[col] |= taken_bit;
                    state.row_taken[row] |= taken_bit;
                    state.box_taken[row / BOX * BOX + col / BOX] |= taken_bit;
                }
            }
        }

        state
    }

    fn assign(&self, row: usize, col: usize, candidate: u8) -> Self {
        let mut new_state = self.clone();
        new_state.last_edit_pos = (row, col);
        new_state.last_edit_val = candidate;
        new_state.grid[row][col] = candidate;
        new_state.mark_taken(row, col, candidate);
        new_state
    }

    fn available_candidate_count(forb_cand_bits: u16) -> u8 {
        (FULL_MASK_N & !forb_cand_bits).count_ones() as u8
    }

    fn box_index(i: usize, j: usize) -> (usize, usize) {
        (i / BOX * BOX, j / BOX * BOX)
    }

    fn forbidden_candidates(&self, row: usize, col: usize) -> u16 {
        FULL_MASK_N
            & (self.box_taken[row / BOX * BOX + col / BOX]
                | self.col_taken[col]
                | self.row_taken[row]
                | self.cell_constraints[row][col])
    }

    fn mark_taken(&mut self, row: usize, col: usize, n: u8) {
        self.box_taken[row / BOX * BOX + col / BOX] |= 1u16 << (n - 1);
        self.col_taken[col] |= 1u16 << (n - 1);
        self.row_taken[row] |= 1u16 << (n - 1);
    }

    /// Propagates singleton candidates (cells with only one possible value).
    /// Returns Err(SolveError::Unsolvable) if any cell has no candidates.
    fn singleton_propagation(&mut self) -> Result<(), SolveError> {
        //TODO: hidden singles??
        loop {
            let mut changed = false;
            for row in 0..N {
                for col in 0..N {
                    if self.grid[row][col] == 0 {
                        let taken = self.forbidden_candidates(row, col);
                        let avail_count = Self::available_candidate_count(taken);
                        if avail_count == 0 {
                            return Err(SolveError::Unsolvable);
                        } else if avail_count == 1 {
                            let avail_bits = FULL_MASK_N & !taken;
                            let k = (avail_bits.trailing_zeros() + 1) as u8;
                            self.grid[row][col] = k;
                            self.mark_taken(row, col, k);
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        Ok(())
    }

    /// Finds the cell with minimum remaining values (MRV).
    /// Returns Ok(MrvRes::Cell(i, j))) for a cell to fill, or Ok(MrvRes::Solved) if solved, or Err(SolveError::Unsolvable).
    fn mrv(&mut self) -> Result<MrvRes, SolveError> {
        self.singleton_propagation()?;

        let mut cell_to_fill: (usize, usize) = (0, 0);
        let mut min_avail_count = u8::MAX;
        let mut best_peers_stat = PeerStat {
            peers_count: 0u8,
            peers_domains_sum: u16::MAX,
        };

        for row in 0..N {
            for col in 0..N {
                if self.grid[row][col] > 0 {
                    continue;
                }
                let available_count =
                    Self::available_candidate_count(self.forbidden_candidates(row, col));
                if available_count < min_avail_count {
                    min_avail_count = available_count;
                    cell_to_fill = (row, col);
                    best_peers_stat = self.peers_stat(row, col);
                } else if min_avail_count == available_count {
                    let candidate_stat_target = self.peers_stat(row, col);
                    if best_peers_stat.peers_count < candidate_stat_target.peers_count
                        || (best_peers_stat.peers_count == candidate_stat_target.peers_count
                            && best_peers_stat.peers_domains_sum
                                > candidate_stat_target.peers_domains_sum)
                    {
                        cell_to_fill = (row, col);
                    }
                }
            }
        }

        if min_avail_count == u8::MAX {
            return Ok(MrvRes::Solved);
        }

        Ok(MrvRes::Cell(cell_to_fill.0, cell_to_fill.1))
    }

    // peers_count - the more empty neighbors, the sooner dead-end branches will be cut off
    // sum_peers_domains - the fewer domains peers have, the higher the chance that substitution will remove dead-end branches from other zero cells
    fn peers_stat(&self, row: usize, col: usize) -> PeerStat {
        let mut res = PeerStat {
            peers_count: 0,
            peers_domains_sum: 0,
        };

        for i in 0..N {
            if i != row && self.grid[i][col] == 0 {
                res.peers_count += 1;
                res.peers_domains_sum +=
                    Self::available_candidate_count(self.forbidden_candidates(i, col)) as u16;
            }

            if i != col && self.grid[row][i] == 0 {
                res.peers_count += 1;
                res.peers_domains_sum +=
                    Self::available_candidate_count(self.forbidden_candidates(row, i)) as u16;
            }
        }

        let (box_row_start, box_col_start) = Self::box_index(row, col);
        for b_row in 0..BOX {
            for b_col in 0..BOX {
                if box_row_start + b_row != row
                    && box_col_start + b_col != col
                    && self.grid[box_row_start + b_row][box_col_start + b_col] == 0
                {
                    res.peers_count += 1;
                    res.peers_domains_sum += Self::available_candidate_count(
                        self.forbidden_candidates(box_row_start + b_row, box_col_start + b_col),
                    ) as u16;
                }
            }
        }

        res
    }

    // Least Constraining Value
    // select the candidate who is least likely affects the peers - this way we reduce the probability of a dead-end
    fn lcv(&mut self, row: usize, col: usize) -> u8 {
        let mut max_score: u8 = 0;
        let mut val = 0;

        let mut avail_can_bits = FULL_MASK_N & !self.forbidden_candidates(row, col);
        while avail_can_bits > 0 {
            let lsb = avail_can_bits & (!avail_can_bits + 1);
            let mut score = 0;
            for l in 0..N {
                if l != col
                    && self.grid[row][l] == 0
                    && self.forbidden_candidates(row, l) & lsb != 0
                {
                    score += 1;
                }
                if l != row
                    && self.grid[l][col] == 0
                    && self.forbidden_candidates(l, col) & lsb != 0
                {
                    score += 1;
                }
            }

            let (box_row_start, box_col_start) = Self::box_index(row, col);
            for b_row in 0..BOX {
                for b_col in 0..BOX {
                    if box_row_start + b_row != row
                        && box_col_start + b_col != col
                        && self.grid[box_row_start + b_row][box_col_start + b_col] == 0
                        && self.forbidden_candidates(box_row_start + b_row, box_col_start + b_col)
                            & lsb
                            != 0
                    {
                        score += 1;
                    }
                }
            }

            if score >= max_score {
                max_score = score;
                val = lsb.trailing_zeros() + 1;
            }

            avail_can_bits ^= lsb; // remove lsb
        }

        val as u8
    }
}

pub struct DfsBacktracking;

impl DfsBacktracking {
    pub fn solve(&mut self, s: &mut Sudoku) -> Result<(), SolveError> {
        let mut dfs_stack: Vec<SudokuState> = Vec::with_capacity(N * N);
        dfs_stack.push(SudokuState::new(s.init));

        while !dfs_stack.is_empty() {
            if let Some(last_state) = dfs_stack.last_mut() {
                match last_state.mrv() {
                    Err(err) => {
                        let incorrect_state = {
                            let this = dfs_stack.pop();
                            match this {
                                Some(val) => val,
                                None => return Err(err),
                            }
                        };

                        if let Some(last_correct_state) = dfs_stack.last_mut() {
                            last_correct_state.cell_constraints[incorrect_state.last_edit_pos.0]
                                [incorrect_state.last_edit_pos.1] |=
                                1u16 << (incorrect_state.last_edit_val - 1)
                        } else {
                            return Err(err);
                        }
                    }
                    Ok(res) => match res {
                        MrvRes::Cell(i, j) => {
                            let candidate = last_state.lcv(i, j);
                            let new_state = last_state.assign(i, j, candidate);
                            dfs_stack.push(new_state);
                        }
                        MrvRes::Solved => {
                            s.solution = last_state.grid;
                            return Ok(());
                        }
                    },
                }
            }
        }

        Err(SolveError::Unsolvable)
    }
}
