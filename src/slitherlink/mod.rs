mod field;
mod dictionary;
mod generator;
mod format;

pub use self::field::*;
pub use self::dictionary::*;
pub use self::generator::*;
pub use self::format::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clue(pub i32);
const NO_CLUE: Clue = Clue(-1);
