mod field;

pub use self::field::*;

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
}
