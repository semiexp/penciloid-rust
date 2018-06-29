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
        assert!(n_alpha >= 2);
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
    fn get_clue(&self, loc: ClueLoc, idx: i32) -> Clue {
        match loc {
            ClueLoc::Left => self.clue_front[idx as usize],
            ClueLoc::Right => self.clue_back[idx as usize],
            ClueLoc::Top => self.clue_front[(idx + self.size) as usize],
            ClueLoc::Bottom => self.clue_back[(idx + self.size) as usize],
        }
    }
    fn set_clue(&mut self, loc: ClueLoc, idx: i32, clue: Clue) {
        let current = self.get_clue(loc, idx);
        if current != NO_CLUE {
            if current != clue {
                self.inconsistent = true;
            }
            return;
        }
        let size = self.size;
        match loc {
            ClueLoc::Left => self.clue_front[idx as usize] = clue,
            ClueLoc::Right => self.clue_back[idx as usize] = clue,
            ClueLoc::Top => self.clue_front[(idx + size) as usize] = clue,
            ClueLoc::Bottom => self.clue_back[(idx + size) as usize] = clue,
        }
        if loc == ClueLoc::Left || loc == ClueLoc::Right {
            self.inspect_row(idx);
        } else {
            self.inspect_row(idx + size);
        }
    }
    /// Returns `pos`-th cell of group `gid`.
    fn group(&self, gid: i32, pos: i32) -> Coord {
        if gid < self.size {
            (Y(gid), X(pos))
        } else {
            (Y(pos), X(gid - self.size))
        }
    }
    /// Returns `pos`-th cell of group `gid` with reversed indexing of cells when `dir` is `true`.
    fn directed_group(&self, gid: i32, pos: i32, dir: bool) -> Coord {
        self.group(gid, if dir { self.size - pos - 1 } else { pos })
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

        for a in 0..n_alpha {
            let mut loc = -1;
            for i in 0..size {
                if self.cand[self.group(group, i)].is_set(a) {
                    if loc == -1 {
                        loc = i;
                    } else {
                        loc = -2;
                        break;
                    }
                }
            } 
            if loc == -1 {
                self.inconsistent = true;
                return;
            } else if loc != -2 {
                let c = self.group(group, loc);
                self.decide(c, Value(a));
            }
        }
        for &dir in [true, false].iter() {
            let clue = if !dir { self.clue_front[group as usize] } else { self.clue_back[group as usize] };
            if clue == NO_CLUE {
                continue;
            }

            let mut first_nonempty_id = -1;
            for i in 0..size {
                let v = self.value[self.directed_group(group, i, dir)];
                if v != EMPTY {
                    first_nonempty_id = i;
                    break;
                }
            }
            if first_nonempty_id == -1 {
                self.inconsistent = true;
                return;
            }
            let first_nonempty_cell = self.directed_group(group, first_nonempty_id, dir);
            self.limit_cand(first_nonempty_cell, Cand::singleton(clue.0));

            let mut first_diff_id = -1;
            for i in 0..size {
                let v = self.value[self.directed_group(group, i, dir)];
                if v != UNDECIDED && !self.cand[self.directed_group(group, i, dir)].is_set(clue.0) {
                    first_diff_id = i;
                }
            }
            if first_diff_id != -1 {
                for i in (first_diff_id + 1)..size {
                    let c = self.directed_group(group, i, dir);
                    self.limit_cand(c, !Cand::singleton(clue.0));
                }
            }

            let mut n_back_diff = n_alpha - 1;
            for i in 0..size {
                let c = self.directed_group(group, i, !dir);
                let v = self.value[c];
                if v != EMPTY {
                    self.limit_cand(c, !Cand::singleton(clue.0));
                    n_back_diff -= 1;
                    if n_back_diff == 0 { break; }
                }
            }
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
        {
            let mut field = Field::empty_board(5, 3);

            field.limit_cand((Y(0), X(2)), Cand(5));
            field.limit_cand((Y(2), X(2)), Cand(5));
            field.limit_cand((Y(3), X(2)), Cand(5));
            field.limit_cand((Y(4), X(2)), Cand(5));

            assert_eq!(field.inconsistent, false);
            assert_eq!(field.value[(Y(1), X(2))], Value(1));
            assert_eq!(field.cand[(Y(1), X(3))], Cand(5));
        }
    }

    #[test]
    fn test_clue() {
        {
            let mut field = Field::empty_board(5, 3);

            field.set_clue(ClueLoc::Left, 0, Clue(0));

            assert_eq!(field.cand[(Y(0), X(0))], Cand(1));
            assert_eq!(field.cand[(Y(0), X(3))], Cand(6));
            assert_eq!(field.cand[(Y(0), X(4))], Cand(6));
        }
    }
}
