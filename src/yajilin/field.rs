use super::super::{Coord, Grid, X, Y};
use super::*;
use grid_loop::{Edge, GridLoop, GridLoopField};
use std::cmp;
use std::fmt;

#[derive(Clone)]
pub struct Field {
    grid_loop: GridLoop,
    clue: Grid<Clue>,
    cell: Grid<Cell>,
    blocked_either_down: Grid<bool>,
    blocked_either_right: Grid<bool>,
    technique: Technique,
}

const FOUR_NEIGHBORS: [(i32, i32); 4] = [(-1, 0), (0, -1), (1, 0), (0, 1)];

impl Field {
    pub fn new(clue: &Grid<Clue>) -> Field {
        let height = clue.height();
        let width = clue.width();

        let mut cell = Grid::new(height, width, Cell::Undecided);
        let mut grid_loop = GridLoop::new(height - 1, width - 1);

        {
            let mut handle = GridLoop::get_handle(&mut grid_loop);
            for y in 0..height {
                for x in 0..width {
                    let c = clue[(Y(y), X(x))];
                    if c != Clue::NoClue {
                        cell[(Y(y), X(x))] = Cell::Clue;
                        GridLoop::decide_edge(&mut *handle, (Y(y * 2 - 1), X(x * 2)), Edge::Blank);
                        GridLoop::decide_edge(&mut *handle, (Y(y * 2 + 1), X(x * 2)), Edge::Blank);
                        GridLoop::decide_edge(&mut *handle, (Y(y * 2), X(x * 2 - 1)), Edge::Blank);
                        GridLoop::decide_edge(&mut *handle, (Y(y * 2), X(x * 2 + 1)), Edge::Blank);
                    }
                }
            }
        }
        Field {
            grid_loop,
            clue: clue.clone(),
            cell,
            blocked_either_down: Grid::new(height - 1, width, false),
            blocked_either_right: Grid::new(height, width - 1, false),
            technique: Technique::new(),
        }
    }
    pub fn get_technique(&self) -> Technique {
        self.technique
    }
    pub fn set_technique(&mut self, technique: Technique) {
        self.technique = technique;
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
    pub fn get_cell(&self, cd: Coord) -> Cell {
        self.cell[cd]
    }
    pub fn get_cell_safe(&self, cd: Coord) -> Cell {
        if self.cell.is_valid_coord(cd) {
            self.cell[cd]
        } else {
            // The outside of the field can be identified with a (meaningless) clue
            Cell::Clue
        }
    }
    pub fn get_edge(&self, cd: Coord) -> Edge {
        self.grid_loop.get_edge(cd)
    }
    pub fn get_edge_safe(&self, cd: Coord) -> Edge {
        self.grid_loop.get_edge_safe(cd)
    }

    pub fn set_cell(&mut self, cd: Coord, v: Cell) {
        let mut handle = GridLoop::get_handle(self);
        handle.set_cell_internal(cd, v);
    }
    pub fn check_all_cell(&mut self) {
        let height = self.height();
        let width = self.width();
        let mut handle = GridLoop::get_handle(self);
        for y in 0..height {
            for x in 0..width {
                GridLoop::check(&mut *handle, (Y(y * 2), X(x * 2)));
            }
        }
    }
    pub fn solve(&mut self) {
        loop {
            let current_decided_lines = self.grid_loop.num_decided_lines();
            self.check_all_cell();
            GridLoop::apply_inout_rule(self);
            GridLoop::check_connectability(self);
            self.apply_inout_rule_advanced();
            self.check_local_parity();
            if current_decided_lines == self.grid_loop.num_decided_lines() {
                break;
            }
        }
    }
    pub fn apply_inout_rule_advanced(&mut self) {
        if !self.technique.inout_advanced {
            return;
        }
        let height = self.height() - 1;
        let width = self.width() - 1;
        let outside = height * width;

        let cell_id = |(Y(y), X(x))| {
            if 0 <= y && y < height && 0 <= x && x < width {
                (y * width + x)
            } else {
                outside
            }
        };

        let mut union_find = UnionFind::new((outside * 2 + 2) as usize);

        for y in 0..height {
            for x in 0..(width + 1) {
                match self.grid_loop.get_edge((Y(y * 2 + 1), X(x * 2))) {
                    Edge::Line => {
                        let u = cell_id((Y(y), X(x - 1)));
                        let v = cell_id((Y(y), X(x)));
                        union_find.join(u * 2, v * 2 + 1);
                        union_find.join(u * 2 + 1, v * 2);
                    }
                    Edge::Blank => {
                        let u = cell_id((Y(y), X(x - 1)));
                        let v = cell_id((Y(y), X(x)));
                        union_find.join(u * 2, v * 2);
                        union_find.join(u * 2 + 1, v * 2 + 1);
                    }
                    Edge::Undecided => (),
                }
                /*
                if self.blocked_either_down[(Y(y), X(x))]
                    && self.get_cell_safe((Y(y - 1), X(x))).is_blocking()
                    && self.get_cell_safe((Y(y + 2), X(x))).is_blocking()
                    */
                if self.blocked_either_down[(Y(y), X(x))]
                    && self.get_edge_safe((Y(y * 2 - 1), X(x * 2))) == Edge::Blank
                    && self.get_edge_safe((Y(y * 2 + 3), X(x * 2))) == Edge::Blank
                {
                    let u = cell_id((Y(y - 1), X(x)));
                    let v = cell_id((Y(y + 1), X(x)));
                    union_find.join(u * 2, v * 2 + 1);
                    union_find.join(u * 2 + 1, v * 2);
                }
            }
        }
        for y in 0..(height + 1) {
            for x in 0..width {
                match self.grid_loop.get_edge((Y(y * 2), X(x * 2 + 1))) {
                    Edge::Line => {
                        let u = cell_id((Y(y - 1), X(x)));
                        let v = cell_id((Y(y), X(x)));
                        union_find.join(u * 2, v * 2 + 1);
                        union_find.join(u * 2 + 1, v * 2);
                    }
                    Edge::Blank => {
                        let u = cell_id((Y(y - 1), X(x)));
                        let v = cell_id((Y(y), X(x)));
                        union_find.join(u * 2, v * 2);
                        union_find.join(u * 2 + 1, v * 2 + 1);
                    }
                    Edge::Undecided => (),
                }
                /*
                if self.blocked_either_right[(Y(y), X(x))]
                    && self.get_cell_safe((Y(y), X(x - 1))).is_blocking()
                    && self.get_cell_safe((Y(y), X(x + 2))).is_blocking()
                    */
                if self.blocked_either_right[(Y(y), X(x))]
                    && self.get_edge_safe((Y(y * 2), X(x * 2 - 1))) == Edge::Blank
                    && self.get_edge_safe((Y(y * 2), X(x * 2 + 3))) == Edge::Blank
                {
                    let u = cell_id((Y(y), X(x - 1)));
                    let v = cell_id((Y(y), X(x + 1)));
                    union_find.join(u * 2, v * 2 + 1);
                    union_find.join(u * 2 + 1, v * 2);
                }
            }
        }

        for y in 0..height {
            for x in 0..(width + 1) {
                let u = cell_id((Y(y), X(x - 1)));
                let v = cell_id((Y(y), X(x)));

                if union_find.root(u * 2) == union_find.root(v * 2) {
                    GridLoop::decide_edge(self, (Y(y * 2 + 1), X(x * 2)), Edge::Blank);
                } else if union_find.root(u * 2) == union_find.root(v * 2 + 1) {
                    GridLoop::decide_edge(self, (Y(y * 2 + 1), X(x * 2)), Edge::Line);
                }
            }
        }
        for y in 0..(height + 1) {
            for x in 0..width {
                let u = cell_id((Y(y - 1), X(x)));
                let v = cell_id((Y(y), X(x)));

                if union_find.root(u * 2) == union_find.root(v * 2) {
                    GridLoop::decide_edge(self, (Y(y * 2), X(x * 2 + 1)), Edge::Blank);
                } else if union_find.root(u * 2) == union_find.root(v * 2 + 1) {
                    GridLoop::decide_edge(self, (Y(y * 2), X(x * 2 + 1)), Edge::Line);
                }
            }
        }
    }
    pub fn check_local_parity(&mut self) {
        if !self.technique.local_parity {
            return;
        }
        let height = self.height();
        let width = self.width();

        let mut ids = Grid::new(height, width, -1);
        let mut id = 0;

        fn visit(y: i32, x: i32, id: i32, ids: &mut Grid<i32>, grid_loop: &GridLoop) {
            if ids[(Y(y), X(x))] != -1 {
                return;
            }
            ids[(Y(y), X(x))] = id;
            if y > 0 && grid_loop.get_edge((Y(2 * y - 1), X(2 * x))) == Edge::Undecided {
                visit(y - 1, x, id, ids, grid_loop);
            }
            if y < ids.height() - 1
                && grid_loop.get_edge((Y(2 * y + 1), X(2 * x))) == Edge::Undecided
            {
                visit(y + 1, x, id, ids, grid_loop);
            }
            if x > 0 && grid_loop.get_edge((Y(2 * y), X(2 * x - 1))) == Edge::Undecided {
                visit(y, x - 1, id, ids, grid_loop);
            }
            if x < ids.width() - 1
                && grid_loop.get_edge((Y(2 * y), X(2 * x + 1))) == Edge::Undecided
            {
                visit(y, x + 1, id, ids, grid_loop);
            }
        }
        for y in 0..height {
            for x in 0..width {
                if ids[(Y(y), X(x))] == -1 {
                    visit(y, x, id, &mut ids, &self.grid_loop);
                    id += 1;
                }
            }
        }
        let mut undecided_loc = vec![(-1, -1); id as usize];
        let mut num_endpoint_even = vec![0; id as usize];
        let mut num_endpoint_odd = vec![0; id as usize];
        let mut num_cells = vec![0; id as usize];
        for y in 0..(2 * height - 1) {
            for x in 0..(2 * width - 1) {
                if y % 2 == x % 2 {
                    continue;
                }
                if self.get_edge((Y(y), X(x))) == Edge::Line
                    && ids[(Y(y / 2), X(x / 2))] != ids[(Y((y + 1) / 2), X((x + 1) / 2))]
                {
                    // waf
                    let id1 = ids[(Y(y / 2), X(x / 2))] as usize;
                    let id2 = ids[(Y((y + 1) / 2), X((x + 1) / 2))] as usize;
                    if (y / 2 + x / 2) % 2 == 0 {
                        num_endpoint_even[id1] += 1;
                        num_endpoint_odd[id2] += 1;
                    } else {
                        num_endpoint_odd[id1] += 1;
                        num_endpoint_even[id2] += 1;
                    }
                }
            }
        }
        for y in 0..height {
            for x in 0..width {
                let id = ids[(Y(y), X(x))] as usize;
                num_cells[id] += 1;
                if self.get_cell((Y(y), X(x))) == Cell::Undecided {
                    if undecided_loc[id].0 == -1 {
                        undecided_loc[id] = (y, x);
                    } else {
                        undecided_loc[id] = (-2, -2);
                    }
                }
            }
        }
        for i in 0..(id as usize) {
            if undecided_loc[i].0 < 0 {
                continue;
            }
            let (y, x) = undecided_loc[i];
            let cell_parity = ((num_endpoint_even[i] - num_endpoint_odd[i]) / 2) & 1;
            if cell_parity == (num_cells[i] & 1) {
                self.set_cell((Y(y), X(x)), Cell::Line);
            } else {
                self.set_cell((Y(y), X(x)), Cell::Blocked);
            }
        }
    }
    pub fn trial_and_error(&mut self, depth: i32) {
        let height = self.height();
        let width = self.width();
        self.solve();

        if depth == 0 {
            return;
        }
        loop {
            let mut updated = false;
            for y in 0..(height * 2 - 1) {
                for x in 0..(width * 2 - 1) {
                    if y % 2 == x % 2 {
                        continue;
                    }
                    if self.get_edge((Y(y), X(x))) != Edge::Undecided {
                        continue;
                    }
                    if !self.grid_loop.is_root((Y(y), X(x))) {
                        continue;
                    }

                    {
                        let mut field_line = self.clone();
                        GridLoop::decide_edge(&mut field_line, (Y(y), X(x)), Edge::Line);
                        field_line.trial_and_error(depth - 1);

                        if field_line.inconsistent() {
                            updated = true;
                            GridLoop::decide_edge(self, (Y(y), X(x)), Edge::Blank);
                            self.trial_and_error(depth - 1);
                        }
                    }
                    {
                        let mut field_blank = self.clone();
                        GridLoop::decide_edge(&mut field_blank, (Y(y), X(x)), Edge::Blank);
                        field_blank.trial_and_error(depth - 1);

                        if field_blank.inconsistent() {
                            updated = true;
                            GridLoop::decide_edge(self, (Y(y), X(x)), Edge::Line);
                            self.trial_and_error(depth - 1);
                        }
                    }
                    if self.inconsistent() {
                        return;
                    }
                }
            }
            if !updated {
                break;
            }
        }
    }

    fn set_cell_internal(&mut self, cd: Coord, v: Cell) {
        let current = self.cell[cd];
        if current != Cell::Undecided {
            if current != v {
                self.set_inconsistent();
            }
            return;
        }

        self.cell[cd] = v;
        let (Y(y), X(x)) = cd;
        match v {
            Cell::Undecided => (),
            Cell::Clue => (), // don't do this!
            Cell::Line => GridLoop::check(self, (Y(y * 2), X(x * 2))),
            Cell::Blocked => {
                for &(dy, dx) in &FOUR_NEIGHBORS {
                    if self.get_cell_safe((Y(y + dy), X(x + dx))) != Cell::Clue {
                        self.set_cell_internal((Y(y + dy), X(x + dx)), Cell::Line);
                    }
                    GridLoop::decide_edge(self, (Y(y * 2 + dy), X(x * 2 + dx)), Edge::Blank);
                }
            }
        }
    }
    fn set_cell_internal_unless_clue(&mut self, cd: Coord, v: Cell) {
        if self.get_cell_safe(cd) == Cell::Clue {
            return;
        }
        self.set_cell_internal(cd, v);
    }
    fn set_blocked_either(&mut self, cd1: Coord, cd2: Coord) {
        if self.get_cell(cd1) == Cell::Clue || self.get_cell(cd2) == Cell::Clue {
            return;
        }

        let (Y(y1), X(x1)) = cd1;
        let (Y(y2), X(x2)) = cd2;

        if y1 == y2 {
            if x2 == x1 + 1 {
                self.blocked_either_right[(Y(y1), X(x1))] = true;
            } else if x1 == x2 + 1 {
                self.blocked_either_right[(Y(y1), X(x2))] = true;
            } else {
                panic!();
            }
        } else if x1 == x2 {
            if y2 == y1 + 1 {
                self.blocked_either_down[(Y(y1), X(x1))] = true;
            } else if y1 == y2 + 1 {
                self.blocked_either_down[(Y(y2), X(x1))] = true;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
    fn two_by_two(&mut self, top: Coord) {
        if !self.technique.two_by_two {
            return;
        }
        // 2x2 square (y, x) -- (y+1, x+1) has 2 blocked cells
        let (Y(y), X(x)) = top;
        for &(y2, x2) in &[
            (y - 1, x),
            (y - 1, x + 1),
            (y, x - 1),
            (y, x + 2),
            (y + 1, x - 1),
            (y + 1, x + 2),
            (y + 2, x),
            (y + 2, x + 1),
        ] {
            self.set_cell_internal_unless_clue((Y(y2), X(x2)), Cell::Line);
        }
    }
    fn check_two_by_three_rr(
        &mut self,
        y0: i32,
        x0: i32,
        y1: i32,
        x1: i32,
        ty0: i32,
        tx0: i32,
        ty1: i32,
        tx1: i32,
    ) {
        if self.blocked_either_right[(Y(y0), X(x0))] && self.blocked_either_right[(Y(y1), X(x1))] {
            if self.get_cell_safe((Y(ty0), X(tx0))) != Cell::Clue
                && self.get_cell_safe((Y(ty1), X(tx1))) != Cell::Clue
            {
                self.set_cell_internal((Y(ty0), X(tx0)), Cell::Line);
                self.set_cell_internal((Y(ty1), X(tx1)), Cell::Line);
            }
        }
    }
    fn check_two_by_three_rd(
        &mut self,
        y0: i32,
        x0: i32,
        y1: i32,
        x1: i32,
        ty0: i32,
        tx0: i32,
        ty1: i32,
        tx1: i32,
    ) {
        if self.blocked_either_right[(Y(y0), X(x0))] && self.blocked_either_down[(Y(y1), X(x1))] {
            if self.get_cell_safe((Y(ty0), X(tx0))) != Cell::Clue
                && self.get_cell_safe((Y(ty1), X(tx1))) != Cell::Clue
            {
                self.set_cell_internal((Y(ty0), X(tx0)), Cell::Line);
                self.set_cell_internal((Y(ty1), X(tx1)), Cell::Line);
            }
        }
    }
    fn check_two_by_three_dd(
        &mut self,
        y0: i32,
        x0: i32,
        y1: i32,
        x1: i32,
        ty0: i32,
        tx0: i32,
        ty1: i32,
        tx1: i32,
    ) {
        if self.blocked_either_down[(Y(y0), X(x0))] && self.blocked_either_down[(Y(y1), X(x1))] {
            if self.get_cell_safe((Y(ty0), X(tx0))) != Cell::Clue
                && self.get_cell_safe((Y(ty1), X(tx1))) != Cell::Clue
            {
                self.set_cell_internal((Y(ty0), X(tx0)), Cell::Line);
                self.set_cell_internal((Y(ty1), X(tx1)), Cell::Line);
            }
        }
    }
    fn check_two_by_three(&mut self, top: Coord) {
        if !self.technique.two_by_three {
            return;
        }
        let (Y(y), X(x)) = top;

        if y <= self.height() - 2 && x <= self.width() - 3 {
            // aa.
            // .bb
            self.check_two_by_three_rr(y, x, y + 1, x + 1, y, x + 2, y + 1, x);

            // aab
            // ..b
            self.check_two_by_three_rd(y, x, y, x + 2, y + 1, x, y + 1, x + 1);

            // abb
            // a..
            self.check_two_by_three_rd(y, x + 1, y, x, y + 1, x + 1, y + 1, x + 2);

            // a..
            // abb
            self.check_two_by_three_rd(y + 1, x + 1, y, x, y, x + 1, y, x + 2);

            // .bb
            // aa.
            self.check_two_by_three_rr(y + 1, x, y, x + 1, y, x, y + 1, x + 2);

            // ..b
            // aab
            self.check_two_by_three_rd(y + 1, x, y, x + 2, y, x, y, x + 1);
        }
        if y <= self.height() - 3 && x <= self.width() - 2 {
            // a.
            // ab
            // .b
            self.check_two_by_three_dd(y, x, y + 1, x + 1, y + 2, x, y, x + 1);

            // a.
            // a.
            // bb
            self.check_two_by_three_rd(y + 2, x, y, x, y, x + 1, y + 1, x + 1);

            // aa
            // b.
            // b.
            self.check_two_by_three_rd(y, x, y + 1, x, y + 1, x + 1, y + 2, x + 1);

            // aa
            // .b
            // .b
            self.check_two_by_three_rd(y, x, y + 1, x + 1, y + 1, x, y + 2, x);

            // .a
            // ba
            // b.
            self.check_two_by_three_dd(y + 1, x, y, x + 1, y, x, y + 2, x + 1);

            // .a
            // .a
            // bb
            self.check_two_by_three_rd(y + 2, x, y, x + 1, y, x, y + 1, x);
        }
    }
    fn around_blocked_either(&mut self, cd: Coord) {
        let (Y(y), X(x)) = cd;

        if y < self.height() - 1 && self.blocked_either_down[cd] {
            if self.get_cell_safe((Y(y - 1), X(x))) == Cell::Clue {
                self.set_cell_internal_unless_clue((Y(y), X(x - 1)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y), X(x + 1)), Cell::Line);
            }
            if self.get_cell_safe((Y(y + 2), X(x))) == Cell::Clue {
                self.set_cell_internal_unless_clue((Y(y + 1), X(x - 1)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y + 1), X(x + 1)), Cell::Line);
            }
            if self.get_cell_safe((Y(y), X(x - 1))) == Cell::Clue
                || self.get_cell_safe((Y(y), X(x + 1))) == Cell::Clue
            {
                self.set_cell_internal_unless_clue((Y(y), X(x - 1)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y - 1), X(x)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y), X(x + 1)), Cell::Line);
            }
            if self.get_cell_safe((Y(y + 1), X(x - 1))) == Cell::Clue
                || self.get_cell_safe((Y(y + 1), X(x + 1))) == Cell::Clue
            {
                self.set_cell_internal_unless_clue((Y(y + 1), X(x - 1)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y + 2), X(x)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y + 1), X(x + 1)), Cell::Line);
            }
        }
        if x < self.width() - 1 && self.blocked_either_right[cd] {
            if self.get_cell_safe((Y(y), X(x - 1))) == Cell::Clue {
                self.set_cell_internal_unless_clue((Y(y - 1), X(x)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y + 1), X(x)), Cell::Line);
            }
            if self.get_cell_safe((Y(y), X(x + 2))) == Cell::Clue {
                self.set_cell_internal_unless_clue((Y(y - 1), X(x + 1)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y + 1), X(x + 1)), Cell::Line);
            }
            if self.get_cell_safe((Y(y - 1), X(x))) == Cell::Clue
                || self.get_cell_safe((Y(y + 1), X(x))) == Cell::Clue
            {
                self.set_cell_internal_unless_clue((Y(y - 1), X(x)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y), X(x - 1)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y + 1), X(x)), Cell::Line);
            }
            if self.get_cell_safe((Y(y - 1), X(x + 1))) == Cell::Clue
                || self.get_cell_safe((Y(y + 1), X(x + 1))) == Cell::Clue
            {
                self.set_cell_internal_unless_clue((Y(y - 1), X(x + 1)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y), X(x + 2)), Cell::Line);
                self.set_cell_internal_unless_clue((Y(y + 1), X(x + 1)), Cell::Line);
            }
        }
    }
    fn inspect_clue(&mut self, cell_cd: Coord) {
        let (Y(y), X(x)) = cell_cd;
        let clue = self.clue[cell_cd];
        let (dy, dx, mut n) = match clue {
            Clue::NoClue | Clue::Empty => return,
            Clue::Up(n) => (-1, 0, n),
            Clue::Left(n) => (0, -1, n),
            Clue::Right(n) => (0, 1, n),
            Clue::Down(n) => (1, 0, n),
        };
        let mut involving_cells = 0;
        {
            let mut y = y + dy;
            let mut x = x + dx;
            while self.clue.is_valid_coord((Y(y), X(x))) {
                let c = self.clue[(Y(y), X(x))];
                if c.same_shape(clue) {
                    n -= c.clue_number();
                    break;
                }
                y += dy;
                x += dx;
                involving_cells += 1;
            }
        }
        if involving_cells == 0 && n != 0 {
            self.set_inconsistent();
            return;
        }
        let mut dp_left = vec![(0, 0); (involving_cells + 1) as usize];
        let mut dp_right = vec![(0, 0); (involving_cells + 1) as usize];

        for i in 0..involving_cells {
            let c = self.get_cell((Y(y + dy * (i + 1)), X(x + dx * (i + 1))));
            dp_left[(i + 1) as usize] = match c {
                Cell::Undecided => {
                    let mut skip_three = false;
                    if i >= 2 && self.get_cell((Y(y + dy * i), X(x + dx * i))) != Cell::Clue {
                        if self
                            .get_cell_safe((Y(y + dy * i + dx), X(x + dx * i + dy)))
                            .is_blocking()
                            || self
                                .get_cell_safe((Y(y + dy * i - dx), X(x + dx * i - dy)))
                                .is_blocking()
                        {
                            skip_three = true;
                        }
                        if dx == 0
                            && self.technique.skip_three_from_blocked_either
                            && (self
                                .blocked_either_right
                                .get_or_default((Y(y + dy * i), X(x + dx * i - 1)), false)
                                || self
                                    .blocked_either_right
                                    .get_or_default((Y(y + dy * i), X(x + dx * i)), false))
                        {
                            skip_three = true;
                        }
                        if dy == 0
                            && (self
                                .blocked_either_down
                                .get_or_default((Y(y + dy * i - 1), X(x + dx * i)), false)
                                || self
                                    .blocked_either_down
                                    .get_or_default((Y(y + dy * i), X(x + dx * i)), false))
                        {
                            skip_three = true;
                        }
                    }
                    let (lo, hi) =
                        dp_left[cmp::max(0, i - if skip_three { 2 } else { 1 }) as usize];
                    (lo, hi + 1)
                }
                Cell::Clue | Cell::Line => dp_left[i as usize],
                Cell::Blocked => {
                    let (lo, hi) = dp_left[cmp::max(0, i - 1) as usize];
                    (lo + 1, hi + 1)
                }
            };
        }
        for i in 0..involving_cells {
            let i = involving_cells - 1 - i;
            let c = self.get_cell((Y(y + dy * (i + 1)), X(x + dx * (i + 1))));
            dp_right[i as usize] = match c {
                Cell::Undecided => {
                    let mut skip_three = false;
                    if i <= involving_cells - 3
                        && self.get_cell((Y(y + dy * (i + 2)), X(x + dx * (i + 2)))) != Cell::Clue
                    {
                        if self
                            .get_cell_safe((Y(y + dy * (i + 2) + dx), X(x + dx * (i + 2) + dy)))
                            .is_blocking()
                            || self
                                .get_cell_safe((Y(y + dy * (i + 2) - dx), X(x + dx * (i + 2) - dy)))
                                .is_blocking()
                        {
                            skip_three = true;
                        }
                        if dx == 0
                            && self.technique.skip_three_from_blocked_either
                            && (self.blocked_either_right.get_or_default(
                                (Y(y + dy * (i + 2)), X(x + dx * (i + 2) - 1)),
                                false,
                            ) || self
                                .blocked_either_right
                                .get_or_default((Y(y + dy * (i + 2)), X(x + dx * (i + 2))), false))
                        {
                            skip_three = true;
                        }
                        if dy == 0
                            && (self.blocked_either_down.get_or_default(
                                (Y(y + dy * (i + 2) - 1), X(x + dx * (i + 2))),
                                false,
                            ) || self
                                .blocked_either_down
                                .get_or_default((Y(y + dy * (i + 2)), X(x + dx * (i + 2))), false))
                        {
                            skip_three = true;
                        }
                    }
                    let (lo, hi) = dp_right
                        [cmp::min(involving_cells, i + if skip_three { 3 } else { 2 }) as usize];
                    (lo, hi + 1)
                }
                Cell::Clue | Cell::Line => dp_right[(i + 1) as usize],
                Cell::Blocked => {
                    let (lo, hi) = dp_right[cmp::min(involving_cells, i + 2) as usize];
                    (lo + 1, hi + 1)
                }
            };
        }
        for i in 0..involving_cells {
            let (left_lo, left_hi) = dp_left[i as usize];
            let (right_lo, right_hi) = dp_right[(i + 1) as usize];

            if left_hi + right_hi < n - 1 {
                self.set_inconsistent();
                return;
            } else if left_hi + right_hi == n - 1 {
                self.set_cell_internal((Y(y + dy * (i + 1)), X(x + dx * (i + 1))), Cell::Blocked);
            }

            if left_lo + right_lo > n {
                self.set_inconsistent();
                return;
            } else if left_lo + right_lo == n {
                if self.get_cell((Y(y + dy * (i + 1)), X(x + dx * (i + 1)))) != Cell::Clue {
                    self.set_cell_internal((Y(y + dy * (i + 1)), X(x + dx * (i + 1))), Cell::Line);
                }
            }

            if i != involving_cells - 1 && left_hi + dp_right[(i + 2) as usize].1 == n - 1 {
                let cell1 = (Y(y + dy * (i + 1)), X(x + dx * (i + 1)));
                let cell2 = (Y(y + dy * (i + 2)), X(x + dx * (i + 2)));
                if self.get_cell(cell1) != Cell::Clue && self.get_cell(cell2) != Cell::Clue {
                    self.set_blocked_either(cell1, cell2);
                }
            }
        }
    }
}

