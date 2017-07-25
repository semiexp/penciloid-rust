mod field_shape;
mod dictionary;

const MAX_VAL: i32 = 9;
const MAX_SUM: i32 = MAX_VAL * (MAX_VAL + 1) / 2;
type Cand = u32;
const CAND_ALL: Cand = (1 << MAX_VAL) - 1;

pub use self::field_shape::*;
pub use self::dictionary::*;
