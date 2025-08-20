use solver::solver::SolverEngine;
use solver::sudoku::Sudoku;

fn main() {
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

    let mut sudoku = Sudoku::new(init_sudoku);
    let mut solver_engine = SolverEngine::new(solver::solver::Kind::Stoch);
    let res = solver_engine.solve(&mut sudoku);

    println!("{}", res);
    println!("{}", sudoku);
}
