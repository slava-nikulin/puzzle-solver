use crate::{solver::SolveError, sudoku::Sudoku};
use std::collections::VecDeque;

struct Constraints<const N: usize, const BR: usize, const BC: usize> {
    row: [u16; N],
    col: [u16; N],
    s_box: [u16; N],
    backtrack: [[u16; N]; N],
}

impl<const N: usize, const BR: usize, const BC: usize> Constraints<N, BR, BC> {
    const FULL_MASK_N: u16 = (1u16 << N) - 1; // Set N bits

    fn fork(&self) -> Constraints<N, BR, BC> {
        Constraints {
            row: self.row,
            col: self.col,
            s_box: self.s_box,
            backtrack: [[0; N]; N],
        }
    }

    fn mark_taken(&mut self, row: usize, col: usize, val: u8) {
        let mark_mask = 1u16 << (val - 1);
        self.s_box[Sudoku::<N, BR, BC>::box_index(row, col)] |= mark_mask;
        self.col[col] |= mark_mask;
        self.row[row] |= mark_mask;
    }

    fn forbid_cell_val(&mut self, row: usize, col: usize, val: u8) {
        self.backtrack[row][col] |= 1u16 << (val - 1);
    }

    fn forbidden_candidates(&self, row: usize, col: usize) -> u16 {
        Self::FULL_MASK_N
            & (self.s_box[Sudoku::<N, BR, BC>::box_index(row, col)]
                | self.col[col]
                | self.row[row]
                | self.backtrack[row][col])
    }

    fn available_candidates(&self, row: usize, col: usize) -> u16 {
        Self::FULL_MASK_N & !self.forbidden_candidates(row, col)
    }
}

struct ChoosenVal {
    row: usize,
    col: usize,
    val: u8,
}
struct PeerStat {
    peers_count: u8,
    peers_domains_sum: u16,
}

enum MrvRes {
    Solved,
    Cell(usize, usize),
}

struct DfsNode<const N: usize, const BR: usize, const BC: usize> {
    grid: [[u8; N]; N],
    choice: Option<ChoosenVal>,
    constraints: Constraints<N, BR, BC>,
}

impl<const N: usize, const BR: usize, const BC: usize> DfsNode<N, BR, BC> {
    fn new(grid: [[u8; N]; N]) -> Self {
        let mut state = Self {
            grid,
            constraints: Constraints {
                row: [0; N],
                col: [0; N],
                s_box: [0; N],
                backtrack: [[0; N]; N],
            },
            choice: None,
        };

        for row in 0..N {
            for col in 0..N {
                if state.grid[row][col] > 0 {
                    let taken_bit = 1u16 << (state.grid[row][col] - 1);
                    state.constraints.col[col] |= taken_bit;
                    state.constraints.row[row] |= taken_bit;
                    state.constraints.s_box[Sudoku::<N, BR, BC>::box_index(row, col)] |= taken_bit;
                }
            }
        }

        state
    }

    fn fork(&mut self, row: usize, col: usize, val: u8) -> Self {
        let mut node = DfsNode {
            grid: self.grid,
            choice: Some(ChoosenVal { row, col, val }),
            constraints: Constraints::fork(&self.constraints),
        };
        node.constraints.mark_taken(row, col, val);
        node.grid[row][col] = val;

        node
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
                    // let taken = self.constraints.forbidden_candidates(r, c);
                    let avail_count =
                        self.constraints.available_candidates(r, c).count_ones() as u8;
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
                continue;
            }
            let domain_bits = self.constraints.available_candidates(r, c);
            if domain_bits.count_ones() != 1 {
                continue;
            }

            let k = (domain_bits.trailing_zeros() + 1) as u8;
            self.grid[r][c] = k;
            self.constraints.mark_taken(r, c, k);

