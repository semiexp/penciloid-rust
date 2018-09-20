use super::super::{Coord, FiniteSearchQueue, Grid, Symmetry, X, Y};
use super::*;

extern crate rand;

use rand::Rng;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Undecided,
    Line,
    Blank,
}

#[derive(Clone)]
pub struct AnswerField {
    height: i32,
    width: i32,
    chain_union: Grid<usize>,      // height * width
    chain_connectivity: Grid<i32>, // height * width
    chain_length: Grid<i32>,       // height * width
    field: Grid<Edge>,             // (2 * height - 1) * (2 * width - 1)
    seed_idx: Grid<i32>,
    seeds: Vec<Coord>,
    seed_count: usize,
    endpoint_constraint: Grid<Endpoint>,
    endpoints: i32,
    endpoint_forced_cells: i32,
    chain_threshold: i32,
    forbid_adjacent_clue: bool,
    symmetry: Symmetry,
    invalid: bool,
    search_queue: FiniteSearchQueue,
}

#[derive(PartialEq, Eq)]
enum Cnt<T> {
    None,
    One(T),
    Many,
}

impl AnswerField {
    pub fn new(height: i32, width: i32, opt: &GeneratorOption) -> AnswerField {
        let mut ret = AnswerField {
            height,
            width,
            chain_union: Grid::new(height, width, 0),
            chain_connectivity: Grid::new(height, width, -1),
            chain_length: Grid::new(height, width, 0),
            field: Grid::new(2 * height - 1, 2 * width - 1, Edge::Undecided),
            seed_idx: Grid::new(2 * height - 1, 2 * width - 1, -1),
            seeds: vec![(Y(0), X(0)); (height * width) as usize],
            seed_count: 0,
            endpoint_constraint: match opt.endpoint_constraint {
                Some(ep) => ep.clone(),
                None => Grid::new(height, width, Endpoint::Any),
            },
            endpoints: 0,
            endpoint_forced_cells: 0,
            chain_threshold: opt.chain_threshold,
            forbid_adjacent_clue: opt.forbid_adjacent_clue,
            symmetry: opt.symmetry,
            invalid: false,
            search_queue: FiniteSearchQueue::new((height * width) as usize),
        };

        for idx in 0..((height * width) as usize) {
            ret.chain_union[idx] = idx;
            if ret.endpoint_constraint[idx] == Endpoint::Forced {
                ret.endpoint_forced_cells += 1;
            }
        }

        ret.seeds[0] = (Y(0), X(0));
        ret.seeds[1] = (Y(0), X(2 * width - 2));
        ret.seeds[2] = (Y(2 * height - 2), X(0));
        ret.seeds[3] = (Y(2 * height - 2), X(2 * width - 2));
        ret.seed_count = 4;
        ret.seed_idx[(Y(0), X(0))] = 0;
        ret.seed_idx[(Y(0), X(2 * width - 2))] = 1;
        ret.seed_idx[(Y(2 * height - 2), X(0))] = 2;
        ret.seed_idx[(Y(2 * height - 2), X(2 * width - 2))] = 3;
        ret
    }

    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn is_invalid(&self) -> bool {
        self.invalid
    }
    pub fn set_invalid(&mut self) {
        self.invalid = true;
    }
    pub fn endpoint_forced_cells(&self) -> i32 {
        self.endpoint_forced_cells
    }

    pub fn get_edge(&self, cd: Coord) -> Edge {
        if self.field.is_valid_coord(cd) {
            self.field[cd]
        } else {
            Edge::Blank
        }
    }

    pub fn get_endpoint_constraint(&self, cd: Coord) -> Endpoint {
        self.endpoint_constraint[cd]
    }

    /// Counts the number of (Line, Undecided) around `cd`
    pub fn count_neighbor(&self, cd: Coord) -> (i32, i32) {
        let (Y(y), X(x)) = cd;
        let mut line = 0;
        let mut undecided = 0;
        let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        for &(dy, dx) in &dirs {
            let e = self.get_edge((Y(y + dy), X(x + dx)));
            if e == Edge::Line {
                line += 1;
            } else if e == Edge::Undecided {
                undecided += 1;
            }
        }
        (line, undecided)
    }

