use std::collections::VecDeque;

use crate::{solver::SolveError, sudoku::Sudoku};

const N: usize = 9;
const FULL_MASK_N: u16 = (1u16 << N) - 1; // Set N bits
const BOX: usize = 3;

struct SudokuState {
    grid: [[u8; N]; N],  // grid[r][c] is 0 for empty or 1..=9 for value
    row_taken: [u16; N], //row_taken/col_taken/box_taken have bit k-1 set if k is used
    col_taken: [u16; N],
    box_taken: [u16; N],
    cell_constraints: [[u16; N]; N], // cell_constraints[r][c] forbids candidate bits that have been backtracked already for that specific cell and state.
    choice_stack: Vec<RootChoice>,
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
            choice_stack: Vec::<RootChoice>::new(),
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

    fn box_index(i: usize, j: usize) -> (usize, usize) {
        (i / BOX * BOX, j / BOX * BOX)
    }

    fn forbidden_candidates(&self, row: usize, col: usize) -> u16 {
        let constraints = self
            .choice_stack
            .last()
            .map_or(self.cell_constraints[row][col], |current_choice| {
                current_choice.cell_constraints[row][col]
            });

        FULL_MASK_N
            & (self.box_taken[row / BOX * BOX + col / BOX]
                | self.col_taken[col]
                | self.row_taken[row]
                | constraints)
    }

    fn mark_taken(&mut self, row: usize, col: usize, val: u8) {
        let mark_mask = 1u16 << (val - 1);
        self.box_taken[row / BOX * BOX + col / BOX] |= mark_mask;
        self.col_taken[col] |= mark_mask;
        self.row_taken[row] |= mark_mask;
    }

    fn choose_single(&mut self, row: usize, col: usize, val: u8) {
        self.mark_taken(row, col, val);
        self.grid[row][col] = val;
        // for non-root singles
        if let Some(current_choice) = self.choice_stack.last_mut() {
            current_choice.singles.push(Choice { row, col, val });
        }
    }

    fn unmark_taken(&mut self, row: usize, col: usize, val: u8) {
        let umark_mask = !(1u16 << (val - 1));
        self.box_taken[row / BOX * BOX + col / BOX] &= umark_mask;
        self.col_taken[col] &= umark_mask;
        self.row_taken[row] &= umark_mask;
    }

    fn make_choice(&mut self, row: usize, col: usize, val: u8) {
        self.mark_taken(row, col, val);
        self.grid[row][col] = val;
        self.choice_stack.push(RootChoice {
            choice: Choice { row, col, val },
            singles: Vec::<Choice>::new(),
            cell_constraints: [[0; N]; N],
        });
    }

    fn undo_last_choice(&mut self) -> Option<()> {
        if let Some(wrong_root) = self.choice_stack.pop() {
            self.grid[wrong_root.choice.row][wrong_root.choice.col] = 0;
            self.unmark_taken(
                wrong_root.choice.row,
                wrong_root.choice.col,
                wrong_root.choice.val,
            );

            for single in wrong_root.singles.iter() {
                self.grid[single.row][single.col] = 0;
                self.unmark_taken(single.row, single.col, single.val);
            }

            if let Some(prev_root) = self.choice_stack.last_mut() {
                prev_root.cell_constraints[wrong_root.choice.row][wrong_root.choice.col] |=
                    1u16 << (wrong_root.choice.val - 1);
            } else {
                self.cell_constraints[wrong_root.choice.row][wrong_root.choice.col] |=
                    1u16 << (wrong_root.choice.val - 1);
            }
            Some(())
        } else {
            None
        }
    }

