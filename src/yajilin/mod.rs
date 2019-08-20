mod field;
mod generator;

pub use self::field::*;
pub use self::generator::*;
use super::D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clue {
    NoClue,
    Empty,
    Up(i32),
    Left(i32),
    Down(i32),
    Right(i32),
}

impl Clue {
    pub fn same_shape(self, other: Clue) -> bool {
        match (self, other) {
            (Clue::Up(_), Clue::Up(_))
            | (Clue::Left(_), Clue::Left(_))
            | (Clue::Down(_), Clue::Down(_))
            | (Clue::Right(_), Clue::Right(_)) => true,
            _ => false,
        }
    }
    pub fn clue_number(self) -> i32 {
        match self {
            Clue::NoClue | Clue::Empty => -1,
            Clue::Up(n) | Clue::Left(n) | Clue::Down(n) | Clue::Right(n) => n,
        }
    }
    pub fn get_direction(self) -> D {
        match self {
            Clue::NoClue | Clue::Empty => D(0, 0),
            Clue::Up(_) => D(-1, 0),
            Clue::Left(_) => D(0, -1),
            Clue::Down(_) => D(1, 0),
            Clue::Right(_) => D(0, 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Clue,
    Undecided,
    Line,
    Blocked,
}

impl Cell {
    pub fn is_blocking(self) -> bool {
        match self {
            Cell::Clue | Cell::Blocked => true,
            _ => false,
        }
    }
    pub fn can_be_blocked(self) -> bool {
        match self {
            Cell::Undecided | Cell::Blocked => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Technique {
    pub two_by_two: bool,
    pub two_by_three: bool,
    pub one_in_three_orthogonal_either: bool,
    pub one_in_three_remote: bool,
    pub inout_advanced: bool,
    pub local_parity: bool,
    pub two_rows: bool,
    pub avoid_branching: bool,
}

impl Technique {
    pub fn new() -> Technique {
        Technique::with_all(true)
    }
    pub fn disabled_all() -> Technique {
        Technique::with_all(false)
    }
    fn with_all(val: bool) -> Technique {
        Technique {
            two_by_two: val,
            two_by_three: val,
            one_in_three_orthogonal_either: val,
            one_in_three_remote: val,
            inout_advanced: val,
            local_parity: val,
            two_rows: val,
            avoid_branching: val,
        }
    }
}