    /// Returns all neighbors whose state is `Undecided` around `cd`
    pub fn undecided_neighbors(&self, cd: Coord) -> Vec<Coord> {
        let (Y(y), X(x)) = cd;
        let mut ret = vec![];
        let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        for &(dy, dx) in &dirs {
            let cd2 = (Y(y + dy), X(x + dx));
            let e = self.get_edge(cd2);
            if e == Edge::Undecided {
                ret.push(cd2);
            }
        }
        ret
    }

    fn undecided_neighbors_summary(&self, cd: Coord) -> Cnt<Coord> {
        let (Y(y), X(x)) = cd;
        let mut ret = Cnt::None;
        let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        for &(dy, dx) in &dirs {
            let cd2 = (Y(y + dy), X(x + dx));
            let e = self.get_edge(cd2);
            if e == Edge::Undecided {
                ret = match ret {
                    Cnt::None => Cnt::One(cd2),
                    _ => return Cnt::Many,
                }
            }
        }
        ret
    }

    /// Returns whether vertex `cd` is a seed
    pub fn is_seed(&self, cd: Coord) -> bool {
        let nb = self.count_neighbor(cd);
        nb == (0, 2) || (nb.0 == 1 && nb.1 > 0)
    }

    /// Copy `src` into this `AnswerField`.
    /// the shape of these `AnswerField`s must match.
    pub fn copy_from(&mut self, src: &AnswerField) {
        self.chain_union.copy_from(&src.chain_union);
        self.chain_connectivity.copy_from(&src.chain_connectivity);
        self.chain_length.copy_from(&src.chain_length);
        self.field.copy_from(&src.field);
        self.seed_idx.copy_from(&src.seed_idx);

        self.seeds[0..src.seed_count].copy_from_slice(&src.seeds[0..src.seed_count]);
        self.seed_count = src.seed_count;

        self.endpoint_constraint.copy_from(&src.endpoint_constraint);
        self.endpoints = src.endpoints;
        self.endpoint_forced_cells = src.endpoint_forced_cells;
        self.chain_threshold = src.chain_threshold;
        self.forbid_adjacent_clue = src.forbid_adjacent_clue;
        self.symmetry = src.symmetry;
        self.invalid = src.invalid;
    }

    /// Returns the representative node of the union containing `x` in `chain_connectivity`.
    /// Performs path compression to reduce complexity.
    fn root_mut(&mut self, x: usize) -> usize {
        if self.chain_connectivity[x] < 0 {
            x as usize
        } else {
            let parent = self.chain_connectivity[x] as usize;
            let ret = self.root_mut(parent);
            self.chain_connectivity[x] = ret as i32;
            ret
        }
    }

    /// Returns the representative node of the union containing `x` in `chain_connectivity`.
    fn root(&self, x: usize) -> usize {
        if self.chain_connectivity[x] < 0 {
            x as usize
        } else {
            let parent = self.chain_connectivity[x] as usize;
            self.root(parent)
        }
    }

    pub fn root_from_coord(&self, cd: Coord) -> usize {
        self.root(self.chain_connectivity.index(cd))
    }

    /// Join `x` and `y` in `chain_connectivity`
    fn join(&mut self, x: usize, y: usize) {
        let x = self.root_mut(x);
        let y = self.root_mut(y);
        if x != y {
            if self.chain_connectivity[x] < self.chain_connectivity[y] {
                self.chain_connectivity[x] += self.chain_connectivity[y];
                self.chain_connectivity[y] = x as i32;
            } else {
                self.chain_connectivity[y] += self.chain_connectivity[x];
                self.chain_connectivity[x] = y as i32;
            }
        }
    }

    /// Returns whether there is at least one seed
    pub fn has_seed(&self) -> bool {
        self.seed_count != 0
    }

    /// Returns a random seed using `rng`
    pub fn random_seed<R: Rng>(&self, rng: &mut R) -> Coord {
        let idx = rng.gen_range(0, self.seed_count);
        self.seeds[idx]
    }

