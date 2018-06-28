use super::super::{Y, X, Coord, Grid};
use super::*;

pub struct Field {
    size: i32,
    n_alpha: i32,
    cand: Grid<Cand>,
    value: Grid<Value>,
    clue_front: Vec<Clue>,
    clue_back: Vec<Clue>,
    inconsistent: bool,
    solved: bool,
}

impl Field {
    pub fn empty_board(size: i32, n_alpha: i32) -> Field {
        Field {
            size,
            n_alpha,
            cand: Grid::new(size, size, Cand((1u32 << n_alpha) - 1)),
            value: Grid::new(size, size, UNDECIDED),
            clue_front: vec![NO_CLUE; (2 * size) as usize],
            clue_back: vec![NO_CLUE; (2 * size) as usize],
            inconsistent: false,
            solved: false,
        }
    }
    pub fn decide(&mut self, cell: Coord, val: Value) {
        let current = self.value[cell];
        if current != UNDECIDED {
            if current != val {
                self.inconsistent = true;
            }
            return;
        }

        self.value[cell] = val;

        if val == EMPTY {
            self.limit_cand(cell, Cand(0));
        } else if val == SOME {
            self.inspect_cell(cell);
        } else if val != UNDECIDED {
            self.limit_cand(cell, Cand::singleton(val.0));

            let (Y(y), X(x)) = cell;
            let limit = !Cand::singleton(val.0);
            for y2 in 0..self.size {
                if y != y2 { self.limit_cand((Y(y2), X(x)), limit); }
            }
            for x2 in 0..self.size {
                if x != x2 { self.limit_cand((Y(y), X(x2)), limit); }
            }
        }
    }
    fn group(&self, gid: i32, pos: i32) -> Coord {
        if gid < self.size {
            (Y(gid), X(pos))
        } else {
            (Y(pos), X(gid - self.size))
        }
    }
    fn limit_cand(&mut self, cell: Coord, lim: Cand) {
        let current_cand = self.cand[cell];

        if (current_cand & lim) == current_cand {
            return;
        }

        let new_cand = current_cand & lim;
        self.cand[cell] = new_cand;
        
        if self.cand[cell] == Cand(0) {
            self.decide(cell, EMPTY);
        }
        self.inspect_cell(cell);
    }
    fn inspect_cell(&mut self, cell: Coord) {
        if self.value[cell] == SOME && self.cand[cell].count_set_cands() == 1 {
            let val = Value(self.cand[cell].smallest_set_cand());
            self.decide(cell, val);
        }
        let size = self.size;
        let (Y(y), X(x)) = cell;
        self.inspect_row(y);
        self.inspect_row(x + size);
    }
    fn inspect_row(&mut self, group: i32) {
        let size = self.size;
        let n_alpha = self.n_alpha;

        let mut n_some = 0;
        let mut n_empty = 0;

        for i in 0..size {
            let v = self.value[self.group(group, i)];
            if v == EMPTY {
                n_empty += 1;
            } else if v != UNDECIDED {
                n_some += 1;
            }
        }

        if n_some == n_alpha {
            for i in 0..size {
                let c = self.group(group, i);
                if self.value[c] == UNDECIDED {
                    self.decide(c, EMPTY);
                }
            }
        }
        if n_empty == size - n_alpha {
            for i in 0..size {
                let c = self.group(group, i);
                if self.value[c] == UNDECIDED {
                    self.decide(c, SOME);
                }
            }
        }
        let clue_front = self.clue_front[group as usize];
        if clue_front != NO_CLUE {
            // TODO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduction() {
        {
            // a symbol shouldn't occur more than once in a row / a column
            let mut field = Field::empty_board(5, 3);

            assert_eq!(field.cand[(Y(0), X(0))], Cand(7));
            assert_eq!(field.cand[(Y(0), X(1))], Cand(7));
            assert_eq!(field.cand[(Y(1), X(1))], Cand(7));

            field.decide((Y(0), X(0)), Value(0));

            assert_eq!(field.inconsistent, false);
            assert_eq!(field.cand[(Y(0), X(0))], Cand(1));
            assert_eq!(field.cand[(Y(0), X(1))], Cand(6));
            assert_eq!(field.cand[(Y(1), X(0))], Cand(6));
            assert_eq!(field.cand[(Y(1), X(1))], Cand(7));
        }
        {
            // there must be exactly `n_alpha` symbols in a row / a column
            let mut field = Field::empty_board(5, 3);

            field.decide((Y(1), X(0)), SOME);
            field.decide((Y(1), X(1)), SOME);
            field.decide((Y(1), X(2)), SOME);

            assert_eq!(field.inconsistent, false);
            assert_eq!(field.value[(Y(1), X(3))], EMPTY);
        }
        {
            // there must be exactly `n_alpha` symbols in a row / a column
            let mut field = Field::empty_board(5, 3);

            field.decide((Y(3), X(2)), EMPTY);
            field.decide((Y(4), X(2)), EMPTY);

            assert_eq!(field.inconsistent, false);
            assert_eq!(field.value[(Y(1), X(2))], SOME);
        }
    }
}