    /// Propagates singleton candidates (cells with only one possible value).
    /// Returns Err(SolveError::Unsolvable) if any cell has no candidates.
    fn singleton_propagation(&mut self) -> Result<(), SolveError> {
        // Preallocate worklist to reduce reallocations (max 81 cells)
        let mut queue: VecDeque<(usize, usize)> = VecDeque::with_capacity(N * N);

        // Seed with initial naked singles
        for r in 0..N {
            for c in 0..N {
                if self.grid[r][c] == 0 {
                    let taken = self.forbidden_candidates(r, c);
                    let avail_count = (FULL_MASK_N & !taken).count_ones() as u8;
                    if avail_count == 0 {
                        return Err(SolveError::Unsolvable);
                    } else if avail_count == 1 {
                        queue.push_back((r, c));
                    }
                }
            }
        }

        // Process the worklist
        while let Some((r, c)) = queue.pop_front() {
            if self.grid[r][c] != 0 {
                continue; // how?
            }
            let mut domain_bits = FULL_MASK_N & !self.forbidden_candidates(r, c);
            if domain_bits == 0 {
                return Err(SolveError::Unsolvable); // how?
            }
            let avail_count = domain_bits.count_ones() as u8;
            if avail_count != 1 {
                continue; // how?
            }

            let k = (domain_bits.trailing_zeros() + 1) as u8;
            self.choose_single(r, c, k);

            // Enqueue peers that became singles
            for i in 0..N {
                if i != r && self.grid[i][c] == 0 {
                    domain_bits = FULL_MASK_N & !self.forbidden_candidates(i, c);
                    if domain_bits.count_ones() == 1 {
                        queue.push_back((i, c));
                    }
                }
                if i != c && self.grid[r][i] == 0 {
                    domain_bits = FULL_MASK_N & !self.forbidden_candidates(r, i);
                    if domain_bits.count_ones() == 1 {
                        queue.push_back((r, i));
                    }
                }
            }
            // Box (exclude row/col to avoid duplicates)
            let (br, bc) = Self::box_index(r, c);
            for dr in 0..BOX {
                for dc in 0..BOX {
                    let rr = br + dr;
                    let cc = bc + dc;
                    if (rr != r) && (cc != c) && self.grid[rr][cc] == 0 {
                        domain_bits = FULL_MASK_N & !self.forbidden_candidates(rr, cc);
                        if domain_bits.count_ones() == 1 {
                            queue.push_back((rr, cc));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Finds the cell with minimum remaining values (MRV).
    /// Returns Ok(MrvRes::Cell(i, j))) for a cell to fill, or Ok(MrvRes::Solved) if solved, or Err(SolveError::Unsolvable).
    fn mrv(&mut self) -> Result<MrvRes, SolveError> {
        self.singleton_propagation()?;

        let mut target_cell: (usize, usize) = (0, 0);
        let mut target_cell_domains_count = u8::MAX;

        for row in 0..N {
            for col in 0..N {
                if self.grid[row][col] > 0 {
                    continue;
                }
                let current_cell_domain_bits = FULL_MASK_N & !self.forbidden_candidates(row, col);
                let current_cell_domains_count = current_cell_domain_bits.count_ones() as u8;
                if current_cell_domains_count < target_cell_domains_count {
                    target_cell_domains_count = current_cell_domains_count;
                    target_cell = (row, col);
                } else if target_cell_domains_count == current_cell_domains_count {
                    // the idea of this tie break is to cut of maximum wrong choice branches
                    let current_cell_peers_stat = self.peers_stat(row, col);
                    let target_cell_peers_stat = self.peers_stat(target_cell.0, target_cell.1);
                    if target_cell_peers_stat.peers_count < current_cell_peers_stat.peers_count
                        || (target_cell_peers_stat.peers_count
                            == current_cell_peers_stat.peers_count
                            && target_cell_peers_stat.peers_domains_sum
                                > current_cell_peers_stat.peers_domains_sum)
                    {
                        target_cell = (row, col);
                    }
                }
            }
        }

        if target_cell_domains_count == u8::MAX {
            return Ok(MrvRes::Solved);
        }

        Ok(MrvRes::Cell(target_cell.0, target_cell.1))
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
                let domain_bits = FULL_MASK_N & !self.forbidden_candidates(i, col);
                res.peers_count += 1;
                res.peers_domains_sum += domain_bits.count_ones() as u16;
            }

            if i != col && self.grid[row][i] == 0 {
                let domain_bits = FULL_MASK_N & !self.forbidden_candidates(row, i);
                res.peers_count += 1;
                res.peers_domains_sum += domain_bits.count_ones() as u16;
            }
        }

        let (box_row_start, box_col_start) = Self::box_index(row, col);
        for b_row in 0..BOX {
            for b_col in 0..BOX {
                if box_row_start + b_row != row
                    && box_col_start + b_col != col
                    && self.grid[box_row_start + b_row][box_col_start + b_col] == 0
                {
                    let domain_bits = FULL_MASK_N
                        & !self.forbidden_candidates(box_row_start + b_row, box_col_start + b_col);
                    res.peers_count += 1;
                    res.peers_domains_sum += domain_bits.count_ones() as u16;
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

struct RootChoice {
    choice: Choice,
    singles: Vec<Choice>,
    cell_constraints: [[u16; N]; N],
}

struct Choice {
    row: usize,
    col: usize,
    val: u8,
}
pub struct DfsBacktracking;

impl DfsBacktracking {
    pub fn solve(&mut self, s: &mut Sudoku) -> Result<(), SolveError> {
        let mut sudoku_state = SudokuState::new(s.init);

        loop {
            match sudoku_state.mrv() {
                Ok(res) => match res {
                    MrvRes::Cell(row, col) => {
                        let val = sudoku_state.lcv(row, col);
                        sudoku_state.make_choice(row, col, val);
                    }
                    MrvRes::Solved => {
                        s.solution = sudoku_state.grid;
                        return Ok(());
                    }
                },
                Err(_) => {
                    if sudoku_state.undo_last_choice().is_none() {
                        return Err(SolveError::Unsolvable);
                    }
                }
            }
        }
    }
}
