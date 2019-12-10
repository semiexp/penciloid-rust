use super::super::{Grid, D, LP, P};
use super::*;
use grid_loop::{Edge, GridLoop, GridLoopField};
use std::cmp;
use std::fmt;
use FOUR_NEIGHBOURS;

#[derive(Clone)]
pub struct Field {
    grid_loop: GridLoop,
    clue: Grid<Clue>,
    cell: Grid<Cell>,
    blocked_either_down: Grid<bool>,
    blocked_either_right: Grid<bool>,
    decided_cells: i32,
    technique: Technique,
}

impl Field {
    pub fn new(clue: &Grid<Clue>) -> Field {
        let height = clue.height();
        let width = clue.width();

        let mut cell = Grid::new(height, width, Cell::Undecided);
        let mut grid_loop = GridLoop::new(height - 1, width - 1);
        let mut decided_cells = 0;
        {
            let mut handle = GridLoop::get_handle(&mut grid_loop);
            for y in 0..height {
                for x in 0..width {
                    let pos = P(y, x);
                    let c = clue[pos];
                    if c != Clue::NoClue {
                        let pos_lp = LP::of_vertex(pos);
                        cell[pos] = Cell::Clue;
                        GridLoop::decide_edge(&mut *handle, pos_lp + D(-1, 0), Edge::Blank);
                        GridLoop::decide_edge(&mut *handle, pos_lp + D(1, 0), Edge::Blank);
                        GridLoop::decide_edge(&mut *handle, pos_lp + D(0, -1), Edge::Blank);
                        GridLoop::decide_edge(&mut *handle, pos_lp + D(0, 1), Edge::Blank);
                        decided_cells += 1;
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
            decided_cells,
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
    pub fn get_cell(&self, pos: P) -> Cell {
        self.cell[pos]
    }
    pub fn get_cell_safe(&self, pos: P) -> Cell {
        if self.cell.is_valid_p(pos) {
            self.cell[pos]
        } else {
            // The outside of the field can be identified with a (meaningless) clue
            Cell::Clue
        }
    }
    pub fn get_edge(&self, pos: LP) -> Edge {
        self.grid_loop.get_edge(pos)
    }
    pub fn get_edge_safe(&self, pos: LP) -> Edge {
        self.grid_loop.get_edge_safe(pos)
    }
    pub fn num_decided_cells(&self) -> i32 {
        self.decided_cells
    }

    pub fn set_cell(&mut self, pos: P, v: Cell) {
        let mut handle = GridLoop::get_handle(self);
        handle.set_cell_internal(pos, v);
    }
    pub fn check_all_cell(&mut self) {
        let height = self.height();
        let width = self.width();
        let mut handle = GridLoop::get_handle(self);
        for y in 0..height {
            for x in 0..width {
                GridLoop::check(&mut *handle, LP(y * 2, x * 2));
            }
        }
    }
    pub fn solve(&mut self) {
        loop {
            let current_decided_lines = self.grid_loop.num_decided_lines();
            let current_decided_cells = self.num_decided_cells();
            self.check_all_cell();
            GridLoop::apply_inout_rule(self);
            GridLoop::check_connectability(self);
            self.apply_inout_rule_advanced();
            self.check_local_parity();
            self.two_rows_entire_board();
            self.clue_counting();
            if current_decided_lines == self.grid_loop.num_decided_lines()
                && current_decided_cells == self.num_decided_cells()
            {
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

        let cell_id = |P(y, x)| {
            if 0 <= y && y < height && 0 <= x && x < width {
                (y * width + x)
            } else {
                outside
            }
        };

        let mut union_find = UnionFind::new((outside * 2 + 2) as usize);

        for y in 0..height {
            for x in 0..(width + 1) {
                match self.grid_loop.get_edge(LP(y * 2 + 1, x * 2)) {
                    Edge::Line => {
                        let u = cell_id(P(y, x - 1));
                        let v = cell_id(P(y, x));
                        union_find.join(u * 2, v * 2 + 1);
                        union_find.join(u * 2 + 1, v * 2);
                    }
                    Edge::Blank => {
                        let u = cell_id(P(y, x - 1));
                        let v = cell_id(P(y, x));
                        union_find.join(u * 2, v * 2);
                        union_find.join(u * 2 + 1, v * 2 + 1);
                    }
                    Edge::Undecided => (),
                }
                if self.blocked_either_down[P(y, x)]
                    && self.get_edge_safe(LP(y * 2 - 1, x * 2)) == Edge::Blank
                    && self.get_edge_safe(LP(y * 2 + 3, x * 2)) == Edge::Blank
                {
                    let u = cell_id(P(y - 1, x));
                    let v = cell_id(P(y + 1, x));
                    union_find.join(u * 2, v * 2 + 1);
                    union_find.join(u * 2 + 1, v * 2);
                }
            }
        }
        for y in 0..(height + 1) {
            for x in 0..width {
                match self.grid_loop.get_edge(LP(y * 2, x * 2 + 1)) {
                    Edge::Line => {
                        let u = cell_id(P(y - 1, x));
                        let v = cell_id(P(y, x));
                        union_find.join(u * 2, v * 2 + 1);
                        union_find.join(u * 2 + 1, v * 2);
                    }
                    Edge::Blank => {
                        let u = cell_id(P(y - 1, x));
                        let v = cell_id(P(y, x));
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
                if self.blocked_either_right[P(y, x)]
                    && self.get_edge_safe(LP(y * 2, x * 2 - 1)) == Edge::Blank
                    && self.get_edge_safe(LP(y * 2, x * 2 + 3)) == Edge::Blank
                {
                    let u = cell_id(P(y, x - 1));
                    let v = cell_id(P(y, x + 1));
                    union_find.join(u * 2, v * 2 + 1);
                    union_find.join(u * 2 + 1, v * 2);
                }
            }
        }

        for y in 0..height {
            for x in 0..(width + 1) {
                let u = cell_id(P(y, x - 1));
                let v = cell_id(P(y, x));

                if union_find.root(u * 2) == union_find.root(v * 2) {
                    GridLoop::decide_edge(self, LP(y * 2 + 1, x * 2), Edge::Blank);
                } else if union_find.root(u * 2) == union_find.root(v * 2 + 1) {
                    GridLoop::decide_edge(self, LP(y * 2 + 1, x * 2), Edge::Line);
                }
            }
        }
        for y in 0..(height + 1) {
            for x in 0..width {
                let u = cell_id(P(y - 1, x));
                let v = cell_id(P(y, x));

                if union_find.root(u * 2) == union_find.root(v * 2) {
                    GridLoop::decide_edge(self, LP(y * 2, x * 2 + 1), Edge::Blank);
                } else if union_find.root(u * 2) == union_find.root(v * 2 + 1) {
                    GridLoop::decide_edge(self, LP(y * 2, x * 2 + 1), Edge::Line);
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
            if ids[P(y, x)] != -1 {
                return;
            }
            ids[P(y, x)] = id;
            if y > 0 && grid_loop.get_edge(LP(2 * y - 1, 2 * x)) == Edge::Undecided {
                visit(y - 1, x, id, ids, grid_loop);
            }
            if y < ids.height() - 1 && grid_loop.get_edge(LP(2 * y + 1, 2 * x)) == Edge::Undecided {
                visit(y + 1, x, id, ids, grid_loop);
            }
            if x > 0 && grid_loop.get_edge(LP(2 * y, 2 * x - 1)) == Edge::Undecided {
                visit(y, x - 1, id, ids, grid_loop);
            }
            if x < ids.width() - 1 && grid_loop.get_edge(LP(2 * y, 2 * x + 1)) == Edge::Undecided {
                visit(y, x + 1, id, ids, grid_loop);
            }
        }
        for y in 0..height {
            for x in 0..width {
                if ids[P(y, x)] == -1 {
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
                let pos = LP(y, x);
                if self.get_edge(pos) == Edge::Line
                    && ids[P(y / 2, x / 2)] != ids[P((y + 1) / 2, (x + 1) / 2)]
                {
                    // waf
                    let id1 = ids[P(y / 2, x / 2)] as usize;
                    let id2 = ids[P((y + 1) / 2, (x + 1) / 2)] as usize;
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
                let id = ids[P(y, x)] as usize;
                num_cells[id] += 1;
                if self.get_cell(P(y, x)) == Cell::Undecided {
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
                self.set_cell(P(y, x), Cell::Line);
            } else {
                self.set_cell(P(y, x), Cell::Blocked);
            }
        }
    }
    pub fn trial_and_error(&mut self, depth: i32) {
        let height = self.height();
        let width = self.width();

        if depth == 0 {
            self.solve();
            return;
        }
        self.trial_and_error(depth - 1);

        loop {
            let mut updated = false;
            for y in 0..(height * 2 - 1) {
                for x in 0..(width * 2 - 1) {
                    if y % 2 == x % 2 {
                        continue;
                    }
                    let pos = LP(y, x);
                    if self.get_edge(pos) != Edge::Undecided {
                        continue;
                    }
                    if !self.grid_loop.is_root(pos) {
                        continue;
                    }

                    {
                        let mut field_line = self.clone();
                        GridLoop::decide_edge(&mut field_line, pos, Edge::Line);
                        field_line.trial_and_error(depth - 1);

                        if field_line.inconsistent() {
                            updated = true;
                            GridLoop::decide_edge(self, pos, Edge::Blank);
                            self.trial_and_error(depth - 1);
                        }
                    }
                    {
                        let mut field_blank = self.clone();
                        GridLoop::decide_edge(&mut field_blank, pos, Edge::Blank);
                        field_blank.trial_and_error(depth - 1);

                        if field_blank.inconsistent() {
                            updated = true;
                            GridLoop::decide_edge(self, pos, Edge::Line);
                            self.trial_and_error(depth - 1);
                        }
                    }
                    if self.inconsistent() {
                        return;
                    }
                }
            }
            for y in 0..height {
                for x in 0..width {
                    let pos = P(y, x);
                    if self.get_cell(pos) != Cell::Undecided {
                        continue;
                    }
                    {
                        let mut field_blocked = self.clone();
                        field_blocked.set_cell(pos, Cell::Blocked);
                        field_blocked.trial_and_error(depth - 1);

                        if field_blocked.inconsistent() {
                            updated = true;
                            self.set_cell(pos, Cell::Line);
                            self.trial_and_error(depth - 1);
                        }
                    }
                    {
                        let mut field_line = self.clone();
                        field_line.set_cell(pos, Cell::Line);
                        field_line.trial_and_error(depth - 1);

                        if field_line.inconsistent() {
                            updated = true;
                            self.set_cell(pos, Cell::Blocked);
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

    fn set_cell_internal(&mut self, pos: P, v: Cell) {
        let current = self.cell[pos];
        if current != Cell::Undecided {
            if current != v {
                self.set_inconsistent();
            }
            return;
        }

        self.decided_cells += 1;
        self.cell[pos] = v;
        match v {
            Cell::Undecided => (),
            Cell::Clue => (), // don't do this!
            Cell::Line => GridLoop::check(self, LP::of_vertex(pos)),
            Cell::Blocked => {
                for &d in &FOUR_NEIGHBOURS {
                    if self.get_cell_safe(pos + d) != Cell::Clue {
                        self.set_cell_internal(pos + d, Cell::Line);
                    }
                    GridLoop::decide_edge(self, LP::of_vertex(pos) + d, Edge::Blank);
                }
            }
        }
    }
    fn set_cell_internal_unless_clue(&mut self, pos: P, v: Cell) {
        if self.get_cell_safe(pos) == Cell::Clue {
            return;
        }
        self.set_cell_internal(pos, v);
    }
    fn set_blocked_either(&mut self, pos1: P, pos2: P) {
        if self.get_cell(pos1) == Cell::Clue || self.get_cell(pos2) == Cell::Clue {
            return;
        }

        let P(y1, x1) = pos1;
        let P(y2, x2) = pos2;

        if y1 == y2 {
            if x2 == x1 + 1 {
                self.blocked_either_right[P(y1, x1)] = true;
            } else if x1 == x2 + 1 {
                self.blocked_either_right[P(y1, x2)] = true;
            } else {
                panic!();
            }
        } else if x1 == x2 {
            if y2 == y1 + 1 {
                self.blocked_either_down[P(y1, x1)] = true;
            } else if y1 == y2 + 1 {
                self.blocked_either_down[P(y2, x1)] = true;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
    fn two_by_two(&mut self, top: P) {
        if !self.technique.two_by_two {
            return;
        }
        // 2x2 square (y, x) -- (y+1, x+1) has 2 blocked cells
        for &d in &[
            D(-1, 0),
            D(-1, 1),
            D(0, -1),
            D(0, 2),
            D(1, -1),
            D(1, 2),
            D(2, 0),
            D(2, 1),
        ] {
            self.set_cell_internal_unless_clue(top + d, Cell::Line);
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
        if self.blocked_either_right[P(y0, x0)] && self.blocked_either_right[P(y1, x1)] {
            if self.get_cell_safe(P(ty0, tx0)) != Cell::Clue
                && self.get_cell_safe(P(ty1, tx1)) != Cell::Clue
            {
                self.set_cell_internal(P(ty0, tx0), Cell::Line);
                self.set_cell_internal(P(ty1, tx1), Cell::Line);
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
        if self.blocked_either_right[P(y0, x0)] && self.blocked_either_down[P(y1, x1)] {
            if self.get_cell_safe(P(ty0, tx0)) != Cell::Clue
                && self.get_cell_safe(P(ty1, tx1)) != Cell::Clue
            {
                self.set_cell_internal(P(ty0, tx0), Cell::Line);
                self.set_cell_internal(P(ty1, tx1), Cell::Line);
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
        if self.blocked_either_down[P(y0, x0)] && self.blocked_either_down[P(y1, x1)] {
            if self.get_cell_safe(P(ty0, tx0)) != Cell::Clue
                && self.get_cell_safe(P(ty1, tx1)) != Cell::Clue
            {
                self.set_cell_internal(P(ty0, tx0), Cell::Line);
                self.set_cell_internal(P(ty1, tx1), Cell::Line);
            }
        }
    }
    fn check_two_by_three(&mut self, top: P) {
        if !self.technique.two_by_three {
            return;
        }
        let P(y, x) = top;

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
    fn around_blocked_either(&mut self, pos: P) {
        if pos.0 < self.height() - 1 && self.blocked_either_down[pos] {
            if !self.get_cell_safe(pos).can_be_blocked() {
                self.set_cell_internal(pos + D(1, 0), Cell::Blocked);
            }
            if !self.get_cell_safe(pos + D(1, 0)).can_be_blocked() {
                self.set_cell_internal(pos, Cell::Blocked);
            }
            if self.get_cell_safe(pos + D(-1, 0)) == Cell::Clue {
                self.set_cell_internal_unless_clue(pos + D(0, -1), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(0, 1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(2, 0)) == Cell::Clue {
                self.set_cell_internal_unless_clue(pos + D(1, -1), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(1, 1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(0, -1)) == Cell::Clue
                || self.get_cell_safe(pos + D(0, 1)) == Cell::Clue
            {
                self.set_cell_internal_unless_clue(pos + D(0, -1), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(-1, 0), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(0, 1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(1, -1)) == Cell::Clue
                || self.get_cell_safe(pos + D(1, 1)) == Cell::Clue
            {
                self.set_cell_internal_unless_clue(pos + D(1, -1), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(2, 0), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(1, 1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(0, -1)) != Cell::Clue
                && (self.get_cell_safe(pos + D(-1, -1)).is_blocking()
                    || self.get_cell_safe(pos + D(0, -2)).is_blocking())
            {
                self.set_cell_internal_unless_clue(pos + D(1, -1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(0, 1)) != Cell::Clue
                && (self.get_cell_safe(pos + D(-1, 1)).is_blocking()
                    || self.get_cell_safe(pos + D(0, 2)).is_blocking())
            {
                self.set_cell_internal_unless_clue(pos + D(1, 1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(1, -1)) != Cell::Clue
                && (self.get_cell_safe(pos + D(2, -1)).is_blocking()
                    || self.get_cell_safe(pos + D(1, -2)).is_blocking())
            {
                self.set_cell_internal_unless_clue(pos + D(0, -1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(1, 1)) != Cell::Clue
                && (self.get_cell_safe(pos + D(2, 1)).is_blocking()
                    || self.get_cell_safe(pos + D(1, 2)).is_blocking())
            {
                self.set_cell_internal_unless_clue(pos + D(0, 1), Cell::Line);
            }
        }
        if pos.1 < self.width() - 1 && self.blocked_either_right[pos] {
            if !self.get_cell_safe(pos).can_be_blocked() {
                self.set_cell_internal(pos + D(0, 1), Cell::Blocked);
            }
            if !self.get_cell_safe(pos + D(0, 1)).can_be_blocked() {
                self.set_cell_internal(pos, Cell::Blocked);
            }
            if self.get_cell_safe(pos + D(0, -1)) == Cell::Clue {
                self.set_cell_internal_unless_clue(pos + D(-1, 0), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(1, 0), Cell::Line);
            }
            if self.get_cell_safe(pos + D(0, 2)) == Cell::Clue {
                self.set_cell_internal_unless_clue(pos + D(-1, 1), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(1, 1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(-1, 0)) == Cell::Clue
                || self.get_cell_safe(pos + D(1, 0)) == Cell::Clue
            {
                self.set_cell_internal_unless_clue(pos + D(-1, 0), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(0, -1), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(1, 0), Cell::Line);
            }
            if self.get_cell_safe(pos + D(-1, 1)) == Cell::Clue
                || self.get_cell_safe(pos + D(1, 1)) == Cell::Clue
            {
                self.set_cell_internal_unless_clue(pos + D(-1, 1), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(0, 2), Cell::Line);
                self.set_cell_internal_unless_clue(pos + D(1, 1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(-1, 0)) != Cell::Clue
                && (self.get_cell_safe(pos + D(-1, -1)).is_blocking()
                    || self.get_cell_safe(pos + D(-2, 0)).is_blocking())
            {
                self.set_cell_internal_unless_clue(pos + D(-1, 1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(1, 0)) != Cell::Clue
                && (self.get_cell_safe(pos + D(1, -1)).is_blocking()
                    || self.get_cell_safe(pos + D(2, 0)).is_blocking())
            {
                self.set_cell_internal_unless_clue(pos + D(1, 1), Cell::Line);
            }
            if self.get_cell_safe(pos + D(-1, 1)) != Cell::Clue
                && (self.get_cell_safe(pos + D(-1, 2)).is_blocking()
                    || self.get_cell_safe(pos + D(-2, 1)).is_blocking())
            {
                self.set_cell_internal_unless_clue(pos + D(-1, 0), Cell::Line);
            }
            if self.get_cell_safe(pos + D(1, 1)) != Cell::Clue
                && (self.get_cell_safe(pos + D(1, 2)).is_blocking()
                    || self.get_cell_safe(pos + D(2, 1)).is_blocking())
            {
                self.set_cell_internal_unless_clue(pos + D(1, 0), Cell::Line);
            }
        }
    }
    fn two_rows_rule_sub(
        &mut self,
        base1: P,
        base2: P,
        dir: D,
        start1: i32,
        end1: i32,
        start2: i32,
        end2: i32,
        n_blocked_total: i32,
    ) {
        let count = cmp::max(end1, end2);
        let mut max_width1 = vec![1; count as usize];
        let mut max_width2 = vec![2; cmp::max(count - 1, 0) as usize];
        let mut max_width3 = vec![3; cmp::max(count - 1, 0) as usize];

        for i in 0..count {
            // width1
            {
                let c = if start1 <= i
                    && i < end1
                    && self.get_cell_safe(base1 + dir * i).can_be_blocked()
                {
                    1
                } else {
                    0
                } + if start2 <= i
                    && i < end2
                    && self.get_cell_safe(base2 + dir * i).can_be_blocked()
                {
                    1
                } else {
                    0
                };
                max_width1[i as usize] = cmp::min(1, c);
            }

            // width2
            if i < count - 1 {
                let c = if start1 <= i + 1 && i < end1 { 1 } else { 0 }
                    + if start2 <= i + 1 && i < end2 { 1 } else { 0 };
                let refined_c;
                if c == 2 {
                    let corner_a = base1 + dir * i;
                    let corner_b = base2 + dir * (i + 1);
                    let top = P(
                        cmp::min(corner_a.0, corner_b.0),
                        cmp::min(corner_a.1, corner_b.1),
                    );

                    let isok_a = self.get_cell_safe(top).can_be_blocked()
                        && self.get_cell_safe(top + D(1, 1)).can_be_blocked()
                        && self.get_cell_safe(top + D(0, 1)) != Cell::Blocked
                        && self.get_cell_safe(top + D(1, 0)) != Cell::Blocked
                        && (self.get_cell_safe(top + D(0, 1)) == Cell::Clue
                            || (!self.get_cell_safe(top + D(-1, 1)).is_blocking()
                                && !self.get_cell_safe(top + D(0, 2)).is_blocking()))
                        && (self.get_cell_safe(top + D(1, 0)) == Cell::Clue
                            || (!self.get_cell_safe(top + D(1, -1)).is_blocking()
                                && !self.get_cell_safe(top + D(2, 0)).is_blocking()));
                    let isok_b = self.get_cell_safe(top + D(0, 1)).can_be_blocked()
                        && self.get_cell_safe(top + D(1, 0)).can_be_blocked()
                        && self.get_cell_safe(top) != Cell::Blocked
                        && self.get_cell_safe(top + D(1, 1)) != Cell::Blocked
                        && (self.get_cell_safe(top) == Cell::Clue
                            || (!self.get_cell_safe(top + D(-1, 0)).is_blocking()
                                && !self.get_cell_safe(top + D(0, -1)).is_blocking()))
                        && (self.get_cell_safe(top + D(1, 1)) == Cell::Clue
                            || (!self.get_cell_safe(top + D(2, 1)).is_blocking()
                                && !self.get_cell_safe(top + D(1, 2)).is_blocking()));
                    if isok_a || isok_b {
                        refined_c = 2;
                    } else {
                        refined_c = 1;
                    }
                } else {
                    refined_c = c;
                }
                max_width2[i as usize] = refined_c;
            }

            // width3
            if i < count - 2 {
                // TODO: more strict bounds
                let c = if self.get_cell_safe(base1 + dir * (i + 1)) == Cell::Clue
                    || self.get_cell_safe(base2 + dir * (i + 1)) == Cell::Clue
                {
                    3
                } else {
                    2
                };
                max_width3[i as usize] = c;
            }
        }

        let mut dp_max_left = vec![count; (count + 1) as usize];
        let mut dp_max_right = vec![count; (count + 1) as usize];

        dp_max_left[0] = 0;
        for i in 0..(count as usize) {
            dp_max_left[i + 1] = cmp::min(dp_max_left[i + 1], dp_max_left[i] + max_width1[i]);
            if i + 1 < count as usize {
                dp_max_left[i + 2] = cmp::min(dp_max_left[i + 2], dp_max_left[i] + max_width2[i]);
            }
            if i + 2 < count as usize {
                dp_max_left[i + 3] = cmp::min(dp_max_left[i + 3], dp_max_left[i] + max_width3[i]);
            }
        }
        dp_max_right[count as usize] = 0;
        for i in 0..(count as usize) {
            let i = count as usize - i;
            dp_max_right[i - 1] =
                cmp::min(dp_max_right[i - 1], dp_max_right[i] + max_width1[i - 1]);
            if i >= 2 {
                dp_max_right[i - 2] =
                    cmp::min(dp_max_right[i - 2], dp_max_right[i] + max_width2[i - 2]);
            }
            if i >= 3 {
                dp_max_right[i - 3] =
                    cmp::min(dp_max_right[i - 3], dp_max_right[i] + max_width3[i - 3]);
            }
        }

        for i in 0..count {
            let cnt = dp_max_left[i as usize] + dp_max_right[(i + 1) as usize];
            if cnt == n_blocked_total - 1 {
                self.set_blocked_either(base1 + dir * i, base2 + dir * i);
            } else if cnt < n_blocked_total - 1 {
                self.set_inconsistent();
                return;
            }
        }
    }
    fn two_rows_by_two_clues(&mut self, pos1: P, pos2: P) {
        if pos1 == P(-1, -1) || pos2 == P(-1, -1) {
            return;
        }
        let (d1, cells1, blocks1) = self.clue_detail(pos1);
        let (d2, cells2, blocks2) = self.clue_detail(pos2);
        let P(y1, x1) = pos1;
        let P(y2, x2) = pos2;

        if d1 != d2 {
            panic!("two_rows_by_two_clues called with clues of different directions");
        }

        let (base1, base2) = match d1 {
            D(1, 0) => (P(cmp::min(y1, y2), x1), P(cmp::min(y1, y2), x2)),
            D(-1, 0) => (P(cmp::max(y1, y2), x1), P(cmp::max(y1, y2), x2)),
            D(0, 1) => (P(y1, cmp::min(x1, x2)), P(y2, cmp::min(x1, x2))),
            D(0, -1) => (P(y1, cmp::max(x1, x2)), P(y2, cmp::max(x1, x2))),
            _ => unreachable!(),
        };

        fn manhattan(d: D) -> i32 {
            let D(y, x) = d;
            y.abs() + x.abs()
        }
        let start1 = manhattan(pos1 - base1);
        let start2 = manhattan(pos2 - base2);
        self.two_rows_rule_sub(
            base1 + d1,
            base2 + d1,
            d1,
            start1,
            start1 + cells1,
            start2,
            start2 + cells2,
            blocks1 + blocks2,
        );
    }
    fn two_rows_entire_board(&mut self) {
        if !self.technique.two_rows {
            return;
        }
        let height = self.height();
        let width = self.width();

        // down
        for x in 0..(width - 1) {
            let mut clue1 = P(-1, -1);
            let mut clue2 = P(-1, -1);
            for y in 0..height {
                if self.clue[P(y, x)].get_direction() == D(1, 0) {
                    clue1 = P(y, x);
                    self.two_rows_by_two_clues(clue1, clue2);
                }
                if self.clue[P(y, x + 1)].get_direction() == D(1, 0) {
                    clue2 = P(y, x + 1);
                    self.two_rows_by_two_clues(clue1, clue2);
                }
            }
        }

        // up
        for x in 0..(width - 1) {
            let mut clue1 = P(-1, -1);
            let mut clue2 = P(-1, -1);
            for y in 0..height {
                let y = height - 1 - y;
                if self.clue[P(y, x)].get_direction() == D(-1, 0) {
                    clue1 = P(y, x);
                    self.two_rows_by_two_clues(clue1, clue2);
                }
                if self.clue[P(y, x + 1)].get_direction() == D(-1, 0) {
                    clue2 = P(y, x + 1);
                    self.two_rows_by_two_clues(clue1, clue2);
                }
            }
        }

        // right
        for y in 0..(height - 1) {
            let mut clue1 = P(-1, -1);
            let mut clue2 = P(-1, -1);
            for x in 0..width {
                if self.clue[P(y, x)].get_direction() == D(0, 1) {
                    clue1 = P(y, x);
                    self.two_rows_by_two_clues(clue1, clue2);
                }
                if self.clue[P(y + 1, x)].get_direction() == D(0, 1) {
                    clue2 = P(y + 1, x);
                    self.two_rows_by_two_clues(clue1, clue2);
                }
            }
        }

        // left
        for y in 0..(height - 1) {
            let mut clue1 = P(-1, -1);
            let mut clue2 = P(-1, -1);
            for x in 0..width {
                let x = width - 1 - x;
                if self.clue[P(y, x)].get_direction() == D(0, -1) {
                    clue1 = P(y, x);
                    self.two_rows_by_two_clues(clue1, clue2);
                }
                if self.clue[P(y + 1, x)].get_direction() == D(0, -1) {
                    clue2 = P(y + 1, x);
                    self.two_rows_by_two_clues(clue1, clue2);
                }
            }
        }
    }
    /// Computes the detailed information of the clue at `pos`.
    /// It returns a tuple (dir, n_cells, n_blocked), where
    /// - dir is the direction of the arrow of the clue,
    /// - n_cells is the number of cells between the clue and the next clue in this direction, and
    /// - n_blocked is the required number of blocked cells between these two clues.
    fn clue_detail(&self, pos: P) -> (D, i32, i32) {
        let clue = self.clue[pos];
        let (dy, dx, mut n) = match clue {
            Clue::NoClue | Clue::Empty => return (D(0, 0), 0, 0),
            Clue::Up(n) => (-1, 0, n),
            Clue::Left(n) => (0, -1, n),
            Clue::Right(n) => (0, 1, n),
            Clue::Down(n) => (1, 0, n),
        };
        let d = D(dy, dx);
        let mut involving_cells = 0;
        {
            let mut pos = pos + d;
            while self.clue.is_valid_p(pos) {
                let c = self.clue[pos];
                if c.same_shape(clue) {
                    n -= c.clue_number();
                    break;
                }
                pos = pos + d;
                involving_cells += 1;
            }
        }
        (d, involving_cells, n)
    }
    fn avoid_branching(&mut self, center: P) {
        if !self.technique.avoid_branching {
            return;
        }
        for &d in &FOUR_NEIGHBOURS {
            let dr = d.rotate_clockwise();
            let edge1 = self.get_edge_safe(LP::of_vertex(center) + d);
            let edge2 = self.get_edge_safe(LP::of_vertex(center) + dr);

            if !((edge1 == Edge::Line && edge2 == Edge::Blank)
                || (edge1 == Edge::Blank && edge2 == Edge::Line))
            {
                continue;
            }

            if !self.get_cell_safe(center - d).is_blocking()
                && !self.get_cell_safe(center - dr).is_blocking()
                && (self.get_cell_safe(center - d * 2).is_blocking()
                    || self.get_cell_safe(center - d + dr).is_blocking())
                && (self.get_cell_safe(center - dr * 2).is_blocking()
                    || self.get_cell_safe(center - dr + d).is_blocking())
            {
                self.set_cell_internal_unless_clue(center - d - dr, Cell::Line);
            }
        }
    }
    fn inspect_clue(&mut self, cell_cd: P) {
        let (d, involving_cells, n) = self.clue_detail(cell_cd);
        let D(dy, dx) = d;
        if d == D(0, 0) {
            return;
        }
        let dr = d.rotate_clockwise();
        if involving_cells == 0 && n != 0 {
            self.set_inconsistent();
            return;
        }

        let mut stride_two_low = vec![0; cmp::max(0, involving_cells - 1) as usize];
        let mut stride_three_hi = vec![2; cmp::max(0, involving_cells - 2) as usize];
        for i in 0..(involving_cells - 1) {
            if dy == 0 && self.blocked_either_right[cell_cd + d * (i + 1) + D(0, cmp::min(0, dx))] {
                stride_two_low[i as usize] = 1;
            }
            if dx == 0 && self.blocked_either_down[cell_cd + d * (i + 1) + D(cmp::min(0, dy), 0)] {
                stride_two_low[i as usize] = 1;
            }
        }
        for i in 0..(involving_cells - 2) {
            if self.get_cell(cell_cd + d * (i + 2)) == Cell::Clue {
                continue;
            }
            if self.get_cell_safe(cell_cd + d * (i + 2) + dr).is_blocking() {
                stride_three_hi[i as usize] = 1;
            }
            if self.get_cell_safe(cell_cd + d * (i + 2) - dr).is_blocking() {
                stride_three_hi[i as usize] = 1;
            }
            if dx == 0
                && self.technique.one_in_three_orthogonal_either
                && (self
                    .blocked_either_right
                    .get_or_default_p(cell_cd + d * (i + 2) + D(0, -1), false)
                    || self
                        .blocked_either_right
                        .get_or_default_p(cell_cd + d * (i + 2), false))
            {
                stride_three_hi[i as usize] = 1;
            }
            if dy == 0
                && self.technique.one_in_three_orthogonal_either
                && (self
                    .blocked_either_down
                    .get_or_default_p(cell_cd + d * (i + 2) + D(-1, 0), false)
                    || self
                        .blocked_either_down
                        .get_or_default_p(cell_cd + d * (i + 2), false))
            {
                stride_three_hi[i as usize] = 1;
            }
            if self.technique.one_in_three_remote {
                for &sgn in &[-1, 1] {
                    if self.get_cell_safe(cell_cd + d * (i + 1) + dr * sgn) != Cell::Clue
                        && self.get_cell_safe(cell_cd + d * (i + 3) + dr * sgn) != Cell::Clue
                        && (self.get_cell_safe(cell_cd + d * i + dr * sgn).is_blocking()
                            || self
                                .get_cell_safe(cell_cd + d * (i + 1) + dr * (sgn * 2))
                                .is_blocking())
                        && (self.get_cell_safe(cell_cd + d * (i + 4) + dr).is_blocking()
                            || self
                                .get_cell_safe(cell_cd + d * (i + 3) + dr * (sgn * 2))
                                .is_blocking())
                    {
                        stride_three_hi[i as usize] = 1;
                    }
                }
            }
        }
        let dummy_max = cmp::max(self.height(), self.width());
        let mut dp_left = vec![(0, dummy_max); (involving_cells + 1) as usize];
        let mut dp_right = vec![(0, dummy_max); (involving_cells + 1) as usize];
        dp_left[0] = (0, 0);
        dp_right[involving_cells as usize] = (0, 0);

        for i in 0..involving_cells {
            let (lo, hi) = dp_left[i as usize];
            let (nlo, nhi) = match self.get_cell(cell_cd + d * (i + 1)) {
                Cell::Blocked => (lo + 1, hi + 1),
                Cell::Undecided => (lo, hi + 1),
                Cell::Line | Cell::Clue => (lo, hi),
            };
            dp_left[(i + 1) as usize].0 = cmp::max(dp_left[(i + 1) as usize].0, nlo);
            dp_left[(i + 1) as usize].1 = cmp::min(dp_left[(i + 1) as usize].1, nhi);

            if i < involving_cells - 1 {
                dp_left[(i + 2) as usize].0 =
                    cmp::max(dp_left[(i + 2) as usize].0, lo + stride_two_low[i as usize]);
                dp_left[(i + 2) as usize].1 = cmp::min(dp_left[(i + 2) as usize].1, hi + 1);
            }
            if i < involving_cells - 2 {
                dp_left[(i + 3) as usize].1 = cmp::min(
                    dp_left[(i + 3) as usize].1,
                    hi + stride_three_hi[i as usize],
                );
            }
        }
        for i in 0..involving_cells {
            let i = involving_cells - i;
            let (lo, hi) = dp_right[i as usize];
            let (nlo, nhi) = match self.get_cell(cell_cd + d * i) {
                Cell::Blocked => (lo + 1, hi + 1),
                Cell::Undecided => (lo, hi + 1),
                Cell::Line | Cell::Clue => (lo, hi),
            };
            dp_right[(i - 1) as usize].0 = cmp::max(dp_right[(i - 1) as usize].0, nlo);
            dp_right[(i - 1) as usize].1 = cmp::min(dp_right[(i - 1) as usize].1, nhi);

            if i >= 2 {
                dp_right[(i - 2) as usize].0 = cmp::max(
                    dp_right[(i - 2) as usize].0,
                    lo + stride_two_low[(i - 2) as usize],
                );
                dp_right[(i - 2) as usize].1 = cmp::min(dp_right[(i - 2) as usize].1, hi + 1);
            }
            if i >= 3 {
                dp_right[(i - 3) as usize].1 = cmp::min(
                    dp_right[(i - 3) as usize].1,
                    hi + stride_three_hi[(i - 3) as usize],
                );
            }
        }

        for i in 0..involving_cells {
            let (left_lo, left_hi) = dp_left[i as usize];
            let (right_lo, right_hi) = dp_right[(i + 1) as usize];

            if left_hi + right_hi < n - 1 {
                self.set_inconsistent();
                return;
            } else if left_hi + right_hi == n - 1 {
                self.set_cell_internal(cell_cd + d * (i + 1), Cell::Blocked);
            }

            if left_lo + right_lo > n {
                self.set_inconsistent();
                return;
            } else if left_lo + right_lo == n {
                if self.get_cell(cell_cd + d * (i + 1)) != Cell::Clue {
                    self.set_cell_internal(cell_cd + d * (i + 1), Cell::Line);
                }
            }

            if i != involving_cells - 1 && left_hi + dp_right[(i + 2) as usize].1 == n - 1 {
                let cell1 = cell_cd + d * (i + 1);
                let cell2 = cell_cd + d * (i + 2);
                if self.get_cell(cell1) != Cell::Clue && self.get_cell(cell2) != Cell::Clue {
                    self.set_blocked_either(cell1, cell2);
                }
            }
        }
    }
    fn clue_counting(&mut self) {
        if !self.technique.clue_counting {
            return;
        }
        for y in 0..self.height() {
            self.clue_counting_single(y, true);
        }
        for x in 0..self.width() {
            self.clue_counting_single(x, false);
        }
    }
    fn clue_counting_single(&mut self, c: i32, horizontal: bool) {
        let start;
        let end;
        let dir;
        let cnt;
        if horizontal {
            start = P(c, 0);
            end = P(c, self.width());
            dir = D(0, 1);
            cnt = self.width() + 1;
        } else {
            start = P(0, c);
            end = P(self.height(), c);
            dir = D(1, 0);
            cnt = self.height() + 1;
        }
        let mut warshall_floyd = vec![cnt + 10; (cnt * cnt) as usize];
        {
            let mut set_distance = |s, d, v| {
                let pos = (s * cnt + d) as usize;
                warshall_floyd[pos] = cmp::min(warshall_floyd[pos], v);
            };
            for i in 0..(cnt - 1) {
                let pos = start + dir * i;
                let clue = self.clue[pos];
                match clue {
                    Clue::Left(n) => {
                        if horizontal {
                            set_distance(0, i, n);
                            set_distance(i, 0, -n);
                        }
                    }
                    Clue::Right(n) => {
                        if horizontal {
                            set_distance(i + 1, cnt - 1, n);
                            set_distance(cnt - 1, i + 1, n);
                        }
                    }
                    Clue::Up(n) => {
                        if !horizontal {
                            set_distance(0, i, n);
                            set_distance(i, 0, -n);
                        }
                    }
                    Clue::Down(n) => {
                        if !horizontal {
                            set_distance(i + 1, cnt - 1, n);
                            set_distance(cnt - 1, i + 1, n);
                        }
                    }
                    _ => (),
                }
                let cell = self.get_cell(pos);
                if cell == Cell::Blocked {
                    set_distance(i, i + 1, 1);
                    set_distance(i + 1, i, -1);
                } else if cell == Cell::Line || cell == Cell::Clue {
                    set_distance(i, i + 1, 0);
                    set_distance(i + 1, i, 0);
                } else {
                    set_distance(i, i + 1, 1);
                    set_distance(i + 1, i, 0);
                }
                if i < cnt - 2 {
                    set_distance(i, i + 2, 1);
                    if (horizontal && self.blocked_either_right[pos])
                        || (!horizontal && self.blocked_either_down[pos])
                    {
                        set_distance(i + 2, i, -1);
                    }
                }
            }
        }
        for i in 0..cnt {
            for j in 0..cnt {
                for k in 0..cnt {
                    warshall_floyd[(j * cnt + k) as usize] = cmp::min(
                        warshall_floyd[(j * cnt + k) as usize],
                        warshall_floyd[(j * cnt + i) as usize]
                            + warshall_floyd[(i * cnt + k) as usize],
                    );
                }
            }
        }
        for i in 0..(cnt - 1) {
            let pos = start + dir * i;
            {
                let d1 = warshall_floyd[(i * cnt + (i + 1)) as usize];
                let d2 = warshall_floyd[((i + 1) * cnt + i) as usize];
                if d1 == 0 && d2 == 0 && self.get_cell(pos) != Cell::Clue {
                    self.set_cell(pos, Cell::Line);
                }
                if d1 == 1 && d2 == -1 {
                    self.set_cell(pos, Cell::Blocked);
                }
            }
            if i < cnt - 2 {
                let d1 = warshall_floyd[(i * cnt + (i + 2)) as usize];
                let d2 = warshall_floyd[((i + 2) * cnt + i) as usize];
                if d1 == 1 && d2 == -1 {
                    self.set_blocked_either(pos, pos + dir);
                }
            }
        }
    }
}

impl GridLoopField for Field {
    fn grid_loop(&mut self) -> &mut GridLoop {
        &mut self.grid_loop
    }
    fn check_neighborhood(&mut self, pos: LP) {
        if pos.0 % 2 == 0 {
            GridLoop::check(self, pos + D(0, -1));
            GridLoop::check(self, pos + D(0, 1));
        } else {
            GridLoop::check(self, pos + D(-1, 0));
            GridLoop::check(self, pos + D(1, 0));
        }
    }
    fn inspect(&mut self, cd: LP) {
        if !cd.is_vertex() {
            return;
        }
        let cell_cd = cd.as_vertex();
        let cell = self.get_cell(cell_cd);
        if cell == Cell::Line || cell == Cell::Undecided {
            let (n_line, n_undecided) = self.grid_loop.neighbor_summary(cd);

            if cell == Cell::Line {
                if n_line + n_undecided <= 1 {
                    self.set_inconsistent();
                    return;
                } else if n_line + n_undecided <= 2 {
                    for &d in &FOUR_NEIGHBOURS {
                        if self.get_edge_safe(cd + d) != Edge::Blank {
                            GridLoop::decide_edge(self, cd + d, Edge::Line);
                        }
                    }
                }
                self.avoid_branching(cd.as_vertex());
            } else {
                if n_line == 0 && n_undecided == 2 {
                    for &d in &FOUR_NEIGHBOURS {
                        if self.get_edge_safe(cd + d) == Edge::Undecided {
                            self.set_cell_internal(cell_cd + d, Cell::Line);
                        }
                    }
                }
            }

            if cell == Cell::Undecided {
                if n_line >= 1 {
                    self.set_cell_internal(cell_cd, Cell::Line);
                }
                if n_line == 0 && n_undecided <= 1 {
                    self.set_cell_internal(cell_cd, Cell::Blocked);
                }
            }
        } else if cell == Cell::Clue {
            self.inspect_clue(cell_cd);
        }

        if cell_cd.0 != self.height() - 1 && self.blocked_either_down[cell_cd] {
            self.around_blocked_either(cell_cd);
            if cell_cd.1 != 0 && self.blocked_either_down[cell_cd + D(0, -1)] {
                self.two_by_two(cell_cd + D(0, -1));
            }
            if cell_cd.1 != self.width() - 1 && self.blocked_either_down[cell_cd + D(0, 1)] {
                self.two_by_two(cell_cd);
            }
        }
        if cell_cd.1 != self.width() - 1 && self.blocked_either_right[cell_cd] {
            self.around_blocked_either(cell_cd);
            if cell_cd.0 != 0 && self.blocked_either_right[cell_cd + D(-1, 0)] {
                self.two_by_two(cell_cd + D(-1, 0));
            }
            if cell_cd.0 != self.height() - 1 && self.blocked_either_right[cell_cd + D(1, 0)] {
                self.two_by_two(cell_cd);
            }
        }
        self.check_two_by_three(cell_cd);
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
                    (0, 0) => match self.get_cell(P(y / 2, x / 2)) {
                        Cell::Undecided => write!(f, "+")?,
                        Cell::Line => write!(f, "*")?,
                        Cell::Blocked => write!(f, "#")?,
                        Cell::Clue => match self.clue[P(y / 2, x / 2)] {
                            Clue::NoClue => panic!(),
                            Clue::Empty => write!(f, "? ")?,
                            Clue::Up(n) => write!(f, "^{}", n)?,
                            Clue::Left(n) => write!(f, "<{}", n)?,
                            Clue::Right(n) => write!(f, ">{}", n)?,
                            Clue::Down(n) => write!(f, "v{}", n)?,
                        },
                    },
                    (0, 1) => {
                        if self.get_cell(P(y / 2, x / 2)) == Cell::Clue {
                            write!(f, "  ")?;
                        } else if self.get_cell(P(y / 2, x / 2 + 1)) == Cell::Clue {
                            write!(f, "   ")?;
                        } else {
                            match self.get_edge(LP(y, x)) {
                                Edge::Line => write!(f, "---")?,
                                Edge::Blank => write!(f, " x ")?,
                                Edge::Undecided => write!(f, "   ")?,
                            }
                        }
                    }
                    (1, 0) => {
                        if self.get_cell(P(y / 2, x / 2)) == Cell::Clue
                            || self.get_cell(P(y / 2 + 1, x / 2)) == Cell::Clue
                        {
                            write!(f, " ")?;
                        } else {
                            match self.get_edge(LP(y, x)) {
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
            problem[P(0, 0)] = Clue::Right(1);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_edge(LP(0, 1)), Edge::Blank);
            assert_eq!(field.get_edge(LP(1, 0)), Edge::Blank);
        }
        {
            // 0 block cells
            let mut problem = Grid::new(5, 7, Clue::NoClue);
            problem[P(0, 3)] = Clue::Right(0);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell(P(0, 4)), Cell::Line);
            assert_eq!(field.get_cell(P(0, 5)), Cell::Line);
            assert_eq!(field.get_cell(P(0, 6)), Cell::Line);
            assert_eq!(field.get_edge(LP(1, 8)), Edge::Line);
        }
        {
            // 2 block cells in 3 consecutive cells
            let mut problem = Grid::new(5, 5, Clue::NoClue);
            problem[P(2, 1)] = Clue::Right(2);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.fully_solved(), true);
            assert_eq!(field.get_cell(P(2, 2)), Cell::Blocked);
            assert_eq!(field.get_cell(P(2, 4)), Cell::Blocked);
            assert_eq!(field.get_cell(P(2, 0)), Cell::Line);
            assert_eq!(field.get_edge(LP(3, 6)), Edge::Line);
            assert_eq!(field.get_edge(LP(5, 6)), Edge::Line);
            assert_eq!(field.get_edge(LP(2, 3)), Edge::Line);
            assert_eq!(field.get_edge(LP(3, 0)), Edge::Line);
        }
        {
            // Cells which the loop does not pass through must be blocked (or have a clue)
            let mut problem = Grid::new(6, 6, Clue::NoClue);
            problem[P(2, 4)] = Clue::Up(1);
            problem[P(4, 4)] = Clue::Left(1);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell(P(1, 4)), Cell::Blocked);
            assert_eq!(field.get_cell(P(3, 4)), Cell::Blocked);
        }
        {
            // Clues to the same direction
            let mut problem = Grid::new(6, 6, Clue::NoClue);
            problem[P(2, 0)] = Clue::Right(2);
            problem[P(2, 2)] = Clue::Right(1);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell(P(2, 1)), Cell::Blocked);
        }
        {
            // 3 block cells in 7 consecutive cells on the edge
            let mut problem = Grid::new(5, 8, Clue::NoClue);
            problem[P(0, 0)] = Clue::Right(3);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell(P(0, 1)), Cell::Blocked);
            assert_eq!(field.get_cell(P(0, 4)), Cell::Blocked);
            assert_eq!(field.get_cell(P(0, 7)), Cell::Blocked);
        }
        {
            // crossing (2 blocks in 4) and (2 blocks in 5)
            let mut problem = Grid::new(10, 10, Clue::NoClue);
            problem[P(2, 4)] = Clue::Down(2);
            problem[P(7, 4)] = Clue::Down(0);
            problem[P(4, 2)] = Clue::Right(2);
            problem[P(4, 8)] = Clue::Right(0);
            problem[P(5, 6)] = Clue::Up(0);

            let mut field = Field::new(&problem);
            field.check_all_cell();
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell(P(4, 7)), Cell::Blocked);
        }
    }
    #[test]
    fn test_one_in_three_remote() {
        {
            let mut problem = Grid::new(4, 8, Clue::NoClue);
            problem[P(2, 2)] = Clue::Right(2);

            let mut field = Field::new(&problem);
            field.check_all_cell();
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell(P(2, 3)), Cell::Blocked);
            assert_eq!(field.get_cell(P(2, 6)), Cell::Blocked);
        }
        {
            let mut problem = Grid::new(6, 6, Clue::NoClue);
            problem[P(1, 2)] = Clue::Right(1);
            problem[P(3, 4)] = Clue::Left(2);

            let mut field = Field::new(&problem);
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell(P(3, 3)), Cell::Blocked);
        }
    }
    #[test]
    fn test_two_rows() {
        {
            let mut problem = Grid::new(8, 8, Clue::NoClue);
            problem[P(3, 0)] = Clue::Right(1);
            problem[P(4, 0)] = Clue::Right(2);

            let mut field = Field::new(&problem);
            field.set_cell(P(3, 1), Cell::Line);
            field.set_cell(P(3, 2), Cell::Line);
            field.set_cell(P(3, 3), Cell::Line);
            field.set_cell(P(3, 7), Cell::Line);
            field.set_cell(P(4, 1), Cell::Line);
            field.set_cell(P(4, 3), Cell::Line);
            field.set_cell(P(4, 7), Cell::Line);

            field.solve();
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell(P(4, 2)), Cell::Blocked);
        }
        {
            let mut problem = Grid::new(10, 10, Clue::NoClue);
            problem[P(4, 0)] = Clue::Right(3);
            problem[P(5, 0)] = Clue::Right(2);

            let mut field = Field::new(&problem);
            field.set_cell(P(4, 6), Cell::Line);
            field.set_cell(P(4, 7), Cell::Line);
            field.set_cell(P(5, 7), Cell::Line);

            field.solve();
            field.check_all_cell();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell(P(5, 6)), Cell::Blocked);
        }
    }
    #[test]
    fn test_clue_counting() {
        {
            let mut problem = Grid::new(8, 8, Clue::NoClue);
            problem[P(3, 1)] = Clue::Right(1);
            problem[P(3, 5)] = Clue::Left(2);

            let mut field = Field::new(&problem);

            field.solve();

            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.get_cell(P(3, 0)), Cell::Blocked);
            assert_eq!(field.get_cell(P(3, 6)), Cell::Line);
            assert_eq!(field.get_cell(P(3, 7)), Cell::Line);
        }
    }
}
