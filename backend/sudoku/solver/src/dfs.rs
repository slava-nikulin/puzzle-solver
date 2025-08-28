use crate::{solver::SolveError, sudoku::Sudoku};

use std::collections::VecDeque;

const N: usize = 9;
const BOX: usize = 3;
const FULL_MASK_N: u16 = (1u16 << N) - 1; // 0b1_1111_1111

type Mask = u16;

#[derive(Clone)]
struct SudokuState {
    // Grid: 0 for empty, 1..=9 for filled value
    grid: [[u8; N]; N],

    // Taken masks per unit
    row_taken: [Mask; N],
    col_taken: [Mask; N],
    box_taken: [Mask; N],

    // Per-cell constraints to block retried candidates at the decision cell if needed
    // Still available for compatibility but not used in the trail solver flow
    cell_constraints: [[Mask; N]; N],

    // Cached per-cell forbidden mask and available count
    forb_mask: [[Mask; N]; N],
    avail_count: [[u8; N]; N],
}

struct PeerStat {
    peers_count: u8,
    peers_domains_sum: u16,
}

// Reversible operations to enable do/undo without cloning the whole state
enum Op {
    SetGrid { row: usize, col: usize, prev: u8 },
    SetRowMask { row: usize, prev: Mask },
    SetColMask { col: usize, prev: Mask },
    SetBoxMask { bix: usize, prev: Mask },
    SetForb { row: usize, col: usize, prev_forb: Mask, prev_count: u8 },
}

// A branching decision: a cell and the remaining candidates to try (bitmask)
struct Decision {
    row: usize,
    col: usize,
    remaining_bits: Mask,
}

impl SudokuState {
    fn new(grid: [[u8; N]; N]) -> Self {
        let mut state = Self {
            grid,
            row_taken: [0; N],
            col_taken: [0; N],
            box_taken: [0; N],
            cell_constraints: [[0; N]; N],
            forb_mask: [[0; N]; N],
            avail_count: [[0; N]; N],
        };

        // Build unit masks from given values
        for r in 0..N {
            for c in 0..N {
                let v = state.grid[r][c];
                if v > 0 {
                    let bit = 1u16 << (v - 1);
                    state.row_taken[r] |= bit;
                    state.col_taken[c] |= bit;
                    state.box_taken[r / BOX * BOX + c / BOX] |= bit;
                }
            }
        }

        // Initialize per-cell caches
        for r in 0..N {
            for c in 0..N {
                let forb = FULL_MASK_N
                    & (state.row_taken[r]
                        | state.col_taken[c]
                        | state.box_taken[r / BOX * BOX + c / BOX]
                        | state.cell_constraints[r][c]);
                state.forb_mask[r][c] = forb;
                if state.grid[r][c] == 0 {
                    state.avail_count[r][c] = (FULL_MASK_N & !forb).count_ones() as u8;
                } else {
                    state.avail_count[r][c] = 0;
                }
            }
        }

        state
    }

    #[inline]
    fn box_index(r: usize, c: usize) -> (usize, usize) {
        (r / BOX * BOX, c / BOX * BOX)
    }

    #[inline]
    fn forbidden_candidates(&self, row: usize, col: usize) -> Mask {
        self.forb_mask[row][col]
    }

