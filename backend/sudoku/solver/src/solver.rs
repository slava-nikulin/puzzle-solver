use crate::{dfs::DfsBacktracking, stochastic::StochasticBacktracking, sudoku::Sudoku};

pub enum Kind {
    Stoch,
    Dfs,
}

// Concrete strategies
enum SolverEnum {
    Stoch(StochasticBacktracking),
    Dfs(DfsBacktracking),
}

// Abstract strategy
impl SolverEnum {
    fn solve(&mut self, s: &mut Sudoku) -> bool {
        match self {
            SolverEnum::Stoch(a) => a.solve(s),
            SolverEnum::Dfs(a) => a.solve(s),
        }
    }
}

// Strategy context
pub struct SolverEngine {
    alg: SolverEnum,
}

impl SolverEngine {
    pub fn new(kind: Kind) -> Self {
        Self {
            alg: match kind {
                Kind::Stoch => SolverEnum::Stoch(StochasticBacktracking),
                Kind::Dfs => SolverEnum::Dfs(DfsBacktracking),
            },
        }
    }

    pub fn solve(&mut self, s: &mut Sudoku) -> bool {
        self.alg.solve(s)
    }
}
