use std::ops::Index;

mod solver;
mod solver2;
mod generator;
mod format;

pub use self::solver::*;
pub use self::solver2::*;
pub use self::generator::*;
pub use self::format::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Clue(pub i32);

pub const NO_CLUE: Clue = Clue(0);
pub const UNUSED: Clue = Clue(-1);

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
    pub fn height(&self) -> i32 {
        self.right.height()
    }
    pub fn width(&self) -> i32 {
        self.down.width()
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
    pub fn get(&self, cd: Coord) -> bool {
        let (Y(y), X(x)) = cd;
        match (y % 2, x % 2) {
            (0, 1) => self.right((Y(y / 2), X(x / 2))),
            (1, 0) => self.down((Y(y / 2), X(x / 2))),
            _ => panic!(),
        }
    }
    pub fn get_checked(&self, cd: Coord) -> bool {
        let (Y(y), X(x)) = cd;
        if 0 <= y && y < self.height() * 2 - 1 && 0 <= x && x < self.width() * 2 - 1 {
            self.get(cd)
        } else {
            false
        }
    }
    pub fn isolated(&self, cd: Coord) -> bool {
        let (Y(y), X(x)) = cd;
        !(self.right((Y(y), X(x - 1))) || self.right((Y(y), X(x))) || self.down((Y(y - 1), X(x))) || self.down((Y(y), X(x))))
    }
    pub fn is_endpoint(&self, cd: Coord) -> bool {
        let (Y(y), X(x)) = cd;
        let n_lines =
              if self.get_checked((Y(y * 2 + 0), X(x * 2 - 1))) { 1 } else { 0 }
            + if self.get_checked((Y(y * 2 - 1), X(x * 2 + 0))) { 1 } else { 0 }
            + if self.get_checked((Y(y * 2 + 0), X(x * 2 + 1))) { 1 } else { 0 }
            + if self.get_checked((Y(y * 2 + 1), X(x * 2 + 0))) { 1 } else { 0 };
        n_lines == 1
    }
    pub fn extract_chain_groups(&self) -> Option<Grid<i32>> {
        let height = self.height();
        let width = self.width();
        let mut ids = Grid::new(height, width, -1);
        let mut last_id = 0;

        let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];

        for y in 0..height {
            for x in 0..width {
                if self.is_endpoint((Y(y), X(x))) && ids[(Y(y), X(x))] == -1 {
                    // traverse chain
                    let (mut ly, mut lx) = (-1, -1);
                    let (mut cy, mut cx) = (y, x);
                    'traverse: loop {
                        ids[(Y(cy), X(cx))] = last_id;
                        for d in 0..4 {
                            let (dy, dx) = dirs[d];

                            if (cy + dy, cx + dx) != (ly, lx) && self.get_checked((Y(cy * 2 + dy), X(cx * 2 + dx))) {
                                ly = cy; lx = cx;
                                cy += dy; cx += dx;
                                continue 'traverse;
                            }
                        }
                        break;
                    }
                    last_id += 1;
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                if ids[(Y(y), X(x))] == -1 { return None; }
                if y < height - 1 && (ids[(Y(y), X(x))] == ids[(Y(y + 1), X(x))]) != self.down((Y(y), X(x))) { return None; }
                if x < width - 1 && (ids[(Y(y), X(x))] == ids[(Y(y), X(x + 1))]) != self.right((Y(y), X(x))) { return None; }
            }
        }

        Some(ids)
    }
}

pub struct AnswerDetail {
    pub answers: Vec<LinePlacement>,
    pub fully_checked: bool,
    pub found_not_fully_filled: bool,
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