    fn complexity(&self, cd: Coord) -> i32 {
        let (Y(y), X(x)) = cd;
        let ret = if y > 0 {
            4 - self.count_neighbor((Y(y - 2), X(x))).1
        } else {
            0
        } + if x > 0 {
            4 - self.count_neighbor((Y(y), X(x - 2))).1
        } else {
            0
        } + if y < self.height * 2 - 2 {
            4 - self.count_neighbor((Y(y + 2), X(x))).1
        } else {
            0
        } + if x < self.width * 2 - 2 {
            4 - self.count_neighbor((Y(y), X(x + 2))).1
        } else {
            0
        };

        ret
    }

    /// Returns a seed with largest complexity among `k` samples
    pub fn best_seed<R: Rng>(&self, k: i32, rng: &mut R) -> Coord {
        let mut seed = self.random_seed(rng);
        let mut complexity = self.complexity(seed);

        for _ in 1..k {
            let seed_cand = self.random_seed(rng);
            let complexity_cand = self.complexity(seed_cand);

            if complexity < complexity_cand {
                seed = seed_cand;
                complexity = complexity_cand;
            }
        }

        seed
    }

    /// Update `endpoint_constraint[cd]`.
    /// `cd` must be in vertex-coordinate.
    pub fn update_endpoint_constraint(&mut self, cd: Coord, constraint: Endpoint) {
        if !self.search_queue.is_started() {
            self.search_queue.start();
            self.update_endpoint_constraint_int(cd, constraint);
            self.queue_pop_all();
            self.search_queue.finish();
        } else {
            self.update_endpoint_constraint_int(cd, constraint);
        }
    }

    fn update_endpoint_constraint_int(&mut self, cd: Coord, constraint: Endpoint) {
        let (Y(y), X(x)) = cd;
        if self.endpoint_constraint[cd] == Endpoint::Any {
            self.endpoint_constraint[cd] = constraint;
            if constraint == Endpoint::Forced {
                self.endpoint_forced_cells += 1;
            }
            self.inspect((Y(y * 2), X(x * 2)));
        } else if self.endpoint_constraint[cd] != constraint {
            self.invalid = true;
        }
    }

    fn queue_pop_all(&mut self) {
        while !self.search_queue.empty() && !self.invalid {
            let idx = self.search_queue.pop();
            let (Y(y), X(x)) = self.chain_connectivity.coord(idx);
            self.inspect_int((Y(y * 2), X(x * 2)));
        }
        self.search_queue.clear();
    }

    pub fn decide(&mut self, cd: Coord, state: Edge) {
        if !self.search_queue.is_started() {
            self.search_queue.start();
            self.decide_int(cd, state);
            self.queue_pop_all();
            self.search_queue.finish();
        } else {
            self.decide_int(cd, state);
        }
    }

    fn decide_int(&mut self, cd: Coord, state: Edge) {
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

            self.join(another_end1_id, another_end2_id);
            self.root_mut(another_end1_id);
            self.root_mut(another_end2_id);

            if new_length < self.chain_threshold {
                let cd = self.chain_union.coord(another_end1_id);
                self.extend_chain(cd);
            }
        }

        // check incident vertices
        if y % 2 == 1 {
            if self.count_neighbor((Y(y - 1), X(x))) == (1, 0) {
                self.endpoints += 1;
            }
            if self.count_neighbor((Y(y + 1), X(x))) == (1, 0) {
                self.endpoints += 1;
            }
            self.inspect((Y(y - 1), X(x)));
            self.inspect((Y(y + 1), X(x)));
        } else {
            if self.count_neighbor((Y(y), X(x - 1))) == (1, 0) {
                self.endpoints += 1;
            }
            if self.count_neighbor((Y(y), X(x + 1))) == (1, 0) {
                self.endpoints += 1;
            }
            self.inspect((Y(y), X(x - 1)));
            self.inspect((Y(y), X(x + 1)));
        }