    // Propagate all naked singles using cached counts. Trail-aware.
    fn singleton_propagation(&mut self, trail: &mut Vec<Op>) -> Result<(), SolveError> {
        let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

        // Seed with initial naked singles
        for r in 0..N {
            for c in 0..N {
                if self.grid[r][c] == 0 {
                    let cnt = self.avail_count[r][c];
                    if cnt == 0 {
                        return Err(SolveError::Unsolvable);
                    } else if cnt == 1 {
                        queue.push_back((r, c));
                    }
                }
            }
        }

        while let Some((r, c)) = queue.pop_front() {
            if self.grid[r][c] != 0 {
                continue;
            }

            let cnt = self.avail_count[r][c];
            if cnt == 0 {
                return Err(SolveError::Unsolvable);
            }
            if cnt != 1 {
                // Should stay 1, but if not, skip safely
                continue;
            }

            let avail_bits = FULL_MASK_N & !self.forbidden_candidates(r, c);
            let k = (avail_bits.trailing_zeros() + 1) as u8;

            // Assign and update caches via apply_assignment (trail-aware)
            self.apply_assignment(r, c, k, trail)?;

            // Enqueue peers that became singles
            for i in 0..N {
                if i != c && self.grid[r][i] == 0 && self.avail_count[r][i] == 1 {
                    queue.push_back((r, i));
                }
                if i != r && self.grid[i][c] == 0 && self.avail_count[i][c] == 1 {
                    queue.push_back((i, c));
                }
            }
            let (br, bc) = Self::box_index(r, c);
            for dr in 0..BOX {
                for dc in 0..BOX {
                    let rr = br + dr;
                    let cc = bc + dc;
                    if rr != r && cc != c && self.grid[rr][cc] == 0 && self.avail_count[rr][cc] == 1 {
                        queue.push_back((rr, cc));
                    }
                }
            }
        }

        Ok(())
    }

    // Choose the least constraining value heuristic for a given cell
    fn lcv(&self, row: usize, col: usize) -> u8 {
        let avail_bits = FULL_MASK_N & !self.forbidden_candidates(row, col);
        self.lcv_from_bits(row, col, avail_bits)
    }

    fn lcv_from_bits(&self, row: usize, col: usize, mut bits: Mask) -> u8 {
        let mut max_score: u8 = 0;
        let mut val = 0;

        // Precompute unique peers (row, col, box) and their forbids
        let mut peers: [(usize, usize, Mask); 20] = [(0, 0, 0); 20];
        let mut p = 0usize;
        for c in 0..N {
            if c != col && self.grid[row][c] == 0 {
                peers[p] = (row, c, self.forbidden_candidates(row, c));
                p += 1;
            }
        }
        for r in 0..N {
            if r != row && self.grid[r][col] == 0 {
                peers[p] = (r, col, self.forbidden_candidates(r, col));
                p += 1;
            }
        }
        let (br, bc) = Self::box_index(row, col);
        for dr in 0..BOX {
            for dc in 0..BOX {
                let rr = br + dr;
                let cc = bc + dc;
                if rr != row && cc != col && self.grid[rr][cc] == 0 {
                    peers[p] = (rr, cc, self.forbidden_candidates(rr, cc));
                    p += 1;
                }
            }
        }

        while bits != 0 {
            let lsb = bits & (!bits + 1);
            let mut score = 0;
            for idx in 0..p {
                if peers[idx].2 & lsb != 0 {
                    score += 1;
                }
            }
            if score >= max_score {
                max_score = score;
                val = lsb.trailing_zeros() + 1;
            }
            bits ^= lsb;
        }

        val as u8
    }

    fn is_solved(&self) -> bool {
        for r in 0..N {
            for c in 0..N {
                if self.grid[r][c] == 0 {
                    return false;
                }
            }
        }
        true
    }

