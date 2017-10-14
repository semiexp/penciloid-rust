use super::super::{Y, X, Coord, Grid};
use grid_loop::{Edge, GridLoop, GridLoopField};
use super::*;

#[derive(Clone)]
pub struct Field<'a> {
    grid_loop: GridLoop,
    clue: Grid<Clue>,
    dic: &'a Dictionary,
}
impl<'a> Field<'a> {
    pub fn new(clue: &Grid<Clue>, dic: &'a Dictionary) -> Field<'a> {
        let grid_loop = GridLoop::new(clue.height(), clue.width());

        Field {
            grid_loop: grid_loop,
            clue: clue.clone(),
            dic: dic,
        }
    }
    pub fn height(&self) -> i32 {
        self.clue.height()
    }
    pub fn width(&self) -> i32 {
        self.clue.width()
    }
    pub fn inconsistent(&self) -> bool {
        self.grid_loop.inconsistent()
    }
    pub fn set_inconsistent(&mut self) {
        self.grid_loop.set_inconsistent()
    }
    pub fn fully_solved(&self) -> bool {
        self.grid_loop.fully_solved()
    }
    pub fn check_all_cell(&mut self) {
        let height = self.height();
        let width = self.width();
        let mut handle = GridLoop::get_handle(self);
        for y in 0..height {
            for x in 0..width {
                let clue = handle.get_clue((Y(y), X(x)));
                if clue != NO_CLUE {
                    handle.inspect_technique((Y(y * 2 + 1), X(x * 2 + 1)));
                    GridLoop::check(&mut *handle, (Y(y * 2 + 1), X(x * 2 + 1)));
                }
            }
        }
    }
    pub fn get_clue(&self, cd: Coord) -> Clue {
        self.clue[cd]
    }
    pub fn add_clue(&mut self, cd: Coord, clue: Clue) {
        if self.clue[cd] != NO_CLUE {
            if self.clue[cd] != clue {
                self.grid_loop.set_inconsistent();
            }
        } else {
            self.clue[cd] = clue;

            let mut handle = GridLoop::get_handle(self);
            let (Y(y), X(x)) = cd;
            handle.inspect_technique((Y(y * 2 + 1), X(x * 2 + 1)));
            GridLoop::check(&mut *handle, (Y(y * 2 + 1), X(x * 2 + 1)));
        }
    }
    pub fn get_edge(&self, cd: Coord) -> Edge {
        self.grid_loop.get_edge(cd)
    }
    pub fn get_edge_safe(&self, cd: Coord) -> Edge {
        self.grid_loop.get_edge_safe(cd)
    }

    fn inspect_technique(&mut self, (Y(y), X(x)): Coord) {
        let neighbor = [
            (Y(1), X(0)),
            (Y(0), X(1)),
            (Y(-1), X(0)),
            (Y(0), X(-1)),
        ];

        if y % 2 == 1 && x % 2 == 1 {
            let clue = self.clue[(Y(y / 2), X(x / 2))];
            if clue == Clue(0) {
                for d in 0..4 {
                    let (Y(dy), X(dx)) = neighbor[d];
                    GridLoop::decide_edge(self, (Y(y + dy), X(x + dx)), Edge::Blank);
                }
            }
            if clue == Clue(3) {
                // adjacent 3
                for d in 0..4 {
                    let (Y(dy), X(dx)) = neighbor[d];
                    let cell2 = (Y(y / 2 + dy), X(x / 2 + dx));
                    if self.clue.is_valid_coord(cell2) && self.clue[cell2] == Clue(3) {
                        // Deriberately ignoring the possible small loop encircling the two 3's
                        GridLoop::decide_edge(self, (Y(y - dy), X(x - dx)), Edge::Line);
                        GridLoop::decide_edge(self, (Y(y + dy), X(x + dx)), Edge::Line);
                        GridLoop::decide_edge(self, (Y(y + 3 * dy), X(x + 3 * dx)), Edge::Line);
                        GridLoop::decide_edge(self, (Y(y + dy + 2 * dx), X(x + dx + 2 * dy)), Edge::Blank);
                        GridLoop::decide_edge(self, (Y(y + dy - 2 * dx), X(x + dx - 2 * dy)), Edge::Blank);
                    }
                }

                // diagonal 3
                for d in 0..4 {
                    let (Y(dy1), X(dx1)) = neighbor[d];
                    let (Y(dy2), X(dx2)) = neighbor[(d + 1) % 4];
                    let cell2 = (Y(y / 2 + dy1 + dy2), X(x / 2 + dx1 + dx2));
                    if self.clue.is_valid_coord(cell2) && self.clue[cell2] == Clue(3) {
                        GridLoop::decide_edge(self, (Y(y - dy1), X(x - dx1)), Edge::Line);
                        GridLoop::decide_edge(self, (Y(y - dy2), X(x - dx2)), Edge::Line);
                        GridLoop::decide_edge(self, (Y(y + 2 * dy1 + 3 * dy2), X(x + 2 * dx1 + 3 * dx2)), Edge::Line);
                        GridLoop::decide_edge(self, (Y(y + 3 * dy1 + 2 * dy2), X(x + 3 * dx1 + 2 * dx2)), Edge::Line);
                    }
                }
            }
        }
    }
}
impl<'a> GridLoopField for Field<'a> {
    fn grid_loop(&mut self) -> &mut GridLoop {
        &mut self.grid_loop
    }
    fn check_neighborhood(&mut self, (Y(y), X(x)): Coord) {
        if y % 2 == 1 {
            GridLoop::check(self, (Y(y - 1), X(x)));
            GridLoop::check(self, (Y(y + 1), X(x)));

            GridLoop::check(self, (Y(y), X(x - 1)));
            GridLoop::check(self, (Y(y), X(x + 1)));
            GridLoop::check(self, (Y(y - 2), X(x - 1)));
            GridLoop::check(self, (Y(y - 2), X(x + 1)));
            GridLoop::check(self, (Y(y + 2), X(x - 1)));
            GridLoop::check(self, (Y(y + 2), X(x + 1)));
        } else {
            GridLoop::check(self, (Y(y), X(x - 1)));
            GridLoop::check(self, (Y(y), X(x + 1)));

            GridLoop::check(self, (Y(y - 1), X(x)));
            GridLoop::check(self, (Y(y + 1), X(x)));
            GridLoop::check(self, (Y(y - 1), X(x - 2)));
            GridLoop::check(self, (Y(y + 1), X(x - 2)));
            GridLoop::check(self, (Y(y - 1), X(x + 2)));
            GridLoop::check(self, (Y(y + 1), X(x + 2)));
        }
    }
    fn inspect(&mut self, (Y(y), X(x)): Coord) {
        if y % 2 == 1 && x % 2 == 1 {
            let clue = self.clue[(Y(y / 2), X(x / 2))];
            if clue == NO_CLUE || clue == Clue(0) { return; }

            let mut neighbors_code = 0;
            let mut pow3 = 1;
            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                let (Y(dy), X(dx)) = DICTIONARY_EDGE_OFFSET[i];
                neighbors_code += pow3 * match self.grid_loop.get_edge_safe((Y(y + dy), X(x + dx))) {
                    Edge::Undecided => 0,
                    Edge::Line => 1,
                    Edge::Blank => 2,
                };
                pow3 *= 3;
            }

            let res = self.dic.consult_raw(clue, neighbors_code);
            if res == DICTIONARY_INCONSISTENT {
                self.grid_loop.set_inconsistent();
                return;
            }
            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                let e = (res >> (2 * i)) & 3;
                if e == 1 {
                    let (Y(dy), X(dx)) = DICTIONARY_EDGE_OFFSET[i];
                    GridLoop::decide_edge(self, (Y(y + dy), X(x + dx)), Edge::Line);
                } else if e == 2 {
                    let (Y(dy), X(dx)) = DICTIONARY_EDGE_OFFSET[i];
                    GridLoop::decide_edge(self, (Y(y + dy), X(x + dx)), Edge::Blank);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common;

    fn run_problem_test(dic: &Dictionary, input: &[&str], fully_solved: bool) {
        let height = (input.len() / 2) as i32;
        let width = (input[0].len() / 2) as i32;

        let mut clue = Grid::new(height, width, NO_CLUE);
        for y in 0..height {
            let mut row_iter = input[(y * 2 + 1) as usize].chars();

            for x in 0..width {
                row_iter.next();
                let c = row_iter.next().unwrap();
                if '0' <= c && c <= '3' {
                    clue[(Y(y), X(x))] = Clue(((c as u8) - ('0' as u8)) as i32);
                }
            }
        }

        let mut field = Field::new(&clue, dic);
        field.check_all_cell();

        assert_eq!(field.inconsistent(), false);
        assert_eq!(field.fully_solved(), fully_solved);

        for y in 0..(input.len() as i32) {
            let mut row_iter = input[y as usize].chars();

            for x in 0..(input[0].len() as i32) {
                let ch = row_iter.next().unwrap();

                if !field.grid_loop().is_edge((Y(y), X(x))) {
                    continue;
                }

                let expected_edge = match ch {
                    '|' | '-' => Edge::Line,
                    'x' => Edge::Blank,
                    _ => Edge::Undecided,
                };

                assert_eq!(field.get_edge((Y(y), X(x))), expected_edge, "Comparing at y={}, x={}", y, x);
            }
        }
    }

    #[test]
    fn test_problem() {
        let dic = Dictionary::complete();
        
        run_problem_test(&dic, &[
            "+x+-+-+ +",
            "x | x    ",
            "+x+-+x+ +",
            "x0x3|    ",
            "+x+-+x+ +",
            "x | x    ",
            "+x+-+-+ +",
        ], false);
        run_problem_test(&dic, &[
            "+x+-+x+x+",
            "x |3| x x",
            "+x+x+-+-+",
            "x | x x3|",
            "+x+-+x+-+",
            "x0x2| | x",
            "+x+x+-+x+",
        ], true);
        run_problem_test(&dic, &[
            "+-+-+-+-+",
            "|3x x x |",
            "+-+ +-+x+",
            "x     | |",
            "+x+ +x+-+",
            "x x x0x1x",
            "+x+x+x+x+",
        ], false);
        run_problem_test(&dic, &[
            "+ +-+-+x+",
            " 2  x2| x",
            "+ +x+x+-+",
            "| x x0x |",
            "+ + +x+ +",
            "         ",
            "+ + + + +",
        ], false);
        run_problem_test(&dic, &[
            "+ +-+ +x+",
            "   3   1x",
            "+x+-+x+ +",
            "   3  |  ",
            "+ +-+ + +",
            "         ",
            "+ + + + +",
        ], false);
        run_problem_test(&dic, &[
            "+-+-+ + +",
            "|2x      ",
            "+x+-+ + +",
            "| |3     ",
            "+ + + + +",
            "     3| x",
            "+ + +-+x+",
        ], false);
    }
}