        // check for canonization rule
        if state == Edge::Line {
            if y % 2 == 1 {
                let related = [(Y(y), X(x - 2)), (Y(y - 1), X(x - 1)), (Y(y + 1), X(x - 1))];
                for i in 0..3 {
                    if self.get_edge(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
                let related = [(Y(y), X(x + 2)), (Y(y - 1), X(x + 1)), (Y(y + 1), X(x + 1))];
                for i in 0..3 {
                    if self.get_edge(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
            } else {
                let related = [(Y(y - 2), X(x)), (Y(y - 1), X(x - 1)), (Y(y - 1), X(x + 1))];
                for i in 0..3 {
                    if self.get_edge(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
                let related = [(Y(y + 2), X(x)), (Y(y + 1), X(x - 1)), (Y(y + 1), X(x + 1))];
                for i in 0..3 {
                    if self.get_edge(related[i]) == Edge::Line {
                        self.decide(related[(i + 1) % 3], Edge::Blank);
                        self.decide(related[(i + 2) % 3], Edge::Blank);
                    }
                }
            }
        }
    }

    /// Inspect all vertices
    pub fn inspect_all(&mut self) {
        assert_eq!(self.search_queue.is_started(), false);

        self.search_queue.start();

        let height = self.height;
        let width = self.width;

        for y in 0..height {
            for x in 0..width {
                self.inspect((Y(y * 2), X(x * 2)));
            }
        }

        self.queue_pop_all();
        self.search_queue.finish();
    }

    /// Inspect vertex (y, x)
    fn inspect(&mut self, cd: Coord) {
        assert_eq!(self.search_queue.is_started(), true);

        let (Y(y), X(x)) = cd;
        self.search_queue
            .push(self.chain_connectivity.index((Y(y / 2), X(x / 2))));
    }

    fn inspect_int(&mut self, (Y(y), X(x)): Coord) {
        let (line, undecided) = self.count_neighbor((Y(y), X(x)));
        let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        if line == 0 {
            if undecided == 0 {
                self.invalid = true;
                return;
            }
            if undecided == 1 {
                for &(dy, dx) in &dirs {
                    let e = self.get_edge((Y(y + dy), X(x + dx)));
                    if e == Edge::Undecided {
                        self.decide((Y(y + dy), X(x + dx)), Edge::Line);
                    }
                }
            }
        } else if line == 2 {
            for &(dy, dx) in &dirs {
                let e = self.get_edge((Y(y + dy), X(x + dx)));
                if e == Edge::Undecided {
                    self.decide((Y(y + dy), X(x + dx)), Edge::Blank);
                }
            }
        } else if line == 1 {
            // avoid too short chains
            if self.chain_length[(Y(y / 2), X(x / 2))] < self.chain_threshold {
                self.extend_chain((Y(y / 2), X(x / 2)));

                let (Y(ay), X(ax)) = self.chain_union
                    .coord(self.chain_union[(Y(y / 2), X(x / 2))]);
                if self.count_neighbor((Y(ay * 2), X(ax * 2))) == (1, 0) {
                    let minimum_len =
                        self.chain_threshold - self.chain_length[(Y(y / 2), X(x / 2))];
                    for &(dy, dx) in &dirs {
                        if self.get_edge((Y(y + dy), X(x + dx))) == Edge::Undecided {
                            let (Y(ay), X(ax)) = self.chain_union
                                .coord(self.chain_union[(Y(y / 2 + dy), X(x / 2 + dx))]);
                            if self.count_neighbor((Y(ay * 2), X(ax * 2))) == (1, 0)
                                && self.chain_length[(Y(y / 2 + dy), X(x / 2 + dx))] < minimum_len
                            {
                                self.decide((Y(y + dy), X(x + dx)), Edge::Blank);
                            }
                        }
                    }
                }
            }
        } else if line >= 3 {
            self.invalid = true;
            return;
        }

        if line == 1 && undecided == 0 {
            if self.get_endpoint_constraint((Y(y / 2), X(x / 2))) == Endpoint::Prohibited {
                self.invalid = true;
                return;
            }
            if self.endpoint_constraint[(Y(y / 2), X(x / 2))] == Endpoint::Any {
                self.endpoint_constraint[(Y(y / 2), X(x / 2))] = Endpoint::Forced;
                self.endpoint_forced_cells += 1;
            }
        }
        if line == 2 {
            if self.get_endpoint_constraint((Y(y / 2), X(x / 2))) == Endpoint::Forced {
                self.invalid = true;
                return;
            }
            if self.endpoint_constraint[(Y(y / 2), X(x / 2))] == Endpoint::Any {
                self.endpoint_constraint[(Y(y / 2), X(x / 2))] = Endpoint::Prohibited;
            }
        }

        if self.forbid_adjacent_clue
            && (self.get_endpoint_constraint((Y(y / 2), X(x / 2))) == Endpoint::Forced
                || (line == 1 && undecided == 0))
        {
            for dy in -1..2 {
                for dx in -1..2 {
                    if dy == 0 && dx == 0 {
                        continue;
                    }
                    if y / 2 + dy < 0 || y / 2 + dy >= self.height || x / 2 + dx < 0
                        || x / 2 + dx >= self.width
                    {
                        continue;
                    }
                    self.update_endpoint_constraint(
                        (Y(y / 2 + dy), X(x / 2 + dx)),
                        Endpoint::Prohibited,
                    );
                }
            }
        }
        if self.forbid_adjacent_clue && line + undecided == 2 {
            let adj = [(Y(1), X(0)), (Y(0), X(1)), (Y(-1), X(0)), (Y(0), X(-1))];
            for &(Y(dy), X(dx)) in &adj {
                if self.get_edge((Y(y + dy), X(x + dx))) != Edge::Blank {
                    let nb = self.count_neighbor((Y(y + 2 * dy), X(x + 2 * dx)));
                    if nb.0 + nb.1 == 2 {
                        self.decide((Y(y + dy), X(x + dx)), Edge::Line);
                    }
                }
            }
        }

        let con = self.get_endpoint_constraint((Y(y / 2), X(x / 2)));
        if con != Endpoint::Any {
            let height = self.height;
            let width = self.width;
            if self.symmetry.tetrad {
                self.update_endpoint_constraint((Y(x / 2), X(width - 1 - y / 2)), con);
            } else if self.symmetry.dyad {
                self.update_endpoint_constraint((Y(height - 1 - y / 2), X(width - 1 - x / 2)), con);
            }
            if self.symmetry.horizontal {
                self.update_endpoint_constraint((Y(height - 1 - y / 2), X(x / 2)), con);
            }
            if self.symmetry.vertical {
                self.update_endpoint_constraint((Y(y / 2), X(width - 1 - x / 2)), con);
            }
        }

        match self.get_endpoint_constraint((Y(y / 2), X(x / 2))) {
            Endpoint::Any => (),
            Endpoint::Forced => {
                if line == 1 {
                    for &(dy, dx) in &dirs {
                        let e = self.get_edge((Y(y + dy), X(x + dx)));
                        if e == Edge::Undecided {
                            self.decide((Y(y + dy), X(x + dx)), Edge::Blank);
                        }
                    }
                } else if line >= 2 {
                    self.invalid = true;
                }
            }
            Endpoint::Prohibited => {
                if line == 1 {
                    if undecided == 0 {
                        self.invalid = true;
                        return;
                    } else if undecided == 1 {
                        for &(dy, dx) in &dirs {
                            let e = self.get_edge((Y(y + dy), X(x + dx)));
                            if e == Edge::Undecided {
                                self.decide((Y(y + dy), X(x + dx)), Edge::Line);
                            }
                        }
                    }
                } else if line == 0 && undecided == 2 {
                    for &(dy, dx) in &dirs {
                        let e = self.get_edge((Y(y + dy), X(x + dx)));
                        if e == Edge::Undecided {
                            self.decide((Y(y + dy), X(x + dx)), Edge::Line);
                        }
                    }
                }
            }
        }

        let is_seed = self.is_seed((Y(y), X(x)));
        let seed_idx = self.seed_idx[(Y(y), X(x))];

        if seed_idx != -1 && !is_seed {
            // (y, x) is no longer a seed
            let moved = self.seeds[self.seed_count - 1];
            self.seed_idx[moved] = seed_idx;
            self.seeds[seed_idx as usize] = moved;
            self.seed_count -= 1;
            self.seed_idx[(Y(y), X(x))] = -1;
        } else if seed_idx == -1 && is_seed {
            // (y, x) is now a seed
            self.seed_idx[(Y(y), X(x))] = self.seed_count as i32;
            self.seeds[self.seed_count] = (Y(y), X(x));
            self.seed_count += 1;
        }
    }

    /// Extend the chain one of whose endpoint is `(y, x)`
    fn extend_chain(&mut self, (Y(y), X(x)): Coord) {
        let end1_id = self.chain_union.index((Y(y), X(x)));
        let end2_id = self.chain_union[end1_id];

        let end1 = (Y(y * 2), X(x * 2));
        let (Y(y2), X(x2)) = self.chain_union.coord(end2_id);
        let end2 = (Y(y2 * 2), X(x2 * 2));

        let end1_undecided = self.undecided_neighbors_summary(end1);
        let end2_undecided = self.undecided_neighbors_summary(end2);

        if end1_undecided == Cnt::None {
            let con = self.endpoint_constraint[(Y(y2), X(x2))];
            match con {
                Endpoint::Forced => {
                    self.invalid = true;
                    return;
                }
                Endpoint::Any => {
                    self.endpoint_constraint[(Y(y2), X(x2))] = Endpoint::Prohibited;
                    self.inspect((Y(y2 * 2), X(x2 * 2)));
                }
                Endpoint::Prohibited => (),
            }
        }
        if end2_undecided == Cnt::None {
            let con = self.endpoint_constraint[(Y(y), X(x))];
            match con {
                Endpoint::Forced => {
                    self.invalid = true;
                    return;
                }
                Endpoint::Any => {
                    self.endpoint_constraint[(Y(y), X(x))] = Endpoint::Prohibited;
                    self.inspect((Y(y * 2), X(x * 2)));
                }
                Endpoint::Prohibited => (),
            }
        }
        match (end1_undecided, end2_undecided) {
            (Cnt::None, Cnt::None) => {
                self.invalid = true;
                return;
            }
            (Cnt::None, Cnt::One(e)) | (Cnt::One(e), Cnt::None) => self.decide(e, Edge::Line),
            _ => (),
        }
    }

    pub fn forbid_further_endpoint(&mut self) {
        // this function should not be called from internal functions
        assert_eq!(self.search_queue.is_started(), false);

        self.search_queue.start();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.endpoint_constraint[(Y(y), X(x))] == Endpoint::Any {
                    self.update_endpoint_constraint((Y(y), X(x)), Endpoint::Prohibited);
                }
            }
        }
        self.queue_pop_all();
        self.search_queue.finish();
    }

    /// Convert into `LinePlacement`
    pub fn as_line_placement(&self) -> LinePlacement {
        let height = self.height;
        let width = self.width;
        let mut ret = LinePlacement::new(height, width);

        for y in 0..height {
            for x in 0..width {
                if y < height - 1 {
                    if self.get_edge((Y(y * 2 + 1), X(x * 2))) == Edge::Line {
                        ret.set_down((Y(y), X(x)), true);
                    }
                }
                if x < width - 1 {
                    if self.get_edge((Y(y * 2), X(x * 2 + 1))) == Edge::Line {
                        ret.set_right((Y(y), X(x)), true);
                    }
                }
            }
        }

        ret
    }
}

impl fmt::Debug for AnswerField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let height = self.height;
        let width = self.width;

        for y in 0..(2 * height - 1) {
            for x in 0..(2 * width - 1) {
                match (y % 2, x % 2) {
                    (0, 0) => write!(
                        f,
                        "{}",
                        match self.endpoint_constraint[(Y(y / 2), X(x / 2))] {
                            Endpoint::Any => '#',
                            Endpoint::Forced => '*',
                            Endpoint::Prohibited => '+',
                        }
                    )?,
                    (0, 1) => write!(
                        f,
                        "{}",
                        match self.get_edge((Y(y), X(x))) {
                            Edge::Undecided => ' ',
                            Edge::Line => '-',
                            Edge::Blank => 'x',
                        }
                    )?,
                    (1, 0) => write!(
                        f,
                        "{}",
                        match self.get_edge((Y(y), X(x))) {
                            Edge::Undecided => ' ',
                            Edge::Line => '|',
                            Edge::Blank => 'x',
                        }
                    )?,
                    (1, 1) => write!(f, " ")?,
                    _ => unreachable!(),
                }
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
