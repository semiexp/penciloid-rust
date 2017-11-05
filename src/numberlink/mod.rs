mod solver;

pub use self::solver::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Clue(pub i32);

const NO_CLUE: Clue = Clue(0);

use super::{Grid, Y, X, Coord};

#[derive(Clone)]
pub struct LinePlacement {
    right: Grid<bool>,
    down: Grid<bool>,
}

impl LinePlacement {
    pub fn new(height: i32, width: i32) -> LinePlacement {
        LinePlacement {
            right: Grid::new(height, width - 1, false),
            down: Grid::new(height - 1, width, false),
        }
    }
    pub fn right(&self, cd: Coord) -> bool {
        self.right.is_valid_coord(cd) && self.right[cd]
    }
    pub fn set_right(&mut self, cd: Coord, e: bool) {
        self.right[cd] = e;
    }
    pub fn down(&self, cd: Coord) -> bool {
        self.down.is_valid_coord(cd) && self.down[cd]
    }
    pub fn set_down(&mut self, cd: Coord, e: bool) {
        self.down[cd] = e;
    }
}