impl GridLoopField for Field {
    fn grid_loop(&mut self) -> &mut GridLoop {
        &mut self.grid_loop
    }
    fn check_neighborhood(&mut self, (Y(y), X(x)): Coord) {
        if y % 2 == 0 {
            GridLoop::check(self, (Y(y), X(x - 1)));
            GridLoop::check(self, (Y(y), X(x + 1)));
        } else {
            GridLoop::check(self, (Y(y - 1), X(x)));
            GridLoop::check(self, (Y(y + 1), X(x)));
        }
    }
    fn inspect(&mut self, cd: Coord) {
        let (Y(y), X(x)) = cd;
        if !(y % 2 == 0 && x % 2 == 0) {
            return;
        }
        let cy = y / 2;
        let cx = x / 2;
        let cell = self.get_cell((Y(cy), X(cx)));
        if cell == Cell::Line || cell == Cell::Undecided {
            let (n_line, n_undecided) = self.grid_loop.neighbor_summary(cd);

            if cell == Cell::Line {
                if n_line + n_undecided <= 1 {
                    self.set_inconsistent();
                    return;
                } else if n_line + n_undecided <= 2 {
                    for &(dy, dx) in &FOUR_NEIGHBORS {
                        if self.get_edge_safe((Y(y + dy), X(x + dx))) != Edge::Blank {
                            GridLoop::decide_edge(self, (Y(y + dy), X(x + dx)), Edge::Line);
                        }
                    }
                }
            } else {
                if n_line == 0 && n_undecided == 2 {
                    for &(dy, dx) in &FOUR_NEIGHBORS {
                        if self.get_edge_safe((Y(y + dy), X(x + dx))) == Edge::Undecided {
                            self.set_cell_internal((Y(y / 2 + dy), X(x / 2 + dx)), Cell::Line);
                        }
                    }
                }
            }

            if cell == Cell::Undecided {
                if n_line >= 1 {
                    self.set_cell_internal((Y(y / 2), X(x / 2)), Cell::Line);
                }
                if n_line == 0 && n_undecided <= 1 {
                    self.set_cell_internal((Y(y / 2), X(x / 2)), Cell::Blocked);
                }
            }
        } else if cell == Cell::Clue {
            self.inspect_clue((Y(y / 2), X(x / 2)));
        }

        if cy != self.height() - 1 && self.blocked_either_down[(Y(cy), X(cx))] {
            if cx != 0 && self.blocked_either_down[(Y(cy), X(cx - 1))] {
                self.two_by_two((Y(cy), X(cx - 1)));
            }
            if cx != self.width() - 1 && self.blocked_either_down[(Y(cy), X(cx + 1))] {
                self.two_by_two((Y(cy), X(cx)));
            }
        }
        if cx != self.width() - 1 && self.blocked_either_right[(Y(cy), X(cx))] {
            if cy != 0 && self.blocked_either_right[(Y(cy - 1), X(cx))] {
                self.two_by_two((Y(cy - 1), X(cx)));
            }
            if cy != self.height() - 1 && self.blocked_either_right[(Y(cy + 1), X(cx))] {
                self.two_by_two((Y(cy), X(cx)));
            }
        }
        self.check_two_by_three((Y(cy), X(cx)));
    }
}

