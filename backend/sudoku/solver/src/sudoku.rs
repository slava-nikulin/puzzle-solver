use std::fmt;

pub struct Sudoku {
    pub(crate) init: [[u8; 9]; 9],
    pub(crate) solution: [[u8; 9]; 9],
}

impl Sudoku {
    pub fn new(init: [[u8; 9]; 9]) -> Self {
        Sudoku {
            init,
            solution: init,
        }
    }

    pub fn check(&self) -> bool {
        let mut small_square_row_num: usize;
        let mut small_square_col_num: usize;
        let mut small_square_row_num_check: usize;
        let mut small_square_col_num_check: usize;
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    small_square_row_num = i * 3 + k;
                    for l in 0..3 {
                        small_square_col_num = j * 3 + l;
                        for n in 0..3 {
                            small_square_row_num_check = i * 3 + n;
                            for m in 0..3 {
                                small_square_col_num_check = j * 3 + m;

                                if small_square_row_num != small_square_row_num_check
                                    && small_square_col_num != small_square_col_num_check
                                    && self.solution[small_square_row_num][small_square_col_num]
                                        == self.solution[small_square_row_num_check]
                                            [small_square_col_num_check]
                                {
                                    return false;
                                }
                            }
                        }

                        for n in 0..9 {
                            if (n != small_square_row_num
                                && self.solution[n][small_square_col_num]
                                    == self.solution[small_square_row_num][small_square_col_num])
                                || (n != small_square_col_num
                                    && self.solution[small_square_row_num][n]
                                        == self.solution[small_square_row_num]
                                            [small_square_col_num])
                            {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        true
    }
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.solution {
            for val in row {
                write!(f, "{} ", val)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
