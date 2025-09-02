use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use solver::{
    solver::{Kind, SolverEngine},
    sudoku::Sudoku9,
};
use std::hint::black_box;

fn bench_algos(c: &mut Criterion) {
    let init = [
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

    c.bench_function("dfs", |b| {
        b.iter_batched(
            || Sudoku9::new(init),
            |mut s| {
                let mut eng = SolverEngine::new(Kind::Dfs);
                black_box(eng.solve(&mut s)).unwrap();
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_algos);
criterion_main!(benches);