struct UnionFind {
    parent: Vec<i32>,
}

impl UnionFind {
    fn new(size: usize) -> UnionFind {
        UnionFind {
            parent: vec![-1; size],
        }
    }
    fn root(&mut self, i: i32) -> i32 {
        if self.parent[i as usize] < 0 {
            i
        } else {
            let p = self.parent[i as usize];
            let ret = self.root(p);
            self.parent[i as usize] = ret;
            ret
        }
    }
    fn join(&mut self, u: i32, v: i32) -> bool {
        let u = self.root(u);
        let v = self.root(v);
        if u == v {
            return false;
        }
        self.parent[u as usize] += self.parent[v as usize];
        self.parent[v as usize] = u;
        true
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let height = self.height();
        let width = self.width();
        for y in 0..(2 * height - 1) {
            for x in 0..(2 * width - 1) {
                match (y % 2, x % 2) {
                    (0, 0) => match self.get_cell((Y(y / 2), X(x / 2))) {
                        Cell::Undecided => write!(f, "+")?,
                        Cell::Line => write!(f, "*")?,
                        Cell::Blocked => write!(f, "#")?,
                        Cell::Clue => match self.clue[(Y(y / 2), X(x / 2))] {
                            Clue::NoClue => panic!(),
                            Clue::Empty => write!(f, "? ")?,
                            Clue::Up(n) => write!(f, "^{}", n)?,
                            Clue::Left(n) => write!(f, "<{}", n)?,
                            Clue::Right(n) => write!(f, ">{}", n)?,
                            Clue::Down(n) => write!(f, "v{}", n)?,
                        },
                    },
                    (0, 1) => {
                        if self.get_cell((Y(y / 2), X(x / 2))) == Cell::Clue {
                            write!(f, "  ")?;
                        } else if self.get_cell((Y(y / 2), X(x / 2 + 1))) == Cell::Clue {
                            write!(f, "   ")?;
                        } else {
                            match self.get_edge((Y(y), X(x))) {
                                Edge::Line => write!(f, "---")?,
                                Edge::Blank => write!(f, " x ")?,
                                Edge::Undecided => write!(f, "   ")?,
                            }
                        }
                    }
                    (1, 0) => {
                        if self.get_cell((Y(y / 2), X(x / 2))) == Cell::Clue
                            || self.get_cell((Y(y / 2 + 1), X(x / 2))) == Cell::Clue
                        {
                            write!(f, " ")?;
                        } else {
                            match self.get_edge((Y(y), X(x))) {
                                Edge::Line => write!(f, "|")?,
                                Edge::Blank => write!(f, "x")?,
                                Edge::Undecided => write!(f, " ")?,
                            }
                        }
                    }
                    (1, 1) => write!(f, "   ")?,
                    _ => unreachable!(),
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yajilin_clue() {
        {
            // Loop must not pass through clue cells
            let mut problem = Grid::new(5, 5, Clue::NoClue);
            problem[(Y(0), X(0))] = Clue::Right(1);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_edge((Y(0), X(1))), Edge::Blank);
            assert_eq!(field.get_edge((Y(1), X(0))), Edge::Blank);
        }
        {
            // 0 block cells
            let mut problem = Grid::new(5, 7, Clue::NoClue);
            problem[(Y(0), X(3))] = Clue::Right(0);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell((Y(0), X(4))), Cell::Line);
            assert_eq!(field.get_cell((Y(0), X(5))), Cell::Line);
            assert_eq!(field.get_cell((Y(0), X(6))), Cell::Line);
            assert_eq!(field.get_edge((Y(1), X(8))), Edge::Line);
        }
        {
            // 2 block cells in 3 consecutive cells
            let mut problem = Grid::new(5, 5, Clue::NoClue);
            problem[(Y(2), X(1))] = Clue::Right(2);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.fully_solved(), true);
            assert_eq!(field.get_cell((Y(2), X(2))), Cell::Blocked);
            assert_eq!(field.get_cell((Y(2), X(4))), Cell::Blocked);
            assert_eq!(field.get_cell((Y(2), X(0))), Cell::Line);
            assert_eq!(field.get_edge((Y(3), X(6))), Edge::Line);
            assert_eq!(field.get_edge((Y(5), X(6))), Edge::Line);
            assert_eq!(field.get_edge((Y(2), X(3))), Edge::Line);
            assert_eq!(field.get_edge((Y(3), X(0))), Edge::Line);
        }
        {
            // Cells which the loop does not pass through must be blocked (or have a clue)
            let mut problem = Grid::new(6, 6, Clue::NoClue);
            problem[(Y(2), X(4))] = Clue::Up(1);
            problem[(Y(4), X(4))] = Clue::Left(1);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell((Y(1), X(4))), Cell::Blocked);
            assert_eq!(field.get_cell((Y(3), X(4))), Cell::Blocked);
        }
        {
            // Clues to the same direction
            let mut problem = Grid::new(6, 6, Clue::NoClue);
            problem[(Y(2), X(0))] = Clue::Right(2);
            problem[(Y(2), X(2))] = Clue::Right(1);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell((Y(2), X(1))), Cell::Blocked);
        }
        {
            // 3 block cells in 7 consecutive cells on the edge
            let mut problem = Grid::new(5, 8, Clue::NoClue);
            problem[(Y(0), X(0))] = Clue::Right(3);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell((Y(0), X(1))), Cell::Blocked);
            assert_eq!(field.get_cell((Y(0), X(4))), Cell::Blocked);
            assert_eq!(field.get_cell((Y(0), X(7))), Cell::Blocked);
        }
    }
}
