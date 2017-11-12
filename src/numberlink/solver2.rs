use super::super::{Y, X, Grid};
use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Edge {
    Undecided,
    Line,
    Blank,
}

enum History {
    AnotherEnd(i32, i32),
    Edge(Coord),
    Inconsistent(bool),
    Checkpoint,
}

struct SolverField {
    another_end: Grid<i32>, // height * width
    has_clue: Grid<bool>, // height * width
    edge: Grid<Edge>, // (2 * height - 1) * (2 * width - 1)
    inconsistent: bool,
    history: Vec<History>,
}

const CLOSED_END: i32 = -1;

impl SolverField {
    fn new(problem: &Grid<Clue>) -> SolverField {
        let height = problem.height();
        let width = problem.width();
        let mut another_end = Grid::new(height, width, 0);
        let mut has_clue = Grid::new(height, width, false);
        for y in 0..height {
            for x in 0..width {
                let c = problem[(Y(y), X(x))];
                if c == NO_CLUE {
                    let id = another_end.index((Y(y), X(x))) as i32;
                    another_end[(Y(y), X(x))] = id;
                } else {
                    another_end[(Y(y), X(x))] = -(c.0 + 1);
                    has_clue[(Y(y), X(x))] = true;
                }
            }
        }
        SolverField {
            another_end,
            has_clue,
            edge: Grid::new(height * 2 - 1, width * 2 - 1, Edge::Undecided),
            inconsistent: false,
            history: Vec::new(),
        }
    }
    fn get_edge(&self, cd: Coord) -> Edge {
        if self.edge.is_valid_coord(cd) {
            self.edge[cd]
        } else {
            Edge::Blank
        }
    }
    fn height(&self) -> i32 { self.another_end.height() }
    fn width(&self) -> i32 { self.another_end.width() }
    fn get_line_placement(&self) -> LinePlacement {
        let height = self.height();
        let width = self.width();
        let mut ret = LinePlacement::new(height, width);
        for y in 0..height {
            for x in 0..width {
                if y != height - 1 && self.get_edge((Y(y * 2 + 1), X(x * 2))) == Edge::Line {
                    ret.set_down((Y(y), X(x)), true);
                }
                if x != width - 1 && self.get_edge((Y(y * 2), X(x * 2 + 1))) == Edge::Line {
                    ret.set_right((Y(y), X(x)), true);
                }
            }
        }
        ret
    }

    fn set_inconsistent(&mut self) -> bool {
        self.history.push(History::Inconsistent(self.inconsistent));
        self.inconsistent = true;
        return true;
    }
    fn update_another_end(&mut self, id: i32, value: i32) {
        self.history.push(History::AnotherEnd(id, self.another_end[id as usize]));
        self.another_end[id as usize] = value;
    }
    /// Add an checkpoint.
    fn add_checkpoint(&mut self) {
        self.history.push(History::Checkpoint);
    }
    /// Rollback until the last checkpoint.
    fn rollback(&mut self) {
        while let Some(entry) = self.history.pop() {
            match entry {
                History::AnotherEnd(id, val) => self.another_end[id as usize] = val,
                History::Edge(cd) => self.edge[cd] = Edge::Undecided,
                History::Inconsistent(ic) => self.inconsistent = ic,
                History::Checkpoint => break,
            }
        }
    }
    /// Decide edge `cd`.
    /// `cd` must be in universal-coordination.
    fn decide_edge(&mut self, cd: Coord, state: Edge) -> bool {
        let current_state = self.get_edge(cd);
        if current_state != Edge::Undecided {
            if current_state != state {
                self.inconsistent = true;
                return true;
            }
            return false;
        }

        let (Y(y), X(x)) = cd;

        // update endpoints or detect inconsistency
        let end1;
        let end2;
        if y % 2 == 0 {
            end1 = (Y(y / 2), X(x / 2));
            end2 = (Y(y / 2), X(x / 2 + 1));
        } else {
            end1 = (Y(y / 2), X(x / 2));
            end2 = (Y(y / 2 + 1), X(x / 2));
        }
        let end1_id = self.another_end.index(end1) as i32;
        let end2_id = self.another_end.index(end2) as i32;

        if state == Edge::Line {
            let another_end1_id = self.another_end[end1];
            let another_end2_id = self.another_end[end2];

            // connecting closed ends / closing single chain
            if another_end1_id == CLOSED_END || another_end2_id == CLOSED_END || another_end1_id == end2_id {
                return self.set_inconsistent();
            }
            match (another_end1_id < 0, another_end2_id < 0) {
                (true, true) => {
                    if another_end1_id == another_end2_id {
                        self.update_another_end(end1_id, CLOSED_END);
                        self.update_another_end(end2_id, CLOSED_END);
                    } else {
                        return self.set_inconsistent();
                    }
                },
                (false, true) => {
                    if end1_id != another_end1_id {
                        self.update_another_end(end1_id, CLOSED_END);
                    }
                    self.update_another_end(another_end1_id, another_end2_id);
                    self.update_another_end(end2_id, CLOSED_END);
                },
                (true, false) => {
                    if end2_id != another_end2_id {
                        self.update_another_end(end2_id, CLOSED_END);
                    }
                    self.update_another_end(another_end2_id, another_end1_id);
                    self.update_another_end(end1_id, CLOSED_END);
                },
                (false, false) => {
                    if end1_id != another_end1_id {
                        self.update_another_end(end1_id, CLOSED_END);
                    }
                    self.update_another_end(another_end1_id, another_end2_id);
                    if end2_id != another_end2_id {
                        self.update_another_end(end2_id, CLOSED_END);
                    }
                    self.update_another_end(another_end2_id, another_end1_id);
                }
            }
        }

        // update edge state
        self.history.push(History::Edge(cd));
        self.edge[cd] = state;

        // ensure canonical form
        if state == Edge::Line {
            if y % 2 == 0 {
                if self.get_edge((Y(y - 2), X(x))) == Edge::Line {
                    if self.decide_edge((Y(y - 1), X(x - 1)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y - 1), X(x + 1)), Edge::Blank) { return true; }
                } else if self.get_edge((Y(y - 1), X(x - 1))) == Edge::Line {
                    if self.decide_edge((Y(y - 2), X(x)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y - 1), X(x + 1)), Edge::Blank) { return true; }
                } else if self.get_edge((Y(y - 1), X(x + 1))) == Edge::Line {
                    if self.decide_edge((Y(y - 2), X(x)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y - 1), X(x - 1)), Edge::Blank) { return true; }
                }

