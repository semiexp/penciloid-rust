mod field;
mod dictionary;
mod generator;
mod io;

pub use self::field::*;
pub use self::dictionary::*;
pub use self::generator::*;
pub use self::io::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Undecided,
    Black,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clue(pub i32);

pub const NO_CLUE: Clue = Clue(-1);
pub const CLUE_TYPES: usize = 23;
pub const CLUE_VALUES: [[i32; 5]; CLUE_TYPES] = [
    [-1, -1, -1, -1, -1],
    [1, -1, -1, -1, -1],
    [1, 1, -1, -1, -1],
    [1, 1, 1, -1, -1],
    [1, 1, 1, 1, -1],
    [1, 1, 2, -1, -1],
    [1, 1, 3, -1, -1],
    [1, 2, -1, -1, -1],
    [1, 2, 2, -1, -1],
    [1, 3, -1, -1, -1],
    [1, 4, -1, -1, -1],
    [1, 5, -1, -1, -1],
    [2, -1, -1, -1, -1],
    [2, 2, -1, -1, -1],
    [2, 3, -1, -1, -1],
    [2, 4, -1, -1, -1],
    [3, -1, -1, -1, -1],
    [3, 3, -1, -1, -1],
    [4, -1, -1, -1, -1],
    [5, -1, -1, -1, -1],
    [6, -1, -1, -1, -1],
    [7, -1, -1, -1, -1],
    [8, -1, -1, -1, -1],
];
pub const CLUE_MAX: [i32; CLUE_TYPES] = [
    0, 1, 1, 1, 1, 2, 3, 2, 2, 3, 4, 5, 2, 2, 3, 4, 3, 3, 4, 5, 6, 7, 8
];

pub fn clue_pattern_to_id(pat: &[i32]) -> Option<Clue> {
    let mut sorted = Vec::from(pat);
    sorted.sort();

    if sorted.len() == 1 && sorted[0] == 0 {
        return Some(Clue(0));
    }

    for i in 0..CLUE_TYPES {
        if (&sorted)
            .into_iter()
            .eq(CLUE_VALUES[i].into_iter().filter(|&&x| x != -1))
        {
            return Some(Clue(i as i32));
        }
    }

    None
}