            // Enqueue peers that became singles
            for i in 0..N {
                if i != r && self.grid[i][c] == 0 {
                    // domain_bits = FULL_MASK_N & !self.constraints.forbidden_candidates(i, c);
                    if self.constraints.available_candidates(i, c).count_ones() == 1 {
                        queue.push_back((i, c));
                    }
                }
                if i != c && self.grid[r][i] == 0 {
                    // domain_bits = FULL_MASK_N & !self.constraints.forbidden_candidates(r, i);
                    if self.constraints.available_candidates(r, i).count_ones() == 1 {
                        queue.push_back((r, i));
                    }
                }
            }
            // Box (exclude row/col to avoid duplicates)
            let (br, bc) = Sudoku::<N, BR, BC>::box_coord(r, c);
            for dr in 0..BR {
                for dc in 0..BC {
                    let rr = br + dr;
                    let cc = bc + dc;
                    if (rr != r) && (cc != c) && self.grid[rr][cc] == 0 {
                        // domain_bits = FULL_MASK_N & !self.constraints.forbidden_candidates(rr, cc);
                        if self.constraints.available_candidates(rr, cc).count_ones() == 1 {
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
        let mut target_cell_peers_stat = PeerStat {
            peers_count: 0,
            peers_domains_sum: 0,
        };

        for row in 0..N {
            for col in 0..N {
                if self.grid[row][col] > 0 {
                    continue;
                }
                // let current_cell_domain_bits =
                // FULL_MASK_N & !self.constraints.forbidden_candidates(row, col);
                let current_cell_domains_count =
                    self.constraints.available_candidates(row, col).count_ones() as u8;
                if current_cell_domains_count < target_cell_domains_count {
                    target_cell_domains_count = current_cell_domains_count;
                    target_cell = (row, col);
                    target_cell_peers_stat = self.peers_stat(target_cell.0, target_cell.1);
                } else if target_cell_domains_count == current_cell_domains_count {
                    // the idea of this tie break is to cut of maximum wrong choice branches
                    let current_cell_peers_stat = self.peers_stat(row, col);
                    if target_cell_peers_stat.peers_count < current_cell_peers_stat.peers_count
                        || (target_cell_peers_stat.peers_count
                            == current_cell_peers_stat.peers_count
                            && target_cell_peers_stat.peers_domains_sum
                                > current_cell_peers_stat.peers_domains_sum)
                    {
                        target_cell = (row, col);
                        target_cell_peers_stat = current_cell_peers_stat;
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
    fn peers_stat(&mut self, row: usize, col: usize) -> PeerStat {
        let mut res = PeerStat {
            peers_count: 0,
            peers_domains_sum: 0,
        };

        for i in 0..N {
            if i != row && self.grid[i][col] == 0 {
                // let domain_bits = FULL_MASK_N & !self.constraints.forbidden_candidates(i, col);
                res.peers_count += 1;
                res.peers_domains_sum +=
                    self.constraints.available_candidates(i, col).count_ones() as u16;
            }

            if i != col && self.grid[row][i] == 0 {
                // let domain_bits = FULL_MASK_N & !self.constraints.forbidden_candidates(row, i);
                res.peers_count += 1;
                res.peers_domains_sum +=
                    self.constraints.available_candidates(row, i).count_ones() as u16;
            }
        }

        let (box_row_start, box_col_start) = Sudoku::<N, BR, BC>::box_coord(row, col);
        for b_row in 0..BR {
            for b_col in 0..BC {
                if box_row_start + b_row != row
                    && box_col_start + b_col != col
                    && self.grid[box_row_start + b_row][box_col_start + b_col] == 0
                {
                    // let domain_bits = FULL_MASK_N
                    // & !self
                    // .constraints
                    // .forbidden_candidates(box_row_start + b_row, box_col_start + b_col);

                    res.peers_count += 1;
                    res.peers_domains_sum += self
                        .constraints
                        .available_candidates(box_row_start + b_row, box_col_start + b_col)
                        .count_ones() as u16;
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

        let mut avail_can_bits = self.constraints.available_candidates(row, col);
        while avail_can_bits > 0 {
            let lsb = avail_can_bits & (!avail_can_bits + 1);
            let mut score = 0;
            for l in 0..N {
                if l != col
                    && self.grid[row][l] == 0
                    && self.constraints.forbidden_candidates(row, l) & lsb != 0
                {
                    score += 1;
                }
                if l != row
                    && self.grid[l][col] == 0
                    && self.constraints.forbidden_candidates(l, col) & lsb != 0
                {
                    score += 1;
                }
            }

            let (box_row_start, box_col_start) = Sudoku::<N, BR, BC>::box_coord(row, col);
            for b_row in 0..BR {
                for b_col in 0..BC {
                    if box_row_start + b_row != row
                        && box_col_start + b_col != col
                        && self.grid[box_row_start + b_row][box_col_start + b_col] == 0
                        && self
                            .constraints
                            .forbidden_candidates(box_row_start + b_row, box_col_start + b_col)
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

pub struct DfsBacktracking<const N: usize, const BR: usize, const BC: usize>;

impl<const N: usize, const BR: usize, const BC: usize> DfsBacktracking<N, BR, BC> {
    pub fn solve(&mut self, s: &mut Sudoku<N, BR, BC>) -> Result<(), SolveError> {
        let mut dfs_stack = vec![DfsNode::<N, BR, BC>::new(s.init)];
        loop {
            let top_node = dfs_stack.last_mut().ok_or(SolveError::Unsolvable)?;
            let new_node = match top_node.mrv() {
                Ok(mrv) => match mrv {
                    MrvRes::Cell(row, col) => {
                        let lcv = top_node.lcv(row, col);
                        top_node.fork(row, col, lcv)
                    }
                    MrvRes::Solved => {
                        s.solution = top_node.grid;
                        return Ok(());
                    }
                },
                Err(_) => {
                    let wrong_node = dfs_stack
                        .pop()
                        .ok_or(SolveError::Unsolvable)?
                        .choice
                        .ok_or(SolveError::Unsolvable)?;
                    let top_node = dfs_stack.last_mut().ok_or(SolveError::Unsolvable)?;
                    top_node.constraints.forbid_cell_val(
                        wrong_node.row,
                        wrong_node.col,
                        wrong_node.val,
                    );
                    continue;
                }
            };
            dfs_stack.push(new_node);
        }
    }
}
