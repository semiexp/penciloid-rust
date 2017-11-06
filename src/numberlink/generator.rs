use super::super::{Y, X, Coord, Grid};
use super::*;

extern crate rand;

use rand::{Rng, distributions};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edge {
    Undecided,
    Line,
    Blank,
}

struct AnswerField {
    height: i32,
    width: i32,
    field: Grid<Edge>,
    invalid: bool,
}

impl AnswerField {
    fn new(height: i32, width: i32) -> AnswerField {
        AnswerField {
            height: height,
            width: width,
            field: Grid::new(2 * height - 1, 2 * width - 1, Edge::Undecided),
            invalid: false,
        }
    }
    fn get(&self, cd: Coord) -> Edge {
        if self.field.is_valid_coord(cd) {
            self.field[cd]
        } else {
            Edge::Blank
        }
    }
    /// Counts the number of (Line, Undecided) around `cd`
    fn count_neighbor(&self, cd: Coord) -> (i32, i32) {
        let (Y(y), X(x)) = cd;
        let mut line = 0;
        let mut undecided = 0;
        let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        for &(dy, dx) in &dirs {
            let e = self.get((Y(y + dy), X(x + dx)));
            if e == Edge::Line {
                line += 1;
            } else if e == Edge::Undecided {
                undecided += 1;
            }
        }
        (line, undecided)
    }
    /// Returns all neighbors whose state is `Undecided` around `cd`
    fn undecided_neighbors(&self, cd: Coord) -> Vec<Coord> {
        let (Y(y), X(x)) = cd;
        let mut ret = vec![];
        let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        for &(dy, dx) in &dirs {
            let cd2 = (Y(y + dy), X(x + dx));
            let e = self.get(cd2);
            if e == Edge::Undecided {
                ret.push(cd2);
            }
        }
        ret
    }
    fn decide(&mut self, cd: Coord, state: Edge) {
        let current = self.field[cd];
        if current != Edge::Undecided {
            if current != state {
                self.invalid = true;
            }
            return;
        }
        self.field[cd] = state;

        // check incident vertices
        let (Y(y), X(x)) = cd;
        if y % 2 == 1 {
            self.inspect((Y(y - 1), X(x)));
            self.inspect((Y(y + 1), X(x)));
        } else {
            self.inspect((Y(y), X(x - 1)));
            self.inspect((Y(y), X(x + 1)));
        }

        // check for canonization rule
        if state == Edge::Line {
            if y % 2 == 1 {
                let related = [
                    (Y(y), X(x - 2)),
                    (Y(y - 1), X(x - 1)),
                    (Y(y + 1), X(x - 1)),
                ];
                for i in 0..3 {
                    if self.get(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
                let related = [
                    (Y(y), X(x + 2)),
                    (Y(y - 1), X(x + 1)),
                    (Y(y + 1), X(x + 1)),
                ];
                for i in 0..3 {
                    if self.get(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
            } else {
                let related = [
                    (Y(y - 2), X(x)),
                    (Y(y - 1), X(x - 1)),
                    (Y(y - 1), X(x + 1)),
                ];
                for i in 0..3 {
                    if self.get(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
                let related = [
                    (Y(y + 2), X(x)),
                    (Y(y + 1), X(x - 1)),
                    (Y(y + 1), X(x + 1)),
                ];
                for i in 0..3 {
                    if self.get(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
            }
        }
    }
    /// Inspect vertex (y, x)
    fn inspect(&mut self, (Y(y), X(x)): Coord) {
        let (line, undecided) = self.count_neighbor((Y(y), X(x)));
        let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        if line == 0 {
            if undecided == 0 {
                self.invalid = true;
                return;
            }
            if undecided == 1 {
                for &(dy, dx) in &dirs {
                    let e = self.get((Y(y + dy), X(x + dx)));
                    if e == Edge::Undecided {
                        self.decide((Y(y + dy), X(x + dx)), Edge::Line);
                    }
                }
            }
        } else if line == 2 {
            for &(dy, dx) in &dirs {
                let e = self.get((Y(y + dy), X(x + dx)));
                if e == Edge::Undecided {
                    self.decide((Y(y + dy), X(x + dx)), Edge::Blank);
                }
            }
        }
    }
}

fn generate_placement<R: Rng>(height: i32, width: i32, rng: &mut R) {
    let mut field = AnswerField::new(height, width);

    loop {
        let mut cand = vec![];
        for y in 0..height {
            for x in 0..width {
                let y = y * 2;
                let x = x * 2;
                let (line, undecided) = field.count_neighbor((Y(y), X(x)));
                if (line == 0 && undecided == 2) || (line == 1 && undecided > 0) {
                    cand.push((Y(y), X(x)));
                }
            }
        }

        if cand.len() == 0 { break; }

        let idx = rng.gen_range(0, cand.len());
        let cd = cand[idx];

        if field.count_neighbor(cd) == (0, 2) {
            let nbs = field.undecided_neighbors(cd);
            if rng.next_f64() < 0.9f64 {
                // as angle
                field.decide(nbs[0], Edge::Line);
                field.decide(nbs[1], Edge::Line);
            } else {
                // as an endpoint
                let i = rng.gen_range(0, 2);
                field.decide(nbs[i], Edge::Line);
                field.decide(nbs[(1 - i)], Edge::Blank);
            }
        } else {
            let nbs = field.undecided_neighbors(cd);

            if rng.next_f64() < 1.0f64 {
                // extend
                let i = rng.gen_range(0, nbs.len());
                field.decide(nbs[i], Edge::Line);
            } else {
                // terminate
                for nb in nbs {
                    field.decide(nb, Edge::Blank);
                }
            }
        }
    }

    if field.invalid { return; }

    let mut ids = Grid::new(height, width, -1);
    let mut id = 1;
    for y in 0..height {
        for x in 0..width {
            if ids[(Y(y), X(x))] == -1 {
                fill_line_id((Y(y), X(x)), &field, &mut ids, id);
                id += 1;
            }
        }
    }

    let mut line_len = vec![0; id as usize];
    for y in 0..height {
        for x in 0..width {
            line_len[ids[(Y(y), X(x))] as usize] += 1;
        }
    }
    for i in 1..id {
        if line_len[i as usize] <= 3 { return; }
    }

    for y in 0..(2 * height - 1) {
        for x in 0..(2 * width - 1) {
            if y % 2 == 1 && x % 2 == 0 {
                if (ids[(Y(y / 2), X(x / 2))] == ids[(Y(y / 2 + 1), X(x / 2))]) != (field.get((Y(y), X(x))) == Edge::Line) { return; }
            } else if y % 2 == 0 && x % 2 == 1 {
                if (ids[(Y(y / 2), X(x / 2))] == ids[(Y(y / 2), X(x / 2 + 1))]) != (field.get((Y(y), X(x))) == Edge::Line) { return; }
            }
        }
    }
    for y in 0..(2 * height - 1) {
        for x in 0..(2 * width - 1) {
            match (y % 2, x % 2) {
                (0, 0) => print!("+"),
                (0, 1) => print!("{}", match field.get((Y(y), X(x))) {
                    Edge::Undecided => ' ',
                    Edge::Line => '-',
                    Edge::Blank => ' ',
                }),
                (1, 0) => print!("{}", match field.get((Y(y), X(x))) {
                    Edge::Undecided => ' ',
                    Edge::Line => '|',
                    Edge::Blank => ' ',
                }),
                (1, 1) => print!(" "),
                _ => unreachable!(),
            }
        }
        println!();
    }
    for y in 0..height {
        for x in 0..width {
            if field.count_neighbor((Y(y * 2), X(x * 2))) == (1, 0) {
                print!("{:2} ", ids[(Y(y), X(x))]);
            } else {
                print!(".. ");
            }
        }
        println!();
    }
    println!("----------------");
}
fn fill_line_id(cd: Coord, field: &AnswerField, ids: &mut Grid<i32>, id: i32) {
    if ids[cd] != -1 { return; }
    ids[cd] = id;
    let (Y(y), X(x)) = cd;

    let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    for &(dy, dx) in &dirs {
        if field.get((Y(y * 2 + dy), X(x * 2 + dx))) == Edge::Line {
            fill_line_id((Y(y + dy), X(x + dx)), field, ids, id);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placement_generator() {
        let mut rng = ::rand::thread_rng();
        println!();
        for _ in 0..1000 {
            generate_placement(10, 18, &mut rng);
        }
        assert!(false);
    }
}
