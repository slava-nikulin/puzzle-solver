pub trait Solver {
    fn solve(&mut self) -> bool;
}

pub struct SolverEngine<A: Solver> {
    pub alg: A,
}

impl<A> SolverEngine<A>
where
    A: Solver,
{
    pub fn solve(&mut self) -> bool {
        self.alg.solve()
    }
}
