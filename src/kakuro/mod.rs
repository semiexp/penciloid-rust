use std::ops::{BitAnd, BitOr, BitAndAssign, BitOrAssign, Not};

mod field_shape;
mod dictionary;
mod field;
mod generator;
mod evaluator;

const MAX_VAL: i32 = 9;
const MAX_SUM: i32 = MAX_VAL * (MAX_VAL + 1) / 2;
const UNDECIDED: i32 = -1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cand(u32);
const CAND_ALL: Cand = Cand((1 << MAX_VAL) - 1);

impl Cand {
    fn singleton(n: i32) -> Cand {
        Cand(1u32 << (n - 1))
    }
    fn is_set(&self, n: i32) -> bool {
        (self.0 & (1u32 << (n - 1))) != 0
    }
    fn is_empty(&self) -> bool {
        self.0 == 0u32
    }
    fn count_set_cands(&self) -> i32 {
        self.0.count_ones() as i32
    }
    fn smallest_set_cand(&self) -> i32 {
        (self.0.trailing_zeros() + 1) as i32
    }
    fn largest_set_cand(&self) -> i32 {
        (32 - self.0.leading_zeros()) as i32
    }
    fn exclude(&self, n: i32) -> Cand {
        Cand(self.0 & !(1u32 << (n - 1)))
    }
    fn limit_upper_bound(&self, max: i32) -> Cand {
        if max >= MAX_VAL {
            *self
        } else if max >= 1 {
            Cand(self.0 & ((1u32 << max) - 1))
        } else {
            Cand(0)
        }
    }
    fn limit_lower_bound(&self, min: i32) -> Cand {
        if min <= 1 {
            *self
        } else if min <= MAX_VAL {
            Cand(self.0 & !((1u32 << (min - 1)) - 1))
        } else {
            Cand(0)
        }
    }
    fn cand_sum(&self) -> i32 {
        let mut val = *self;
        let mut ret = 0;
        while val.0 != 0 {
            let smallest = val.smallest_set_cand();
            val = val.exclude(smallest);
            ret += smallest;
        }
        ret
    }
    fn take_smallest_k(&self, k: i32) -> (Cand, Cand) {
        let mut small = Cand(0);
        let mut large = *self;
        for _ in 0..k {
            if large.0 == 0 {
                break;
            }
            let nxt = large.smallest_set_cand();
            small |= Cand::singleton(nxt);
            large = large.exclude(nxt);
        }
        (small, large)
    }
    fn take_largest_k(&self, k: i32) -> (Cand, Cand) {
        let mut small = *self;
        let mut large = Cand(0);
        for _ in 0..k {
            if small.0 == 0 {
                break;
            }
            let nxt = small.largest_set_cand();
            large |= Cand::singleton(nxt);
            small = small.exclude(nxt);
        }
        (large, small)
    }
}
impl BitAnd for Cand {
    type Output = Cand;
    fn bitand(self, rhs: Cand) -> Cand {
        Cand(self.0 & rhs.0)
    }
}
impl BitOr for Cand {
    type Output = Cand;
    fn bitor(self, rhs: Cand) -> Cand {
        Cand(self.0 | rhs.0)
    }
}
impl BitAndAssign for Cand {
    fn bitand_assign(&mut self, rhs: Cand) {
        *self = Cand(self.0 & rhs.0);
    }
}
impl BitOrAssign for Cand {
    fn bitor_assign(&mut self, rhs: Cand) {
        *self = Cand(self.0 | rhs.0);
    }
}
impl Not for Cand {
    type Output = Cand;
    fn not(self) -> Cand {
        Cand(CAND_ALL.0 ^ self.0)
    }
}

#[derive(Clone, Copy)]
pub enum Clue {
    NoClue,
    Clue { horizontal: i32, vertical: i32 },
}

#[derive(Clone, Copy)]
pub struct FieldTechnique {
    dictionary: bool,
    unique_position: bool,
    two_cells_propagation: bool,
    naked_pair: bool,
    min_max: bool,
}

impl FieldTechnique {
    fn new() -> FieldTechnique {
        FieldTechnique {
            dictionary: true,
            unique_position: true,
            two_cells_propagation: true,
            naked_pair: true,
            min_max: true,
        }
    }
}
pub use self::field_shape::*;
pub use self::dictionary::Dictionary;
pub use self::field::*;
pub use self::generator::*;
pub use self::evaluator::*;