    fn find_mrv_cell(&self) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize, u8)> = None;
        for r in 0..N {
            for c in 0..N {
                if self.grid[r][c] != 0 {
                    continue;
                }
                let cnt = self.avail_count[r][c];
                if cnt == 0 {
                    return Some((r, c));
                }
                match best {
                    None => best = Some((r, c, cnt)),
                    Some((_, _, bc)) if cnt < bc => best = Some((r, c, cnt)),
                    _ => {}
                }
            }
        }
        best.map(|(r, c, _)| (r, c))
    }

    // Apply an assignment and record all changes on the trail for undo
    fn apply_assignment(
        &mut self,
        row: usize,
        col: usize,
        val: u8,
        trail: &mut Vec<Op>,
    ) -> Result<(), SolveError> {
        // Set grid value
        trail.push(Op::SetGrid { row, col, prev: self.grid[row][col] });
        self.grid[row][col] = val;

        let bit = 1u16 << (val - 1);

        // Update row/col/box masks (reversible)
        trail.push(Op::SetRowMask { row, prev: self.row_taken[row] });
        trail.push(Op::SetColMask { col, prev: self.col_taken[col] });
        let bix = row / BOX * BOX + col / BOX;
        trail.push(Op::SetBoxMask { bix, prev: self.box_taken[bix] });
        self.row_taken[row] |= bit;
        self.col_taken[col] |= bit;
        self.box_taken[bix] |= bit;

        // Update peers: forbid this bit and decrement avail_count
        // Row peers
        for c in 0..N {
            if c == col || self.grid[row][c] != 0 {
                continue;
            }
            let prev_forb = self.forb_mask[row][c];
            let prev_cnt = self.avail_count[row][c];
            if prev_forb & bit == 0 {
                trail.push(Op::SetForb { row, col: c, prev_forb, prev_count: prev_cnt });
                self.forb_mask[row][c] = prev_forb | bit;
                if prev_cnt == 0 {
                    return Err(SolveError::Unsolvable);
                }
                self.avail_count[row][c] = prev_cnt - 1;
                if self.avail_count[row][c] == 0 {
                    return Err(SolveError::Unsolvable);
                }
            }
        }
        // Column peers
        for r in 0..N {
            if r == row || self.grid[r][col] != 0 {
                continue;
            }
            let prev_forb = self.forb_mask[r][col];
            let prev_cnt = self.avail_count[r][col];
            if prev_forb & bit == 0 {
                trail.push(Op::SetForb { row: r, col, prev_forb, prev_count: prev_cnt });
                self.forb_mask[r][col] = prev_forb | bit;
                if prev_cnt == 0 {
                    return Err(SolveError::Unsolvable);
                }
                self.avail_count[r][col] = prev_cnt - 1;
                if self.avail_count[r][col] == 0 {
                    return Err(SolveError::Unsolvable);
                }
            }
        }
        // Box peers (exclude same row/col)
        let (br, bc) = Self::box_index(row, col);
        for dr in 0..BOX {
            for dc in 0..BOX {
                let rr = br + dr;
                let cc = bc + dc;
                if rr == row || cc == col || self.grid[rr][cc] != 0 {
                    continue;
                }
                let prev_forb = self.forb_mask[rr][cc];
                let prev_cnt = self.avail_count[rr][cc];
                if prev_forb & bit == 0 {
                    trail.push(Op::SetForb { row: rr, col: cc, prev_forb, prev_count: prev_cnt });
                    self.forb_mask[rr][cc] = prev_forb | bit;
                    if prev_cnt == 0 {
                        return Err(SolveError::Unsolvable);
                    }
                    self.avail_count[rr][cc] = prev_cnt - 1;
                    if self.avail_count[rr][cc] == 0 {
                        return Err(SolveError::Unsolvable);
                    }
                }
            }
        }

        Ok(())
    }

    // Undo a single operation
    fn undo(&mut self, op: Op) {
        match op {
            Op::SetGrid { row, col, prev } => self.grid[row][col] = prev,
            Op::SetRowMask { row, prev } => self.row_taken[row] = prev,
            Op::SetColMask { col, prev } => self.col_taken[col] = prev,
            Op::SetBoxMask { bix, prev } => self.box_taken[bix] = prev,
            Op::SetForb { row, col, prev_forb, prev_count } => {
                self.forb_mask[row][col] = prev_forb;
                self.avail_count[row][col] = prev_count;
            }
        }
    }

    // Supplemental: peer stats if you still want MRV tie-break (not used in trail solver, kept for completeness)
    fn peers_stat(&self, row: usize, col: usize) -> PeerStat {
        let mut res = PeerStat { peers_count: 0, peers_domains_sum: 0 };
        for i in 0..N {
            if i != row && self.grid[i][col] == 0 {
                res.peers_count += 1;
                res.peers_domains_sum += self.avail_count[i][col] as u16;
            }
            if i != col && self.grid[row][i] == 0 {
                res.peers_count += 1;
                res.peers_domains_sum += self.avail_count[row][i] as u16;
            }
        }
        let (br, bc) = Self::box_index(row, col);
        for dr in 0..BOX {
            for dc in 0..BOX {
                let rr = br + dr;
                let cc = bc + dc;
                if rr != row && cc != col && self.grid[rr][cc] == 0 {
                    res.peers_count += 1;
                    res.peers_domains_sum += self.avail_count[rr][cc] as u16;
                }
            }
        }
        res
    }
}

