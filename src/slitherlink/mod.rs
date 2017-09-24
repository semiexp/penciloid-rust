mod field;
mod dictionary;

pub use self::field::*;
pub use self::dictionary::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clue(pub i32);
const NO_CLUE: Clue = Clue(-1);