use super::{Grid, Coord};
pub fn answer_to_problem(ans: &Grid<i32>) -> Grid<Clue> {
    let mut has_clue = Grid::new(ans.height(), ans.width(), false);
    for y in 0..ans.height() {
        for x in 0..ans.width() {
            let loc = Coord { y: y, x: x };
            has_clue[loc] = !(1 <= ans[loc] && ans[loc] <= MAX_VAL);
        }
    }
    let shape = FieldShape::new(&has_clue);
    let mut prob_base = Grid::new(ans.height(), ans.width(), (0, 0));
    for y in 0..ans.height() {
        for x in 0..ans.width() {
            let val = ans[Coord { y: y, x: x }];
            if !(1 <= val && val <= MAX_VAL) { continue; }

            let (g1, g2) = shape.cell_to_groups[Coord { y: y, x: x }];
            match shape.clue_locations[g1 as usize] {
                ClueLocation::Horizontal(h) => prob_base[h as usize].0 += val,
                ClueLocation::Vertical(v) => prob_base[v as usize].1 += val,
            }
            match shape.clue_locations[g2 as usize] {
                ClueLocation::Horizontal(h) => prob_base[h as usize].0 += val,
                ClueLocation::Vertical(v) => prob_base[v as usize].1 += val,
            }
        }
    }
    let mut ret = Grid::new(ans.height(), ans.width(), Clue::NoClue);
    for y in 0..ans.height() {
        for x in 0..ans.width() {
            let loc = Coord { y: y, x: x };
            let v = ans[loc];
            if (1 <= v && v <= MAX_VAL) { continue; }
            ret[loc] = Clue::Clue { horizontal: prob_base[loc].0, vertical: prob_base[loc].1 };
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cand() {
        assert_eq!(Cand::singleton(1), Cand(0b1));
        assert_eq!(Cand::singleton(2), Cand(0b10));
        
        assert_eq!(Cand(0b101).is_set(1), true);
        assert_eq!(Cand(0b101).is_set(2), false);
        assert_eq!(Cand(0b101).is_set(3), true);

        assert_eq!(Cand(0b0).is_empty(), true);
        assert_eq!(Cand(0b1).is_empty(), false);

        assert_eq!(Cand(0b10000).count_set_cands(), 1);
        assert_eq!(Cand(0b101101).count_set_cands(), 4);
        
        assert_eq!(Cand(0b10100).smallest_set_cand(), 3);
        assert_eq!(Cand(0b10110).smallest_set_cand(), 2);
        assert_eq!(Cand(0b10000).smallest_set_cand(), 5);

        assert_eq!(Cand(0b1100).largest_set_cand(), 4);
        assert_eq!(Cand(0b10110).largest_set_cand(), 5);
        assert_eq!(Cand(0b10000).largest_set_cand(), 5);

        assert_eq!(Cand(0b11010).exclude(2), Cand(0b11000));
        assert_eq!(Cand(0b11010).exclude(3), Cand(0b11010));
        
        assert_eq!(Cand(0b11010).limit_upper_bound(5), Cand(0b11010));
        assert_eq!(Cand(0b11010).limit_upper_bound(4), Cand(0b01010));
        assert_eq!(Cand(0b11010).limit_upper_bound(3), Cand(0b00010));
        assert_eq!(Cand(0b11010).limit_upper_bound(2), Cand(0b00010));
        assert_eq!(Cand(0b11010).limit_upper_bound(1), Cand(0b00000));

        assert_eq!(Cand(0b11010).limit_lower_bound(5), Cand(0b10000));
        assert_eq!(Cand(0b11010).limit_lower_bound(4), Cand(0b11000));
        assert_eq!(Cand(0b11010).limit_lower_bound(3), Cand(0b11000));
        assert_eq!(Cand(0b11010).limit_lower_bound(2), Cand(0b11010));
        assert_eq!(Cand(0b11010).limit_lower_bound(1), Cand(0b11010));

        assert_eq!(Cand(0b0).cand_sum(), 0);
        assert_eq!(Cand(0b10110).cand_sum(), 10);
        assert_eq!(Cand(0b100000000).cand_sum(), 9);
        assert_eq!(Cand(0b111111111).cand_sum(), 45);

        assert_eq!(Cand(0b1011011).take_smallest_k(2), (Cand(0b11), Cand(0b1011000)));
        assert_eq!(Cand(0b1011011).take_largest_k(2), (Cand(0b1010000), Cand(0b1011)));
    }
}
