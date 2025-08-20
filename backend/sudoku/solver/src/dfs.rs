use crate::sudoku::Sudoku;

struct SudokuState {
    grid: [[i8; 9]; 9],
    candidates: [[u16; 9]; 9],
}

impl SudokuState {
    fn new(grid: [[i8; 9]; 9]) -> Self {
        Self {
            grid,
            candidates: [[0; 9]; 9],
        }
    }
    // Minimum Remaining Values
    fn mrv(&self) -> (usize, usize) {
        for i in 0..3 {
            for j in 0..3 {
                //TODO
            }
        }
    }

    fn mark_candidate(&mut self, i: usize, j: usize, n: u8) {
        self.candidates[i][j] |= 1u16 << (n - 1)
    }
}

pub struct DfsBacktracking;

impl DfsBacktracking {
    pub fn solve(&mut self, s: &mut Sudoku) -> bool {
        let dfs_stack: Vec<SudokuState> = Vec::new();

        false
    }
}
