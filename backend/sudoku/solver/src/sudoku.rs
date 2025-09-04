use std::fmt;

pub struct Sudoku<const N: usize, const BR: usize, const BC: usize> {
    pub(crate) init: [[u8; N]; N],
    pub(crate) solution: [[u8; N]; N],
}

pub type Sudoku9 = Sudoku<9, 3, 3>;
pub type Sudoku6 = Sudoku<6, 2, 3>;

impl<const N: usize, const BR: usize, const BC: usize> Sudoku<N, BR, BC> {
    pub fn new(init: [[u8; N]; N]) -> Self {
        Sudoku {
            init,
            solution: init,
        }
    }

    #[inline]
    pub fn box_index(r: usize, c: usize) -> usize {
        (r / BR) * BC + (c / BC)
    }

    #[inline]
    pub fn box_coord(i: usize, j: usize) -> (usize, usize) {
        (i / BR * BC, j / BC * BR)
    }

    pub fn check(&self) -> bool {
        let full: u32 = (1u32 << N) - 1;

        for i in 0..N {
            let (mut row, mut col) = (0u32, 0u32);
            for j in 0..N {
                let rv = self.solution[i][j];
                let cv = self.solution[j][i];
                if rv == 0 || cv == 0 {
                    return false;
                }

                let rb = 1u32 << (rv - 1);
                let cb = 1u32 << (cv - 1);

                if (row & rb) != 0 || (col & cb) != 0 {
                    return false;
                }

                row |= rb;
                col |= cb;
            }

            if row != full || col != full {
                return false;
            }
        }

        for br in (0..N).step_by(BR) {
            for bc in (0..N).step_by(BC) {
                let mut boxm = 0u32;
                for dr in 0..BR {
                    for dc in 0..BC {
                        let v = self.solution[br + dr][bc + dc];
                        let b = 1u32 << (v - 1);
                        if (boxm & b) != 0 {
                            return false;
                        }
                        boxm |= b;
                    }
                }
                if boxm != full {
                    return false;
                }
            }
        }

        true
    }
}

impl<const N: usize, const BR: usize, const BC: usize> fmt::Display for Sudoku<N, BR, BC> {
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
