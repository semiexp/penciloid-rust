use std::ops::{BitAnd, BitOr, BitAndAssign, BitOrAssign, Not};

mod field;

pub use self::field::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cand(pub u32);

impl Cand {
    fn singleton(n: i32) -> Cand {
        Cand(1u32 << n)
    }
    fn is_set(&self, n: i32) -> bool {
        (self.0 & (1u32 << n)) != 0
    }
    fn count_set_cands(&self) -> i32 {
        self.0.count_ones() as i32
    }
    fn smallest_set_cand(&self) -> i32 {
        self.0.trailing_zeros() as i32
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
        Cand(!self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Value(pub i32);

pub const UNDECIDED: Value = Value(-1);
pub const EMPTY: Value = Value(-2);
pub const SOME: Value = Value(-3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clue(pub i32);

pub const NO_CLUE: Clue = Clue(-1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClueLoc {
    Left,
    Right,
    Top,
    Bottom,
}
