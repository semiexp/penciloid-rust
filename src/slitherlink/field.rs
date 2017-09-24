use super::super::{Y, X, Coord, Grid, Edge, GridLoop, GridLoopField};
use super::*;

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
    pub fn check_all_cell(&mut self) {
        let height = self.height();
        let width = self.width();
        let mut handle = GridLoop::get_handle(self);
        for y in 0..height {
            for x in 0..width {
                let clue = handle.get_clue((Y(y), X(x)));
                if clue != NO_CLUE {
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
            GridLoop::check(self, cd);
        }
    }
    pub fn get_edge(&mut self, cd: Coord) -> Edge {
        self.grid_loop.get_edge(cd)
    }
    pub fn get_edge_safe(&mut self, cd: Coord) -> Edge {
        self.grid_loop.get_edge_safe(cd)
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
            if clue == NO_CLUE { return; }

            let mut neighbors = [Edge::Undecided; DICTIONARY_NEIGHBOR_SIZE];
            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                let (Y(dy), X(dx)) = DICTIONARY_EDGE_OFFSET[i];
                neighbors[i] = self.grid_loop.get_edge_safe((Y(y + dy), X(x + dx)));
            }

            if self.dic.consult(clue, &mut neighbors) {
                self.grid_loop.set_inconsistent();
                return;
            }

            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                if neighbors[i] != Edge::Undecided {
                    let (Y(dy), X(dx)) = DICTIONARY_EDGE_OFFSET[i];
                    GridLoop::decide_edge(self, (Y(y + dy), X(x + dx)), neighbors[i]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common;

    fn run_problem_test(input: &[&str], dic: &Dictionary) {
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
        
        run_problem_test(&[
            "+x+-+-+ +",
            "x | x    ",
            "+x+-+x+ +",
            "x0x3|    ",
            "+x+-+x+ +",
            "x | x    ",
            "+x+-+-+ +",
        ], &dic);
    }
}
