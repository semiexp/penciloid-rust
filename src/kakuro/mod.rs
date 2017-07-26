mod field_shape;
mod dictionary;
mod field;

const MAX_VAL: i32 = 9;
const MAX_SUM: i32 = MAX_VAL * (MAX_VAL + 1) / 2;
type Cand = u32;
const CAND_ALL: Cand = (1 << MAX_VAL) - 1;
const UNDECIDED: i32 = -1;

#[derive(Clone, Copy)]
pub enum Clue {
    NoClue,
    Clue { horizontal: i32, vertical: i32 },
}

pub use self::field_shape::*;
pub use self::dictionary::*;
pub use self::field::*;
