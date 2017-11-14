use std::ops::Index;

mod solver;
mod solver2;
mod generator;

pub use self::solver::*;
pub use self::solver2::*;
pub use self::generator::*;

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
    pub fn isolated(&self, cd: Coord) -> bool {
        let (Y(y), X(x)) = cd;
        !(self.right((Y(y), X(x - 1))) || self.right((Y(y), X(x))) || self.down((Y(y - 1), X(x))) || self.down((Y(y), X(x))))
    }
}

pub struct AnswerDetail {
    pub answers: Vec<LinePlacement>,
    pub fully_checked: bool,
    pub n_steps: u64,
}
impl AnswerDetail {
    pub fn len(&self) -> usize {
        self.answers.len()
    }
}
impl Index<usize> for AnswerDetail {
    type Output = LinePlacement;
    fn index(&self, idx: usize) -> &LinePlacement {
        &self.answers[idx]
    }
}