                if self.get_edge((Y(y + 2), X(x))) == Edge::Line {
                    if self.decide_edge((Y(y + 1), X(x - 1)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y + 1), X(x + 1)), Edge::Blank) { return true; }
                } else if self.get_edge((Y(y + 1), X(x - 1))) == Edge::Line {
                    if self.decide_edge((Y(y + 2), X(x)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y + 1), X(x + 1)), Edge::Blank) { return true; }

                    // yielding L-chain
                    if !self.has_clue[(Y(y / 2 + 1), X(x / 2 + 1))] {
                        if self.decide_edge((Y(y + 2), X(x + 2)), Edge::Line) { return true; }
                        if self.decide_edge((Y(y + 3), X(x + 1)), Edge::Line) { return true; }
                    }
                } else if self.get_edge((Y(y + 1), X(x + 1))) == Edge::Line {
                    if self.decide_edge((Y(y + 2), X(x)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y + 1), X(x - 1)), Edge::Blank) { return true; }

                    // yielding L-chain
                    if !self.has_clue[(Y(y / 2 + 1), X(x / 2))] {
                        if self.decide_edge((Y(y + 2), X(x - 2)), Edge::Line) { return true; }
                        if self.decide_edge((Y(y + 3), X(x - 1)), Edge::Line) { return true; }
                    }
                }
            } else {
                if self.get_edge((Y(y), X(x - 2))) == Edge::Line {
                    if self.decide_edge((Y(y - 1), X(x - 1)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y + 1), X(x - 1)), Edge::Blank) { return true; }
                } else if self.get_edge((Y(y - 1), X(x - 1))) == Edge::Line {
                    if self.decide_edge((Y(y), X(x - 2)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y + 1), X(x - 1)), Edge::Blank) { return true; }

                    // yielding L-chain
                    if !self.has_clue[(Y(y / 2 + 1), X(x / 2 - 1))] {
                        if self.decide_edge((Y(y + 1), X(x - 3)), Edge::Line) { return true; }
                        if self.decide_edge((Y(y + 2), X(x - 2)), Edge::Line) { return true; }
                    }
                } else if self.get_edge((Y(y + 1), X(x - 1))) == Edge::Line {
                    if self.decide_edge((Y(y), X(x - 2)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y - 1), X(x - 1)), Edge::Blank)  { return true; }
                }

                if self.get_edge((Y(y), X(x + 2))) == Edge::Line {
                    if self.decide_edge((Y(y - 1), X(x + 1)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y + 1), X(x + 1)), Edge::Blank) { return true; }
                } else if self.get_edge((Y(y - 1), X(x + 1))) == Edge::Line {
                    if self.decide_edge((Y(y), X(x + 2)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y + 1), X(x + 1)), Edge::Blank) { return true; }

                    // yielding L-chain
                    if !self.has_clue[(Y(y / 2 + 1), X(x / 2 + 1))] {
                        if self.decide_edge((Y(y + 1), X(x + 3)), Edge::Line) { return true; }
                        if self.decide_edge((Y(y + 2), X(x + 2)), Edge::Line) { return true; }
                    }
                } else if self.get_edge((Y(y + 1), X(x + 1))) == Edge::Line {
                    if self.decide_edge((Y(y), X(x + 2)), Edge::Blank) { return true; }
                    if self.decide_edge((Y(y - 1), X(x + 1)), Edge::Blank) { return true; }
                }
            }
        }

        // check incident vertices
        if self.inspect(end1) { return true; }
        if self.inspect(end2) { return true; }

        return false;
    }

    /// Inspect vertex `cd`.
    /// `cd` must be in vertex-coordination.
    fn inspect(&mut self, cd: Coord) -> bool {
        let (Y(y), X(x)) = cd;

        let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        let mut n_line = if self.has_clue[cd] { 1 } else { 0 };
        let mut n_undecided = 0;
        for &(dy, dx) in &dirs {
            match self.get_edge((Y(y * 2 + dy), X(x * 2 + dx))) {
                Edge::Undecided => n_undecided += 1,
                Edge::Line => n_line += 1,
                Edge::Blank => (),
            }
        }

        if n_line >= 3 {
            return self.set_inconsistent();
        }
        if n_line == 2 {
            for &(dy, dx) in &dirs {
                let cd2 = (Y(y * 2 + dy), X(x * 2 + dx));
                if self.get_edge(cd2) == Edge::Undecided {
                    if self.decide_edge(cd2, Edge::Blank) { return true; }
                }
            }
        } else if n_line == 1 {
            if n_undecided == 1 {
                for &(dy, dx) in &dirs {
                    let cd2 = (Y(y * 2 + dy), X(x * 2 + dx));
                    if self.get_edge(cd2) == Edge::Undecided {
                        if self.decide_edge(cd2, Edge::Line) { return true; }
                    }
                }
            } else if n_undecided == 0 {
                return self.set_inconsistent();
            }
        }
        false
    }
}

