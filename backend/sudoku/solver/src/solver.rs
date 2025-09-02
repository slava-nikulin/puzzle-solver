use crate::{dfs::DfsBacktracking, sudoku::Sudoku};
use thiserror::Error;

pub enum Kind {
    Dfs,
}

// Concrete strategies
enum SolverEnum<const N: usize, const BR: usize, const BC: usize> {
    Dfs(DfsBacktracking<N, BR, BC>),
}

#[derive(Debug, Error)]
pub enum SolveError {
    #[error("puzzle is structurally invalid: {0}")]
    InvalidPuzzle(&'static str),

    #[error("puzzle has no solution")]
    Unsolvable,
}

// Abstract strategy
impl<const N: usize, const BR: usize, const BC: usize> SolverEnum<N, BR, BC> {
    fn solve(&mut self, s: &mut Sudoku<N, BR, BC>) -> Result<(), SolveError> {
        match self {
            SolverEnum::Dfs(a) => a.solve(s),
        }
    }
}

// Strategy context
pub struct SolverEngine<const N: usize, const BR: usize, const BC: usize> {
    alg: SolverEnum<N, BR, BC>,
}

impl<const N: usize, const BR: usize, const BC: usize> SolverEngine<N, BR, BC> {
    pub fn new(kind: Kind) -> Self {
        Self {
            alg: match kind {
                Kind::Dfs => SolverEnum::Dfs(DfsBacktracking),
            },
        }
    }

    pub fn solve(&mut self, s: &mut Sudoku<N, BR, BC>) -> Result<(), SolveError> {
        self.alg.solve(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::sudoku::Sudoku9;

    use super::*;

    #[test]
    fn dfs_backtracking_ok() {
        let init_sudoku = [
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
        let expected = [
            [9, 2, 6, 3, 4, 5, 8, 1, 7],
            [8, 5, 1, 7, 2, 6, 3, 9, 4],
            [4, 7, 3, 8, 9, 1, 2, 6, 5],
            [5, 6, 8, 9, 1, 3, 4, 7, 2],
            [3, 4, 2, 6, 8, 7, 9, 5, 1],
            [1, 9, 7, 2, 5, 4, 6, 3, 8],
            [6, 8, 5, 4, 7, 9, 1, 2, 3],
            [7, 3, 4, 1, 6, 2, 5, 8, 9],
            [2, 1, 9, 5, 3, 8, 7, 4, 6],
        ];
        let mut sudoku = Sudoku9::new(init_sudoku);
        let mut solver_engine = SolverEngine::new(Kind::Dfs);
        let res = solver_engine.solve(&mut sudoku);

        assert!(res.is_ok());
        assert_eq!(sudoku.solution, expected);
        assert!(sudoku.check())
    }

    #[test]
    fn dfs_backtracking_zero_ok() {
        let init_sudoku = [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];

        let mut sudoku = Sudoku9::new(init_sudoku);
        let mut solver_engine = SolverEngine::new(Kind::Dfs);
        let res = solver_engine.solve(&mut sudoku);

        assert!(res.is_ok());
        assert!(sudoku.check())
    }
}
