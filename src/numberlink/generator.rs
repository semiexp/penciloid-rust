use super::super::{Y, X, Coord, Grid};
use super::*;

extern crate rand;

use rand::{Rng, distributions};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endpoint {
    Any,
    Forced,
    Prohibited,
}

pub struct GeneratorOption<'a> {
    pub chain_threshold: i32,
    pub endpoint_constraint: Option<&'a Grid<Endpoint>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edge {
    Undecided,
    Line,
    Blank,
}

#[derive(Clone)]
struct AnswerField<'a> {
    height: i32,
    width: i32,
    chain_union: Grid<usize>, // height * width
    chain_length: Grid<i32>, // height * width
    field: Grid<Edge>, // (2 * height - 1) * (2 * width - 1)
    seed_idx: Grid<i32>,
    seeds: Vec<Coord>,
    endpoint_constraint: Option<&'a Grid<Endpoint>>,
    endpoints: i32,
    chain_threshold: i32,
    invalid: bool,
}

impl<'a> AnswerField<'a> {
    fn new(height: i32, width: i32, endpoint_constraint: Option<&'a Grid<Endpoint>>) -> AnswerField<'a> {
        let mut ret = AnswerField {
            height: height,
            width: width,
            chain_union: Grid::new(height, width, 0),
            chain_length: Grid::new(height, width, 0),
            field: Grid::new(2 * height - 1, 2 * width - 1, Edge::Undecided),
            seed_idx: Grid::new(2 * height - 1, 2 * width - 1, -1),
            seeds: vec![],
            endpoint_constraint,
            endpoints: 0,
            chain_threshold: 3,
            invalid: false,
        };

        for idx in 0..((height * width) as usize) {
            ret.chain_union[idx] = idx;
        }

        ret.seeds.push((Y(0), X(0)));
        ret.seeds.push((Y(0), X(2 * width - 2)));
        ret.seeds.push((Y(2 * height - 2), X(0)));
        ret.seeds.push((Y(2 * height - 2), X(2 * width - 2)));
        ret.seed_idx[(Y(0), X(0))] = 0;
        ret.seed_idx[(Y(0), X(2 * width - 2))] = 1;
        ret.seed_idx[(Y(2 * height - 2), X(0))] = 2;
        ret.seed_idx[(Y(2 * height - 2), X(2 * width - 2))] = 3;
        ret
    }
    fn get(&self, cd: Coord) -> Edge {
        if self.field.is_valid_coord(cd) {
            self.field[cd]
        } else {
            Edge::Blank
        }
    }
    fn set_threshold(&mut self, threshold: i32) {
        self.chain_threshold = threshold;
    }
    fn get_threshold(&self) -> i32 {
        self.chain_threshold
    }
    fn endpoint_constraint(&self, cd: Coord) -> Endpoint {
        match self.endpoint_constraint {
            Some(g) => g[cd],
            None => Endpoint::Any,
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
    /// Returns whether vertex `cd` is a seed
    fn is_seed(&self, cd: Coord) -> bool {
        let nb = self.count_neighbor(cd);
        nb == (0, 2) || (nb.0 == 1 && nb.1 > 0)
    }
    fn num_seeds(&self) -> usize { self.seeds.len() }

    /// Returns whether there is at least one seed
    fn has_seed(&self) -> bool {
        self.seeds.len() != 0
    }
    /// Returns a random seed using `rng`
    fn random_seed<R: Rng>(&self, rng: &mut R) -> Coord {
        let idx = rng.gen_range(0, self.seeds.len());
        self.seeds[idx]
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

        let (Y(y), X(x)) = cd;

        // update chain information
        if state == Edge::Line {
            let end1 = (Y(y / 2), X(x / 2));
            let end2 = (Y((y + 1) / 2), X((x + 1) / 2));

            let end1_id = self.chain_union.index(end1);
            let end2_id = self.chain_union.index(end2);
            let another_end1_id = self.chain_union[end1_id];
            let another_end2_id = self.chain_union[end2_id];

            if another_end1_id == end2_id {
                // invalid: a self-loop will be formed
                self.invalid = true;
                return;
            }

            let new_length = self.chain_length[end1_id] + self.chain_length[end2_id] + 1;

            self.chain_union[another_end1_id] = another_end2_id;
            self.chain_union[another_end2_id] = another_end1_id;
            self.chain_length[another_end1_id] = new_length;
            self.chain_length[another_end2_id] = new_length;

            if new_length < self.chain_threshold {
                let cd = self.chain_union.coord(another_end1_id);
                self.extend_chain(cd);
            }
        }

        // check incident vertices
        if y % 2 == 1 {
            if self.count_neighbor((Y(y - 1), X(x))) == (1, 0) { self.endpoints += 1; }
            if self.count_neighbor((Y(y + 1), X(x))) == (1, 0) { self.endpoints += 1; }
            self.inspect((Y(y - 1), X(x)));
            self.inspect((Y(y + 1), X(x)));
        } else {
            if self.count_neighbor((Y(y), X(x - 1))) == (1, 0) { self.endpoints += 1; }
            if self.count_neighbor((Y(y), X(x + 1))) == (1, 0) { self.endpoints += 1; }
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
        } else if line == 1 {
            // avoid too short chains
            if self.chain_length[(Y(y / 2), X(x / 2))] < self.chain_threshold {
                self.extend_chain((Y(y / 2), X(x / 2)));

                let (Y(ay), X(ax)) = self.chain_union.coord(self.chain_union[(Y(y / 2), X(x / 2))]);
                if self.count_neighbor((Y(ay * 2), X(ax * 2))) == (1, 0) {
                    let minimum_len = self.chain_threshold - self.chain_length[(Y(y / 2), X(x / 2))];
                    for &(dy, dx) in &dirs {
                        if self.get((Y(y + dy), X(x + dx))) == Edge::Undecided {
                            let (Y(ay), X(ax)) = self.chain_union.coord(self.chain_union[(Y(y / 2 + dy), X(x / 2 + dx))]);
                            if self.count_neighbor((Y(ay * 2), X(ax * 2))) == (1, 0) && self.chain_length[(Y(y / 2 + dy), X(x / 2 + dx))] < minimum_len {
                                self.decide((Y(y + dy), X(x + dx)), Edge::Blank);
                            }
                        }
                    }
                }
            }
        }

        match self.endpoint_constraint((Y(y / 2), X(x / 2))) {
            Endpoint::Any => (),
            Endpoint::Forced => {
                if line == 1 {
                    for &(dy, dx) in &dirs {
                        let e = self.get((Y(y + dy), X(x + dx)));
                        if e == Edge::Undecided {
                            self.decide((Y(y + dy), X(x + dx)), Edge::Blank);
                        }
                    }
                } else if line >= 2 {
                    self.invalid = true;
                }
            },
            Endpoint::Prohibited => {
                if line == 1 {
                    if undecided == 0 {
                        self.invalid = true;
                        return;
                    } else if undecided == 1 {
                        for &(dy, dx) in &dirs {
                            let e = self.get((Y(y + dy), X(x + dx)));
                            if e == Edge::Undecided {
                                self.decide((Y(y + dy), X(x + dx)), Edge::Line);
                            }
                        }
                    }
                }
            },
        }

        let is_seed = self.is_seed((Y(y), X(x)));
        let seed_idx = self.seed_idx[(Y(y), X(x))];

        if seed_idx != -1 && !is_seed {
            // (y, x) is no longer a seed
            let moved = self.seeds[self.seeds.len() - 1];
            self.seed_idx[moved] = seed_idx;
            self.seeds.swap_remove(seed_idx as usize);
            self.seed_idx[(Y(y), X(x))] = -1;
        } else if seed_idx == -1 && is_seed {
            // (y, x) is now a seed
            self.seed_idx[(Y(y), X(x))] = self.seeds.len() as i32;
            self.seeds.push((Y(y), X(x)));
        }
    }
    /// Extend the chain one of whose endpoint is `(y, x)`
    fn extend_chain(&mut self, (Y(y), X(x)): Coord) {
        let end1_id = self.chain_union.index((Y(y), X(x)));
        let end2_id = self.chain_union[end1_id];

        let end1 = (Y(y * 2), X(x * 2));
        let (Y(y2), X(x2)) = self.chain_union.coord(end2_id);
        let end2 = (Y(y2 * 2), X(x2 * 2));

        let end1_undecided = self.undecided_neighbors(end1);
        let end2_undecided = self.undecided_neighbors(end2);

        match (end1_undecided.len(), end2_undecided.len()) {
            (0, 0) => {
                self.invalid = true;
                return;
            },
            (0, 1) => self.decide(end2_undecided[0], Edge::Line),
            (1, 0) => self.decide(end1_undecided[0], Edge::Line),
            _ => (),
        }
    }
}

pub fn generate_placement_beam<R: Rng>(height: i32, width: i32, opt: &GeneratorOption, rng: &mut R) -> Option<Grid<Clue>> {
    let beam_width = 100;
    let mut fields = vec![AnswerField::new(height, width, opt.endpoint_constraint)];
    fields[0].set_threshold(opt.chain_threshold);

    loop {
        if fields.len() == 0 { return None; }

        let mut fields_next = vec![];
        'outer: for _ in 0..(5 * fields.len()) {
            if fields_next.len() >= beam_width { break; }

            let id = rng.gen_range(0, fields.len());
            let mut field = fields[id].clone();

            if !field.has_seed() { continue; }
            let cd = field.random_seed(rng);
            let (Y(y), X(x)) = cd;
            let cd_vtx = (Y(y / 2), X(x / 2));

            if field.count_neighbor(cd) == (0, 2) {
                let nbs = field.undecided_neighbors(cd);
                let constraint = field.endpoint_constraint(cd_vtx);
                if (constraint != Endpoint::Forced && rng.next_f64() < 0.9f64) || constraint == Endpoint::Prohibited {
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

            if field.invalid { continue; }
            if field.num_seeds() == 0 {
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
                    if line_len[i as usize] <= 3 { continue 'outer; }
                }

                let mut end_count = vec![0; id as usize];
                for y in 0..height {
                    for x in 0..width {
                        if field.count_neighbor((Y(y * 2), X(x * 2))) == (1, 0) {
                            end_count[ids[(Y(y), X(x))] as usize] += 1;
                        }
                    }
                }
                for i in 1..id {
                    if end_count[i as usize] != 2 { continue 'outer; }
                }

                for y in 0..(2 * height - 1) {
                    for x in 0..(2 * width - 1) {
                        if y % 2 == 1 && x % 2 == 0 {
                            if (ids[(Y(y / 2), X(x / 2))] == ids[(Y(y / 2 + 1), X(x / 2))]) != (field.get((Y(y), X(x))) == Edge::Line) { continue 'outer; }
                        } else if y % 2 == 0 && x % 2 == 1 {
                            if (ids[(Y(y / 2), X(x / 2))] == ids[(Y(y / 2), X(x / 2 + 1))]) != (field.get((Y(y), X(x))) == Edge::Line) { continue 'outer; }
                        }
                    }
                }

                let mut ret = Grid::new(height, width, NO_CLUE);
                for y in 0..height {
                    for x in 0..width {
                        if field.count_neighbor((Y(y * 2), X(x * 2))) == (1, 0) {
                            ret[(Y(y), X(x))] = Clue(ids[(Y(y), X(x))]);
                        }
                    }
                }

                return Some(ret);
            }

            fields_next.push(field);
        }

        fields = fields_next;
    }
    None
}

pub fn generate_placement<R: Rng>(height: i32, width: i32, opt: &GeneratorOption, rng: &mut R) -> Option<Grid<Clue>> {
    let mut field = AnswerField::new(height, width, opt.endpoint_constraint);
    field.set_threshold(opt.chain_threshold);

    loop {
        if !field.has_seed() { break; }
        let cd = field.random_seed(rng);
        let (Y(y), X(x)) = cd;
        let cd_vtx = (Y(y / 2), X(x / 2));

        if field.count_neighbor(cd) == (0, 2) {
            let nbs = field.undecided_neighbors(cd);
            let constraint = field.endpoint_constraint(cd_vtx);
            if (constraint != Endpoint::Forced && rng.next_f64() < 0.9f64) || constraint == Endpoint::Prohibited {
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

        if field.invalid { return None; }
    }

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
        if line_len[i as usize] <= 3 { return None; }
    }

    let mut end_count = vec![0; id as usize];
    for y in 0..height {
        for x in 0..width {
            if field.count_neighbor((Y(y * 2), X(x * 2))) == (1, 0) {
                end_count[ids[(Y(y), X(x))] as usize] += 1;
            }
        }
    }
    for i in 1..id {
        if end_count[i as usize] != 2 { return None; }
    }

    for y in 0..(2 * height - 1) {
        for x in 0..(2 * width - 1) {
            if y % 2 == 1 && x % 2 == 0 {
                if (ids[(Y(y / 2), X(x / 2))] == ids[(Y(y / 2 + 1), X(x / 2))]) != (field.get((Y(y), X(x))) == Edge::Line) { return None; }
            } else if y % 2 == 0 && x % 2 == 1 {
                if (ids[(Y(y / 2), X(x / 2))] == ids[(Y(y / 2), X(x / 2 + 1))]) != (field.get((Y(y), X(x))) == Edge::Line) { return None; }
            }
        }
    }

    let mut ret = Grid::new(height, width, NO_CLUE);
    for y in 0..height {
        for x in 0..width {
            if field.count_neighbor((Y(y * 2), X(x * 2))) == (1, 0) {
                ret[(Y(y), X(x))] = Clue(ids[(Y(y), X(x))]);
            }
        }
    }

    Some(ret)
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