pub fn solve2(problem: &Grid<Clue>) -> Vec<LinePlacement> {
    let height = problem.height();
    let width = problem.width();
    let mut solver_field = SolverField::new(problem);
    let mut answers = vec![];

    search(0, 0, &mut solver_field, &mut answers);

    answers
}

fn search(y: i32, x: i32, field: &mut SolverField, answers: &mut Vec<LinePlacement>) {
    let mut y = y;
    let mut x = x;
    if x == field.width() {
        y += 1;
        x = 0;
    }
    while y < field.height() && field.get_edge((Y(y * 2 + 1), X(x * 2))) != Edge::Undecided && field.get_edge((Y(y * 2), X(x * 2 + 1))) != Edge::Undecided {
        if x == field.width() - 1 {
            y += 1;
            x = 0;
        } else {
            x += 1;
        }
    }

    if y == field.height() {
        // answer found
        answers.push(field.get_line_placement());
        return;
    }

    let degree_common = if field.has_clue[(Y(y), X(x))] { 1 } else { 0 }
            + if field.get_edge((Y(y * 2), X(x * 2 - 1))) == Edge::Line { 1 } else { 0 }
            + if field.get_edge((Y(y * 2), X(x * 2 + 1))) == Edge::Line { 1 } else { 0 }
            + if field.get_edge((Y(y * 2 - 1), X(x * 2))) == Edge::Line { 1 } else { 0 }
            + if field.get_edge((Y(y * 2 + 1), X(x * 2))) == Edge::Line { 1 } else { 0 };

    for mask in 0..4 {
        let right = (mask & 1) != 0;
        let down = (mask & 2) != 0;

        if right && field.get_edge((Y(y * 2), X(x * 2 + 1))) != Edge::Undecided { continue; }
        if down && field.get_edge((Y(y * 2 + 1), X(x * 2))) != Edge::Undecided { continue; }

        let degree = degree_common + if right { 1 } else { 0 } + if down { 1 } else { 0 };
        if degree != 0 && degree != 2 { continue; }

        let right_effective = right || (field.get_edge((Y(y * 2), X(x * 2 + 1))) == Edge::Line);
        let down_effective = down || (field.get_edge((Y(y * 2 + 1), X(x * 2))) == Edge::Line);
        if right_effective && down_effective {
            let mut isok = false;
            let mut i = 1;
            while y + i < field.height() && x + i < field.width() {
                if field.has_clue[(Y(y + i), X(x + i))] {
                    isok = true;
                    break;
                }
                i += 1;
            }
            if !isok { continue; }
        }
        if right_effective && field.get_edge((Y(y * 2 + 1), X(x * 2 + 2))) == Edge::Line {
            let mut isok = false;
            let mut i = 1;
            while y + i < field.height() && x + 1 - i >= 0 {
                if field.has_clue[(Y(y + i), X(x + 1 - i))] {
                    isok = true;
                    break;
                }
                i += 1;
            }
            if !isok { continue; }
        }

        field.add_checkpoint();
        let mut inconsistent = false;

        inconsistent |= field.decide_edge((Y(y * 2), X(x * 2 + 1)), if right { Edge::Line } else { Edge::Blank });
        if !inconsistent {
            inconsistent |= field.decide_edge((Y(y * 2 + 1), X(x * 2)), if down { Edge::Line } else { Edge::Blank });
        }

        if !inconsistent {
            search(y, x + 1, field, answers);
        }

        field.rollback();
    }
}
