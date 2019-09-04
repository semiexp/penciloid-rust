use super::*;
use std::fmt;
use GraphSeparation;
use {Grid, D, FOUR_NEIGHBOURS, P};

#[derive(Clone)]
pub struct Field {
    cell: Grid<Cell>,
    decided_cells: i32,
    inconsistent: bool,
}

impl Field {
    pub fn new(problem: &Grid<Option<i32>>) -> Field {
        let height = problem.height();
        let width = problem.width();
        let mut cell = Grid::new(height, width, Cell::Undecided);
        let mut decided_cells = 0;

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);

                if let Some(n) = problem[pos] {
                    cell[pos] = Cell::Cape(n);
                    decided_cells += 1;
                }
            }
        }

        Field {
            cell,
            decided_cells,
            inconsistent: false,
        }
    }
    pub fn height(&self) -> i32 {
        self.cell.height()
    }
    pub fn width(&self) -> i32 {
        self.cell.width()
    }
    pub fn inconsistent(&self) -> bool {
        self.inconsistent
    }
    pub fn set_inconsistent(&mut self) {
        self.inconsistent = true;
    }
    pub fn decided_cells(&self) -> i32 {
        self.decided_cells
    }
    pub fn fully_solved(&self) -> bool {
        self.decided_cells == self.height() * self.width()
    }
    pub fn get_cell(&self, pos: P) -> Cell {
        self.cell[pos]
    }
    pub fn decide_cell(&mut self, pos: P, val: Cell) {
        let current_cell = self.cell[pos];

        if current_cell != Cell::Undecided {
            match (current_cell, val) {
                (Cell::Black, Cell::Black)
                | (Cell::White, Cell::White)
                | (Cell::Cape(_), Cell::White) => (),
                _ => self.set_inconsistent(),
            }
            return;
        }

        self.cell[pos] = val;
        self.decided_cells += 1;

        self.avoid_2x2_cluster(pos);
        self.avoid_2x2_cluster(pos + D(-1, -1));
        self.avoid_2x2_cluster(pos + D(-1, 0));
        self.avoid_2x2_cluster(pos + D(0, -1));
    }
    pub fn inspect_all_cell(&mut self) {
        let height = self.height();
        let width = self.width();

        for y in 0..height {
            for x in 0..width {
                self.inspect(P(y, x));
            }
        }
    }
    pub fn solve(&mut self) {
        loop {
            let last_decided_cells = self.decided_cells();

            self.inspect_all_cell();
            self.avoid_forbidden_pattern_simple();
            self.ensure_connectivity(false);
            self.ensure_connectivity(true);

            if last_decided_cells == self.decided_cells() {
                break;
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
            for y in 0..height {
                for x in 0..width {
                    let pos = P(y, x);
                    if self.get_cell(pos) != Cell::Undecided {
                        continue;
                    }
                    {
                        let mut field_blocked = self.clone();
                        field_blocked.decide_cell(pos, Cell::Black);
                        field_blocked.trial_and_error(depth - 1);

                        if field_blocked.inconsistent() {
                            updated = true;
                            self.decide_cell(pos, Cell::White);
                            self.trial_and_error(depth - 1);
                        }
                    }
                    {
                        let mut field_line = self.clone();
                        field_line.decide_cell(pos, Cell::White);
                        field_line.trial_and_error(depth - 1);

                        if field_line.inconsistent() {
                            updated = true;
                            self.decide_cell(pos, Cell::Black);
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
    fn avoid_2x2_cluster(&mut self, top: P) {
        let P(y, x) = top;
        if !(0 <= y && y < self.height() - 1 && 0 <= x && x < self.width() - 1) {
            return;
        }

        let related = [D(0, 0), D(0, 1), D(1, 0), D(1, 1)];
        let mut n_black = 0;
        let mut n_white = 0;
        for &d in &related {
            match self.get_cell(top + d) {
                Cell::White | Cell::Cape(_) => n_white += 1,
                Cell::Black => n_black += 1,
                _ => (),
            }
        }

        if n_black == 3 && n_white == 0 {
            for &d in &related {
                if self.get_cell(top + d) == Cell::Undecided {
                    self.decide_cell(top + d, Cell::White);
                }
            }
        } else if n_black == 0 && n_white == 3 {
            for &d in &related {
                if self.get_cell(top + d) == Cell::Undecided {
                    self.decide_cell(top + d, Cell::Black);
                }
            }
        }
    }
    fn is_black_or_outside(&self, pos: P) -> bool {
        !self.cell.is_valid_p(pos) || self.get_cell(pos) == Cell::Black
    }
    fn single_forbidden_pattern(&mut self, pos: [P; 4]) {
        for i in 0..4 {
            let mut flg = true;
            for j in 0..4 {
                if i != j {
                    if !self.is_black_or_outside(pos[j]) {
                        flg = false;
                        break;
                    }
                }
            }
            if flg {
                if !self.cell.is_valid_p(pos[i]) {
                    self.set_inconsistent();
                } else {
                    self.decide_cell(pos[i], Cell::White);
                }
            }
        }
    }
    fn avoid_forbidden_pattern_simple(&mut self) {
        let height = self.height();
        let width = self.width();

        // cup technique
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if self.get_cell(pos).is_cape() {
                    continue;
                }

                if y < height - 1 && !self.get_cell(pos + D(1, 0)).is_cape() {
                    if x != 0 {
                        self.single_forbidden_pattern([
                            pos + D(-1, 0),
                            pos + D(0, -1),
                            pos + D(1, -1),
                            pos + D(2, 0),
                        ]);
                    }
                    if x != width - 1 {
                        self.single_forbidden_pattern([
                            pos + D(-1, 0),
                            pos + D(0, 1),
                            pos + D(1, 1),
                            pos + D(2, 0),
                        ]);
                    }
                }
                if x < width - 1 && !self.get_cell(pos + D(0, 1)).is_cape() {
                    if y != 0 {
                        self.single_forbidden_pattern([
                            pos + D(0, -1),
                            pos + D(-1, 0),
                            pos + D(-1, 1),
                            pos + D(0, 2),
                        ]);
                    }
                    if y != height - 1 {
                        self.single_forbidden_pattern([
                            pos + D(0, -1),
                            pos + D(1, 0),
                            pos + D(1, 1),
                            pos + D(0, 2),
                        ]);
                    }
                }
            }
        }
    }
    fn is_bad_cape_direction(&self, pos: P, n: i32, dir: D) -> bool {
        if n <= 0 {
            if !self.cell.is_valid_p(pos + dir) || self.get_cell(pos + dir) == Cell::Black {
                return true;
            }
            return false;
        }
        let end = pos + dir * (n - 1);
        if !self.cell.is_valid_p(end) {
            return true;
        }
        if self.cell.is_valid_p(end + dir) && self.cell[end + dir].is_white_like() {
            return true;
        }
        for i in 1..n {
            let c = self.cell[pos + dir * i];
            match c {
                Cell::Black => return true,
                Cell::Cape(n2) => {
                    if i != n - 1 || n != n2 {
                        return true;
                    }
                }
                _ => (),
            }
        }
        false
    }
    fn decide_cape_direction(&mut self, pos: P, n: i32, dir: D) {
        for &d in &FOUR_NEIGHBOURS {
            if d != dir && self.cell.is_valid_p(pos + d) {
                self.decide_cell(pos + d, Cell::Black);
            }
        }
        if n > 1 {
            if !self.cell.is_valid_p(pos + dir * (n - 1)) {
                self.set_inconsistent();
                return;
            }
            for i in 1..n {
                self.decide_cell(pos + dir * i, Cell::White);
            }
            if self.cell.is_valid_p(pos + dir * n) {
                self.decide_cell(pos + dir * n, Cell::Black);
            }
        }
    }
    fn inspect_clue(&mut self, pos: P) {
        if let Cell::Cape(n) = self.cell[pos] {
            for &d in &FOUR_NEIGHBOURS {
                if self.cell.is_valid_p(pos + d) && self.cell[pos + d] == Cell::White {
                    // direction decided
                    self.decide_cape_direction(pos, n, d);
                    return;
                }
            }

            let mut good_dir = D(0, 0);
            let mut n_good_dirs = 0;
            for &d in &FOUR_NEIGHBOURS {
                if !self.is_bad_cape_direction(pos, n, d) {
                    good_dir = d;
                    n_good_dirs += 1;
                } else {
                    if self.cell.is_valid_p(pos + d) {
                        self.decide_cell(pos + d, Cell::Black);
                    }
                }
            }

            if n_good_dirs == 1 {
                self.decide_cape_direction(pos, n, good_dir);
            } else if n_good_dirs == 0 {
                self.set_inconsistent();
            }
        }
    }
    fn inspect(&mut self, pos: P) {
        if let Cell::Cape(_) = self.get_cell(pos) {
            self.inspect_clue(pos);
        } else {
            let mut n_adjacent_white = 0;
            let mut n_adjacent_undecided = 0;

            for &d in &FOUR_NEIGHBOURS {
                let p = pos + d;
                if self.cell.is_valid_p(p) {
                    match self.get_cell(p) {
                        Cell::Undecided => n_adjacent_undecided += 1,
                        Cell::White | Cell::Cape(_) => n_adjacent_white += 1,
                        _ => (),
                    }
                }
            }

            match (n_adjacent_white, n_adjacent_undecided) {
                (0, 0) | (0, 1) | (1, 0) => self.decide_cell(pos, Cell::Black),
                (0, 2) | (1, 1) => {
                    if self.get_cell(pos) == Cell::White {
                        for &d in &FOUR_NEIGHBOURS {
                            if self.cell.is_valid_p(pos + d)
                                && self.get_cell(pos + d) == Cell::Undecided
                            {
                                self.decide_cell(pos + d, Cell::White);
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }
    fn ensure_connectivity(&mut self, ignore_capes: bool) {
        let height = self.height();
        let width = self.width();
        let mut graph =
            GraphSeparation::new((height * width) as usize, (height * width * 2) as usize);

        let is_considered_cell =
            |cell: Cell| !(cell == Cell::Black || (ignore_capes && cell.is_cape()));

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                let cell = self.get_cell(pos);

                if !is_considered_cell(cell) {
                    continue;
                }
                if cell.is_white_like() {
                    graph.set_weight((y * width + x) as usize, 1);
                }
                if y < height - 1 && is_considered_cell(self.get_cell(pos + D(1, 0))) {
                    graph.add_edge((y * width + x) as usize, ((y + 1) * width + x) as usize);
                }
                if x < width - 1 && is_considered_cell(self.get_cell(pos + D(0, 1))) {
                    graph.add_edge((y * width + x) as usize, (y * width + x + 1) as usize);
                }
            }
        }

        graph.build();

        let mut global_root = None;
        for y in 0..height {
            for x in 0..width {
                let cell = self.get_cell(P(y, x));
                if cell == Cell::Undecided {
                    let sep = graph.separate((y * width + x) as usize);
                    let mut nonzero = 0;
                    for v in sep {
                        if v > 0 {
                            nonzero += 1;
                        }
                    }
                    if nonzero >= 2 {
                        self.decide_cell(P(y, x), Cell::White);
                    }
                } else if cell.is_white_like() {
                    if ignore_capes && cell.is_cape() {
                        continue;
                    }
                    let root = graph.union_root((y * width + x) as usize);
                    match global_root {
                        None => global_root = Some(root),
                        Some(n) => {
                            if n != root {
                                self.set_inconsistent();
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let height = self.height();
        let width = self.width();
        for y in 0..height {
            for x in 0..width {
                match self.get_cell(P(y, x)) {
                    Cell::Black => write!(f, "#")?,
                    Cell::White => write!(f, " ")?,
                    Cell::Undecided => write!(f, ".")?,
                    Cell::Cape(n) => {
                        if n >= 1 {
                            write!(f, "{}", n)?;
                        } else {
                            write!(f, "?")?;
                        }
                    }
                }
                if x == width - 1 {
                    write!(f, "\n")?;
                } else {
                    write!(f, " ")?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem() {
        let mut problem = Grid::new(4, 4, None);
        problem[P(0, 0)] = Some(4);
        problem[P(1, 1)] = Some(2);
        problem[P(0, 3)] = Some(3);

        let mut field = Field::new(&problem);
        field.solve();

        assert_eq!(field.inconsistent(), false);
        assert_eq!(field.fully_solved(), true);
        assert_eq!(field.get_cell(P(3, 2)), Cell::White);
    }
}