pub struct DfsBacktracking;

impl DfsBacktracking {
    pub fn solve(&mut self, s: &mut Sudoku) -> Result<(), SolveError> {
        let mut state = SudokuState::new(s.init);
        let mut trail: Vec<Op> = Vec::with_capacity(4096);
        let mut levels: Vec<usize> = Vec::with_capacity(128);
        let mut decisions: Vec<Decision> = Vec::with_capacity(128);

        'outer: loop {
            // Propagate all forced singles
            if let Err(_) = state.singleton_propagation(&mut trail) {
                // Conflict: backtrack
                loop {
                    let Some(mut dec) = decisions.pop() else { return Err(SolveError::Unsolvable) };
                    let mark = levels.pop().expect("decision level underflow");
                    while trail.len() > mark {
                        let op = trail.pop().unwrap();
                        state.undo(op);
                    }
                    if dec.remaining_bits == 0 {
                        continue; // backtrack further
                    }
                    let choice = state.lcv_from_bits(dec.row, dec.col, dec.remaining_bits);
                    dec.remaining_bits &= !(1u16 << (choice - 1));
                    decisions.push(dec);
                    levels.push(trail.len());
                    if let Err(_) = state.apply_assignment(dec.row, dec.col, choice, &mut trail) {
                        continue; // immediate conflict, continue backtracking
                    }
                    continue 'outer;
                }
            }

            // Solved?
            if state.is_solved() {
                s.solution = state.grid;
                return Ok(());
            }

            // Choose MRV cell and branch
            let Some((row, col)) = state.find_mrv_cell() else { s.solution = state.grid; return Ok(()); };
            let avail_bits = FULL_MASK_N & !state.forbidden_candidates(row, col);
            if avail_bits == 0 {
                // Shouldn't happen after propagation; treat as conflict and backtrack
                loop {
                    let Some(mut dec) = decisions.pop() else { return Err(SolveError::Unsolvable) };
                    let mark = levels.pop().expect("decision level underflow");
                    while trail.len() > mark {
                        let op = trail.pop().unwrap();
                        state.undo(op);
                    }
                    if dec.remaining_bits == 0 { continue; }
                    let choice = state.lcv_from_bits(dec.row, dec.col, dec.remaining_bits);
                    dec.remaining_bits &= !(1u16 << (choice - 1));
                    decisions.push(dec);
                    levels.push(trail.len());
                    if let Err(_) = state.apply_assignment(dec.row, dec.col, choice, &mut trail) {
                        continue; // immediate conflict
                    }
                    continue 'outer;
                }
            }

            // Branch: pick LCV and remember remaining candidates
            let choice = state.lcv_from_bits(row, col, avail_bits);
            let remaining = avail_bits & !(1u16 << (choice - 1));
            decisions.push(Decision { row, col, remaining_bits: remaining });
            levels.push(trail.len());

            if let Err(_) = state.apply_assignment(row, col, choice, &mut trail) {
                // Conflict; will be handled by backtracking in next iteration
                continue;
            }
        }
    }
}
