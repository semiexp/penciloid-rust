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
