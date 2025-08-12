fn main() {
    let sudoku: Vec<Vec<i8>> = vec![vec![2; 9]; 9];

    for row in &sudoku {
        for val in row {
            print!("{} ", val);
        }
        println!();
    }
}
