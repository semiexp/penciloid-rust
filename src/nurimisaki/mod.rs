mod field;

pub use self::field::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Cell {
    Undecided,
    White,
    Black,
    Cape(i32),
}

impl Cell {
    pub fn is_white_like(self) -> bool {
        match self {
            Cell::White | Cell::Cape(_) => true,
            _ => false,
        }
    }
    pub fn is_cape(self) -> bool {
        match self {
            Cell::Cape(_) => true,
            _ => false,
        }
    }
}
