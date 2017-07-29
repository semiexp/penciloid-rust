use super::super::{Grid, Coord, FiniteSearchQueue};
use super::*;

#[derive(Clone, Copy)]
struct FieldGrp {
    unmet_num: i32,
    unmet_sum: i32,
    unused: Cand,
}
pub struct Field<'a> {
    dic: &'a Dictionary,
    shape: FieldShape,
    grps: Vec<FieldGrp>,
    val: Vec<i32>,
    cand: Vec<Cand>,
    inconsistent: bool,
    queue: FiniteSearchQueue
}
impl<'a> Field<'a> {
    pub fn new(problem: &Grid<Clue>, dic: &'a Dictionary) -> Field<'a> {
        let n_cells = (problem.height() * problem.width()) as usize;
        let mut has_clue = Grid::new(problem.height(), problem.width(), false);
        for i in 0..n_cells {
            if let Clue::Clue { vertical: _, horizontal: _ } = problem[i] {
                has_clue[i] = true;
            }
        }
        let shape = FieldShape::new(&has_clue);

        let mut grps = vec![FieldGrp {
            unmet_num: 0,
            unmet_sum: 0,
            unused: 0,
        }; shape.group_to_cells.len()];
        let n_groups = grps.len();

        for i in 0..shape.group_to_cells.len() {
            let loc = shape.clue_locations[i];
            let clue_val = match loc {
                ClueLocation::Vertical(v) => match problem[v as usize] {
                    Clue::NoClue => panic!("unexpected condition"),
                    Clue::Clue { vertical: v, horizontal: _ } => v,
                },
                ClueLocation::Horizontal(h) => match problem[h as usize] {
                    Clue::NoClue => panic!("unexpected condition"),
                    Clue::Clue { vertical: _, horizontal: h } => h,
                },
            };
            let mut n_cells = 0;
            for _ in shape.group_to_cells[i] {
                n_cells += 1;
            }
            grps[i] = FieldGrp {
                unmet_num: n_cells,
                unmet_sum: clue_val,
                unused: CAND_ALL,
            };
        }

        Field {
            dic: dic,
            shape: shape,
            grps: grps,
            val: vec![UNDECIDED; n_cells],
            cand: vec![CAND_ALL; n_cells],
            inconsistent: false,
            queue: FiniteSearchQueue::new(n_groups as i32),
        }
    }
    pub fn inconsistent(&self) -> bool {
        self.inconsistent
    }
    pub fn height(&self) -> i32 {
        self.shape.has_clue.height()
    }
    pub fn width(&self) -> i32 {
        self.shape.has_clue.width()
    }
    pub fn val(&self, loc: Coord) -> i32 {
        self.val[self.location(loc)]
    }
    pub fn decide(&mut self, loc: Coord, val: i32) {
        let loc = self.location(loc);

        self.queue.start();
        self.decide_int(loc, val);
        while !self.queue.empty() {
            let g = self.queue.pop();
            self.check_group(g);
        }
        self.queue.finish();

    }
    pub fn check_all(&mut self) {
        self.queue.start();
        for i in 0..(self.grps.len() as i32) {
            self.queue.push(i);
        }
        while !self.queue.empty() {
            let g = self.queue.pop();
            self.check_group(g);
        }
        self.queue.finish();
    }
    fn location(&self, loc: Coord) -> usize {
        self.shape.has_clue.index(loc)
    }
    fn decide_int(&mut self, loc: usize, val: i32) {
        if self.val[loc] != UNDECIDED {
            if self.val[loc] != val {
                self.inconsistent = true;
            }
            return;
        }

        self.val[loc] = val;
        if (self.cand[loc] & (1 << (val - 1))) == 0 {
            self.inconsistent = true;
            return;
        }
        self.cand[loc] = 1 << (val - 1);

        let (g1, g2) = self.shape.cell_to_groups[loc];
        self.grps[g1 as usize].unmet_num -= 1;
        self.grps[g1 as usize].unmet_sum -= val;
        if (self.grps[g1 as usize].unused & (1 << (val - 1) as Cand)) == 0 {
            self.inconsistent = true;
            return;
        }
        self.grps[g1 as usize].unused &= !(1 << (val - 1) as Cand);
        self.grps[g2 as usize].unmet_num -= 1;
        self.grps[g2 as usize].unmet_sum -= val;
        if (self.grps[g2 as usize].unused & (1 << (val - 1) as Cand)) == 0 {
            self.inconsistent = true;
            return;
        }
        self.grps[g2 as usize].unused &= !(1 << (val - 1) as Cand);

        self.eliminate_cand_from_group(g1, val, loc as i32);
        self.eliminate_cand_from_group(g2, val, loc as i32);

        self.queue.push(g1);
        self.queue.push(g2);
    }
    fn eliminate_cand_from_group(&mut self, grp: i32, rem_cand: i32, cur: i32) {
        let cand = !(1 << (rem_cand - 1) as Cand);
        for c in self.shape.group_to_cells[grp as usize] {
            if c != cur {
                self.limit_cand(c as usize, cand);
            }
        }
    }
    fn limit_cand(&mut self, loc: usize, lim: Cand) {
        if self.cand[loc] & lim == self.cand[loc] {
            return;
        }
        self.cand[loc] &= lim;

        let current_cand = self.cand[loc];
        if current_cand == 0 {
            self.inconsistent = true;
            return;
        }
        if current_cand.count_ones() == 1 {
            self.decide_int(loc, (current_cand.trailing_zeros() + 1) as i32);
        }

        let (g1, g2) = self.shape.cell_to_groups[loc];
        self.queue.push(g1);
        self.queue.push(g2);
    }
    fn check_group(&mut self, gid: i32) {
        let grp = self.grps[gid as usize];
        let (imperative, allowed) = self.dic.at(grp.unmet_num, grp.unmet_sum, grp.unused);
        if (imperative, allowed) == dictionary::IMPOSSIBLE {
            self.inconsistent = true;
            return;
        }

        // unique position technique
        if imperative != 0 {
            let mut uniq = 0;
            let mut mult = 0;
            for c in self.shape.group_to_cells[gid as usize] {
                if self.val[c as usize] == UNDECIDED {
                    mult |= uniq & self.cand[c as usize];
                    uniq |= self.cand[c as usize];
                }
            }
            uniq &= imperative & !mult;
            if uniq != 0 {
                for c in self.shape.group_to_cells[gid as usize] {
                    if self.val[c as usize] == UNDECIDED && (self.cand[c as usize] & uniq) != 0 {
                        let val = ((self.cand[c as usize] & uniq).trailing_zeros() + 1) as i32;
                        self.decide_int(c as usize, val);
                    }
                }
            }
        }

        // candidate limitation
        for c in self.shape.group_to_cells[gid as usize] {
            if self.val[c as usize] == UNDECIDED {
                self.limit_cand(c as usize, allowed);
            }
        }

        // two-cells propagation (TODO: improve complexity)
        let grp = self.grps[gid as usize];
        if grp.unmet_num == 2 {
            let mut c1 = -1;
            let mut c2 = -1;
            for c in self.shape.group_to_cells[gid as usize] {
                if self.val[c as usize] == UNDECIDED {
                    if c1 == -1 {
                        c1 = c;
                    } else {
                        c2 = c;
                    }
                }
            }
            for i in 1..(MAX_VAL + 1) {
                if (self.cand[c1 as usize] & (1 << (i - 1) as Cand) == 0) && 1 <= (grp.unmet_sum - i) && (grp.unmet_sum - i) <= MAX_VAL {
                    self.limit_cand(c2 as usize, !(1 << (grp.unmet_sum - i - 1) as Cand));
                }
                if (self.cand[c2 as usize] & (1 << (i - 1) as Cand) == 0) && 1 <= (grp.unmet_sum - i) && (grp.unmet_sum - i) <= MAX_VAL {
                    self.limit_cand(c1 as usize, !(1 << (grp.unmet_sum - i - 1) as Cand));
                }
            }
        }
        
        // naked pair (TODO: improve complexity)
        for c in self.shape.group_to_cells[gid as usize] {
            if self.val[c as usize] != -1 { continue; }
            for d in self.shape.group_to_cells[gid as usize] {
                if self.val[d as usize] != -1 { continue; }
                if c != d && self.cand[c as usize] == self.cand[d as usize] && self.cand[c as usize].count_ones() == 2 {
                    for e in self.shape.group_to_cells[gid as usize] {
                        if c != e && d != e {
                            let lim = !self.cand[c as usize];
                            self.limit_cand(e as usize, lim);
                        }
                    }
                }
            }
        }

        // min-max method
        let grp = self.grps[gid as usize];
        let mut min_sum = 0;
        let mut max_sum = 0;
        for c in self.shape.group_to_cells[gid as usize] {
            if self.val[c as usize] != -1 { continue; }
            let cand = self.cand[c as usize];
            min_sum += cand.trailing_zeros() + 1;
            max_sum += 32 - cand.leading_zeros();
        }
        for c in self.shape.group_to_cells[gid as usize] {
            if self.val[c as usize] != -1 { continue; }
            let cand = self.cand[c as usize];

            let current_max = grp.unmet_sum - (min_sum - (cand.trailing_zeros() + 1)) as i32;
            let current_min = grp.unmet_sum - (max_sum - (32 - cand.leading_zeros())) as i32;

            let mut lim = CAND_ALL;
            if current_max <= 8 {
                lim &= (1 << current_max as Cand) - 1;
            }
            if current_min >= 2 {
                lim &= !((1 << (current_min as Cand - 1)) - 1);
            }
            if lim != CAND_ALL {
                self.limit_cand(c as usize, lim);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common;

    #[test]
    fn test_field() {
        let dic = Dictionary::default();
        let mut problem_base = Grid::new(3, 3, Clue::NoClue);
        problem_base[Coord { y: 0, x: 0 }] = Clue::Clue { horizontal: -1, vertical: -1 };
        problem_base[Coord { y: 0, x: 1 }] = Clue::Clue { horizontal: -1, vertical: 3 };
        problem_base[Coord { y: 0, x: 2 }] = Clue::Clue { horizontal: -1, vertical: 8 };
        problem_base[Coord { y: 1, x: 0 }] = Clue::Clue { horizontal: 4, vertical: -1 };
        problem_base[Coord { y: 2, x: 0 }] = Clue::Clue { horizontal: 7, vertical: -1 };

        let mut field = Field::new(&problem_base, &dic);
        field.check_all();

        assert_eq!(field.val(Coord { y: 1, x: 1 }), 1);
        assert_eq!(field.val(Coord { y: 1, x: 2 }), 3);
        assert_eq!(field.val(Coord { y: 2, x: 1 }), 2);
        assert_eq!(field.val(Coord { y: 2, x: 2 }), 5);
        assert_eq!(field.inconsistent(), false);
    }

    #[test]
    fn test_inconsistent_field() {
        let dic = Dictionary::default();
        let mut problem_base = Grid::new(3, 3, Clue::NoClue);
        problem_base[Coord { y: 0, x: 0 }] = Clue::Clue { horizontal: -1, vertical: -1 };
        problem_base[Coord { y: 0, x: 1 }] = Clue::Clue { horizontal: -1, vertical: 3 };
        problem_base[Coord { y: 0, x: 2 }] = Clue::Clue { horizontal: -1, vertical: 6 };
        problem_base[Coord { y: 1, x: 0 }] = Clue::Clue { horizontal: 4, vertical: -1 };
        problem_base[Coord { y: 2, x: 0 }] = Clue::Clue { horizontal: 5, vertical: -1 };

        let mut field = Field::new(&problem_base, &dic);
        field.check_all();

        assert_eq!(field.inconsistent(), true);
    }
}
