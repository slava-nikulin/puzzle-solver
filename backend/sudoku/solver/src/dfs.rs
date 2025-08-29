use crate::{solver::SolveError, sudoku::Sudoku};

use std::collections::VecDeque;

const N: usize = 9;
const BOX: usize = 3;
const FULL_MASK_N: u16 = (1u16 << N) - 1; // 0b1_1111_1111

type Mask = u16;

struct SudokuState {
    // Grid: 0 for empty, 1..=9 for filled value
    grid: [[u8; N]; N],
    // Taken masks per unit
    row_taken: [Mask; N],
    col_taken: [Mask; N],
    box_taken: [Mask; N],
    // Optional extra constraints per cell (kept for compatibility)
    // cell_constraints: [[Mask; N]; N],
    // Cached per-cell forbidden mask and available count
    forb_mask: [[Mask; N]; N],
    avail_count: [[u8; N]; N],
}

/// A single backtracking candidate to forbid upon backtrack
struct Candidate {
    row: usize,
    col: usize,
    bit: Mask,   // single bit for the candidate (1 << (val-1))
    mark: usize, // trail length at branch time
}

struct PeerStat {
    peers_count: u8,
    peers_domains_sum: u16,
}

// Reversible operations to enable do/undo without cloning the whole state
/// Reversible operations to enable do/undo without cloning the whole state
enum Op {
    SetGrid {
        row: usize,
        col: usize,
        prev: u8,
    },
    SetRowMask {
        row: usize,
        prev: Mask,
    },
    SetColMask {
        col: usize,
        prev: Mask,
    },
    SetBoxMask {
        bix: usize,
        prev: Mask,
    },
    SetForb {
        row: usize,
        col: usize,
        prev_forb: Mask,
        prev_count: u8,
    },
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
                        | state.box_taken[r / BOX * BOX + c / BOX]);
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
                    // leave detection of conflicts to propagation/assignment
                    continue;
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

    /// Apply an assignment and record all changes on the trail for undo
    fn apply_assignment(
        &mut self,
        row: usize,
        col: usize,
        val: u8,
        trail: &mut Vec<Op>,
    ) -> Result<(), SolveError> {
        debug_assert!(val >= 1 && val <= 9);
        if self.grid[row][col] != 0 {
            // Already assigned contradicts
            return Err(SolveError::Unsolvable);
        }

        // Set grid value (reversible)
        trail.push(Op::SetGrid {
            row,
            col,
            prev: self.grid[row][col],
        });
        self.grid[row][col] = val;

        let bit = 1u16 << (val - 1);

        // Update row/col/box masks (reversible)
        trail.push(Op::SetRowMask {
            row,
            prev: self.row_taken[row],
        });
        trail.push(Op::SetColMask {
            col,
            prev: self.col_taken[col],
        });
        let bix = row / BOX * BOX + col / BOX;
        trail.push(Op::SetBoxMask {
            bix,
            prev: self.box_taken[bix],
        });

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
                trail.push(Op::SetForb {
                    row,
                    col: c,
                    prev_forb,
                    prev_count: prev_cnt,
                });
                self.forb_mask[row][c] = prev_forb | bit;
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
                trail.push(Op::SetForb {
                    row: r,
                    col,
                    prev_forb,
                    prev_count: prev_cnt,
                });
                self.forb_mask[r][col] = prev_forb | bit;
                self.avail_count[r][col] = prev_cnt - 1;
                if self.avail_count[r][col] == 0 {
                    return Err(SolveError::Unsolvable);
                }
            }
        }

        // Box peers (exclude same row/col to avoid double updates)
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
                    trail.push(Op::SetForb {
                        row: rr,
                        col: cc,
                        prev_forb,
                        prev_count: prev_cnt,
                    });
                    self.forb_mask[rr][cc] = prev_forb | bit;
                    self.avail_count[rr][cc] = prev_cnt - 1;
                    if self.avail_count[rr][cc] == 0 {
                        return Err(SolveError::Unsolvable);
                    }
                }
            }
        }

        Ok(())
    }

    /// Propagate all naked singles using cached counts. Trail-aware.
    fn singleton_propagation(&mut self, trail: &mut Vec<Op>) -> Result<(), SolveError> {
        // Preallocate worklist to reduce reallocations (max 81 cells)
        let mut queue: VecDeque<(usize, usize)> = VecDeque::with_capacity(N * N);

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

        // Process the worklist
        while let Some((r, c)) = queue.pop_front() {
            if self.grid[r][c] != 0 {
                continue;
            }
            let cnt = self.avail_count[r][c];
            if cnt != 1 {
                // No longer a single; skip safely
                continue;
            }

            let avail_bits = FULL_MASK_N & !self.forb_mask[r][c];
            if avail_bits == 0 {
                return Err(SolveError::Unsolvable);
            }
            let k = (avail_bits.trailing_zeros() + 1) as u8;

            // Assign and update caches via apply_assignment (trail-aware)
            self.apply_assignment(r, c, k, trail)?;

            // Enqueue peers that became singles
            // Row
            for i in 0..N {
                if i != c && self.grid[r][i] == 0 && self.avail_count[r][i] == 1 {
                    queue.push_back((r, i));
                }
            }
            // Col
            for i in 0..N {
                if i != r && self.grid[i][c] == 0 && self.avail_count[i][c] == 1 {
                    queue.push_back((i, c));
                }
            }
            // Box (exclude row/col to avoid duplicates)
            let (br, bc) = Self::box_index(r, c);
            for dr in 0..BOX {
                for dc in 0..BOX {
                    let rr = br + dr;
                    let cc = bc + dc;
                    if (rr != r)
                        && (cc != c)
                        && self.grid[rr][cc] == 0
                        && self.avail_count[rr][cc] == 1
                    {
                        queue.push_back((rr, cc));
                    }
                }
            }
        }

        Ok(())
    }

    /// Compute LCV using a single pass over peers (row/col plus box excluding duplicates).
    /// Given avail_bits, returns a value 1..=9 to assign.
    fn lcv_from_bits(&self, row: usize, col: usize, avail_bits: Mask) -> u8 {
        // Score per candidate (1..=9), higher is better (least constraining).
        // We count how many peers already forbid this value (no impact), i.e., peer.forb_mask has this bit set.
        let mut scores: [u8; N] = [0; N];

        // Process row peers
        for c in 0..N {
            if c == col || self.grid[row][c] != 0 {
                continue;
            }
            let intersect = avail_bits & self.forb_mask[row][c];
            let mut t = intersect;
            while t != 0 {
                let lsb = t & (!t + 1);
                let idx = lsb.trailing_zeros() as usize; // 0..=8
                scores[idx] = scores[idx].saturating_add(1);
                t ^= lsb;
            }
        }

        // Process column peers
        for r in 0..N {
            if r == row || self.grid[r][col] != 0 {
                continue;
            }
            let intersect = avail_bits & self.forb_mask[r][col];
            let mut t = intersect;
            while t != 0 {
                let lsb = t & (!t + 1);
                let idx = lsb.trailing_zeros() as usize;
                scores[idx] = scores[idx].saturating_add(1);
                t ^= lsb;
            }
        }

        // Process box peers (exclude any in same row or col to avoid double counting)
        let (br, bc) = Self::box_index(row, col);
        for dr in 0..BOX {
            for dc in 0..BOX {
                let rr = br + dr;
                let cc = bc + dc;
                if rr == row || cc == col || self.grid[rr][cc] != 0 {
                    continue;
                }
                let intersect = avail_bits & self.forb_mask[rr][cc];
                let mut t = intersect;
                while t != 0 {
                    let lsb = t & (!t + 1);
                    let idx = lsb.trailing_zeros() as usize;
                    scores[idx] = scores[idx].saturating_add(1);
                    t ^= lsb;
                }
            }
        }

        // Choose candidate with maximum score over available bits (tie-break by last max like previous behavior)
        let mut best_bit: Mask = 0;
        let mut best_score: u8 = 0;
        let mut bits = avail_bits;
        while bits != 0 {
            let lsb = bits & (!bits + 1);
            let idx = lsb.trailing_zeros() as usize;
            let s = scores[idx];
            if s >= best_score {
                best_score = s;
                best_bit = lsb;
            }
            bits ^= lsb;
        }

        // Fallback (shouldn't happen): pick first available
        if best_bit == 0 {
            let first = (avail_bits.trailing_zeros() + 1) as u8;
            return first;
        }

        (best_bit.trailing_zeros() + 1) as u8
    }

    // Undo a single operation
    fn undo(&mut self, op: Op) {
        match op {
            Op::SetGrid { row, col, prev } => self.grid[row][col] = prev,
            Op::SetRowMask { row, prev } => self.row_taken[row] = prev,
            Op::SetColMask { col, prev } => self.col_taken[col] = prev,
            Op::SetBoxMask { bix, prev } => self.box_taken[bix] = prev,
            Op::SetForb {
                row,
                col,
                prev_forb,
                prev_count,
            } => {
                self.forb_mask[row][col] = prev_forb;
                self.avail_count[row][col] = prev_count;
            }
        }
    }

    // Supplemental: peer stats if you still want MRV tie-break (not used in trail solver, kept for completeness)
    fn peers_stat(&self, row: usize, col: usize) -> PeerStat {
        let mut res = PeerStat {
            peers_count: 0,
            peers_domains_sum: 0,
        };
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
        let mut stack: Vec<Candidate> = Vec::with_capacity(128);

        // Backtracking helper: pop a candidate, undo to its mark, forbid that candidate, and continue
        let mut backtrack = |state: &mut SudokuState,
                             trail: &mut Vec<Op>,
                             stack: &mut Vec<Candidate>|
         -> Result<(), SolveError> {
            loop {
                let Some(cand) = stack.pop() else {
                    return Err(SolveError::Unsolvable);
                };
                while trail.len() > cand.mark {
                    let op = trail.pop().unwrap();
                    state.undo(op);
                }
                // Forbid the candidate at its cell (reversible)
                let prev_forb = state.forb_mask[cand.row][cand.col];
                let prev_cnt = state.avail_count[cand.row][cand.col];
                if prev_forb & cand.bit == 0 {
                    trail.push(Op::SetForb {
                        row: cand.row,
                        col: cand.col,
                        prev_forb,
                        prev_count: prev_cnt,
                    });
                    state.forb_mask[cand.row][cand.col] = prev_forb | cand.bit;
                    state.avail_count[cand.row][cand.col] = prev_cnt - 1;
                    if state.avail_count[cand.row][cand.col] == 0 {
                        // Immediate conflict; continue backtracking further
                        continue;
                    }
                }
                return Ok(());
            }
        };

        'outer: loop {
            // Propagate all singles
            if let Err(_) = state.singleton_propagation(&mut trail) {
                // Conflict: backtrack
                backtrack(&mut state, &mut trail, &mut stack)?;
                continue 'outer;
            }

            // Solved?
            if state.is_solved() {
                s.solution = state.grid;
                return Ok(());
            }

            // Choose MRV cell and branch
            let Some((row, col)) = state.find_mrv_cell() else {
                s.solution = state.grid;
                return Ok(());
            };

            let avail_bits = FULL_MASK_N & !state.forb_mask[row][col];
            if avail_bits == 0 {
                // No candidates; backtrack
                backtrack(&mut state, &mut trail, &mut stack)?;
                continue 'outer;
            }

            // Choose LCV and remember other candidates on a single stack
            let choice = state.lcv_from_bits(row, col, avail_bits);
            let bit_choice = 1u16 << (choice - 1);

            // Push other candidates (as forbids to try upon backtrack) with current trail mark
            let mark = trail.len();
            let mut remaining = avail_bits & !bit_choice;
            while remaining != 0 {
                let lsb = remaining & (!remaining + 1);
                stack.push(Candidate {
                    row,
                    col,
                    bit: lsb,
                    mark,
                });
                remaining ^= lsb;
            }

            // Try the chosen assignment; on immediate conflict, we'll backtrack in the next loop iteration
            if let Err(_) = state.apply_assignment(row, col, choice, &mut trail) {
                // Immediate conflict; backtrack now
                backtrack(&mut state, &mut trail, &mut stack)?;
                continue 'outer;
            }
        }
    }
}
