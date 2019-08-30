use super::super::{GraphSeparation, Grid, D, P};
use super::{
    Cell, Clue, ConsecutiveRegionDictionary, Dictionary, CLUE_MAX, CLUE_VALUES,
    CONSECUTIVE_DICTIONARY_ADJACENCY_OFFSET, CONSECUTIVE_DICTIONARY_ADJACENCY_SIZE,
    DICTIONARY_INCONSISTENT, DICTIONARY_NEIGHBOR_OFFSET, DICTIONARY_NEIGHBOR_SIZE, NO_CLUE,
};
use std::cmp;
use std::fmt;

#[derive(Clone)]
pub struct Field<'a, 'b> {
    cell: Grid<Cell>,
    clue: Grid<Clue>,
    inconsistent: bool,
    decided_cells: i32,
    dic: &'a Dictionary,
    consecutive_dic: &'b ConsecutiveRegionDictionary,
    checking_region: Option<(P, P)>,
}

impl<'a, 'b> Field<'a, 'b> {
    pub fn new(
        height: i32,
        width: i32,
        dic: &'a Dictionary,
        consecutive_dic: &'b ConsecutiveRegionDictionary,
    ) -> Field<'a, 'b> {
        Field {
            cell: Grid::new(height, width, Cell::Undecided),
            clue: Grid::new(height, width, NO_CLUE),
            inconsistent: false,
            decided_cells: 0,
            dic,
            consecutive_dic,
            checking_region: None,
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
    pub fn clue(&self, loc: P) -> Clue {
        self.clue[loc]
    }
    pub fn add_clue(&mut self, loc: P, clue: Clue) {
        let current_clue = self.clue[loc];
        if current_clue != NO_CLUE {
            if current_clue != clue {
                self.inconsistent = true;
            }
            return;
        }

        self.clue[loc] = clue;
        self.decide(loc, Cell::White);
        self.inspect(loc);
    }
    pub fn cell(&self, loc: P) -> Cell {
        self.cell[loc]
    }
    pub fn cell_checked(&self, loc: P) -> Cell {
        if self.cell.is_valid_p(loc) {
            self.cell[loc]
        } else {
            Cell::White
        }
    }
    pub fn decide(&mut self, loc: P, v: Cell) {
        if let Some((P(y1, x1), P(y2, x2))) = self.checking_region {
            let P(y, x) = loc;
            if !(y1 <= y && y < y2 && x1 <= x && x < x2) {
                return;
            }
        }
        let current_status = self.cell_checked(loc);
        if current_status != Cell::Undecided {
            if current_status != v {
                self.inconsistent = true;
            }
            return;
        }

        self.cell[loc] = v;
        self.decided_cells += 1;

        if v == Cell::Black {
            self.avoid_cluster(loc + D(-1, -1), loc + D(-1, 0), loc + D(0, -1));
            self.avoid_cluster(loc + D(-1, 1), loc + D(-1, 0), loc + D(0, 1));
            self.avoid_cluster(loc + D(1, -1), loc + D(1, 0), loc + D(0, -1));
            self.avoid_cluster(loc + D(1, 1), loc + D(1, 0), loc + D(0, 1));
        }

        for dy in -1..2 {
            for dx in -1..2 {
                self.inspect(loc + D(dy, dx));
            }
        }
    }
    fn avoid_cluster(&mut self, loc1: P, loc2: P, loc3: P) {
        if self.cell_checked(loc1) == Cell::Black {
            if self.cell_checked(loc2) == Cell::Black {
                self.decide(loc3, Cell::White);
            }
            if self.cell_checked(loc3) == Cell::Black {
                self.decide(loc2, Cell::White);
            }
        } else {
            if self.cell_checked(loc2) == Cell::Black && self.cell_checked(loc3) == Cell::Black {
                self.decide(loc1, Cell::White);
            }
        }
    }
    pub fn inspect_connectivity(&mut self) {
        let height = self.height();
        let width = self.width();
        let cells = (height * width) as usize;
        let mut graph = GraphSeparation::new(cells, cells * 2);

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                let c = self.cell(pos);
                graph.set_weight(
                    (y * width + x) as usize,
                    if c == Cell::Black { 1 } else { 0 },
                );
                if c != Cell::White {
                    if self.cell_checked(pos + D(1, 0)) != Cell::White {
                        graph.add_edge((y * width + x) as usize, ((y + 1) * width + x) as usize);
                    }
                    if self.cell_checked(pos + D(0, 1)) != Cell::White {
                        graph.add_edge((y * width + x) as usize, (y * width + (x + 1)) as usize);
                    }
                }
            }
        }

        graph.build();

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if self.cell(pos) == Cell::Undecided {
                    let sep = graph.separate((y * width + x) as usize);
                    let mut nonzero = 0;
                    for v in sep {
                        if v > 0 {
                            nonzero += 1;
                        }
                    }
                    if nonzero >= 2 {
                        self.decide(pos, Cell::Black);
                    }
                }
            }
        }
    }
    pub fn inspect_connectivity_advanced(&mut self) {
        let height = self.height();
        let width = self.width();
        let cells = (height * width) as usize;
        let mut graph = GraphSeparation::new(cells, cells * 2);
        let mut activated_cell = Grid::new(height, width, true);

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if self.cell(pos) == Cell::White {
                    activated_cell[pos] = false;
                }
            }
        }

        let mut edge_down = Grid::new(height, width, true);
        let mut edge_right = Grid::new(height, width, true);

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                let clue = self.clue(pos);
                if clue != NO_CLUE {
                    let Clue(c) = clue;
                    if CLUE_MAX[c as usize] <= 1 {
                        // clues containing only 1's

                        for dy in -1..2 {
                            for dx in -1..2 {
                                let cd2 = pos + D(dy, dx);
                                if edge_down.is_valid_p(cd2) {
                                    if dy != 1 {
                                        edge_down[cd2] = false;
                                    }
                                    if dx != 1 {
                                        edge_right[cd2] = false;
                                    }
                                }
                            }
                        }
                    }

                    let mut virtual_disconnection_pattern = 0u32;
                    let mut pow3 = 1;
                    for i in 0..8 {
                        let d = DICTIONARY_NEIGHBOR_OFFSET[i];
                        let v = self.cell_checked(pos + d);
                        virtual_disconnection_pattern += pow3
                            * match v {
                                Cell::Undecided => 0,
                                Cell::Black => 1,
                                Cell::White => 2,
                            };
                        pow3 *= 3;
                    }

                    let res = self
                        .dic
                        .virtual_disconnection(clue, virtual_disconnection_pattern);
                    if res != 0 {
                        for i in 0..8 {
                            if ((res >> i) & 1) != 0 {
                                let d = DICTIONARY_NEIGHBOR_OFFSET[i];
                                if i < 2 {
                                    edge_down[pos + d] = false;
                                } else if i < 4 {
                                    edge_right[pos + d] = false;
                                } else if i < 6 {
                                    edge_down[pos + d + D(-1, 0)] = false;
                                } else {
                                    edge_right[pos + d + D(0, -1)] = false;
                                }
                            }
                        }
                    }

                    let mut virtually_ignored_cell_pattern = 0u32;
                    for i in 0..8 {
                        let d = DICTIONARY_NEIGHBOR_OFFSET[i];
                        let v = self.cell_checked(pos + d);

                        if v == Cell::White {
                            virtually_ignored_cell_pattern |= 3 << (i * 2);
                        } else {
                            let mut is_degree_2;
                            if d.0 == 0 || d.1 == 0 {
                                is_degree_2 = self.cell_checked(pos + d * 2) == Cell::White;
                            } else {
                                is_degree_2 = self.cell_checked(P(y + d.0, x + d.1 * 2))
                                    == Cell::White
                                    && self.cell_checked(P(y + d.0 * 2, x + d.1)) == Cell::White;
                            }
                            {
                                let d = DICTIONARY_NEIGHBOR_OFFSET[(i + 1) % 8];
                                if self.cell_checked(pos + d) == Cell::White {
                                    is_degree_2 = false;
                                }
                                let d = DICTIONARY_NEIGHBOR_OFFSET[(i + 7) % 8];
                                if self.cell_checked(pos + d) == Cell::White {
                                    is_degree_2 = false;
                                }
                            }

                            if is_degree_2 {
                                virtually_ignored_cell_pattern |= 1 << (i * 2);
                            } else {
                                if v == Cell::Black {
                                    virtually_ignored_cell_pattern |= 2 << (i * 2);
                                }
                            }
                        }
                    }

                    let res = self
                        .dic
                        .virtually_ignored_cell(clue, virtually_ignored_cell_pattern);
                    if res != 0 {
                        for i in 0..8 {
                            if ((res >> i) & 1) != 0 {
                                let d = DICTIONARY_NEIGHBOR_OFFSET[i];
                                activated_cell[pos + d] = false;
                            }
                        }
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                let c = self.cell(pos);
                graph.set_weight(
                    (y * width + x) as usize,
                    if c == Cell::Black && activated_cell[pos] {
                        1
                    } else {
                        0
                    },
                );
                if c != Cell::White {
                    if edge_down[pos] && y < height - 1 && activated_cell[pos + D(1, 0)] {
                        graph.add_edge((y * width + x) as usize, ((y + 1) * width + x) as usize);
                    }
                    if edge_right[pos] && x < width - 1 && activated_cell[pos + D(0, 1)] {
                        graph.add_edge((y * width + x) as usize, (y * width + (x + 1)) as usize);
                    }
                }
            }
        }

        graph.build();

        let mut black_root = None;
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if self.cell(pos) == Cell::Black && activated_cell[pos] {
                    let root = graph.union_root((y * width + x) as usize);
                    match black_root {
                        Some(b) => {
                            if b != root {
                                self.inconsistent = true;
                                return;
                            }
                        }
                        None => black_root = Some(root),
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if self.cell(pos) == Cell::Undecided && activated_cell[pos] {
                    let root = graph.union_root((y * width + x) as usize);
                    if let Some(b) = black_root {
                        if b != root {
                            self.decide(pos, Cell::White);
                            continue;
                        }
                    }
                    let sep = graph.separate((y * width + x) as usize);
                    let mut nonzero = 0;
                    for v in sep {
                        if v > 0 {
                            nonzero += 1;
                        }
                    }
                    if nonzero >= 2 {
                        self.decide(pos, Cell::Black);
                    }
                }
            }
        }
    }
    pub fn inspect_connectivity_clue_aware(&mut self) {
        let height = self.height();
        let width = self.width();

        let mut id = Grid::new(height, width, -1);
        let mut parent_id = Grid::new(height, width, -1);
        let mut lowlink = Grid::new(height, width, -1);
        let mut weight = Grid::new(height, width, 0);

        let mut correction_value = Grid::new(height, width, [0, 0, 0, 0]);

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                match self.cell(pos) {
                    Cell::Black => weight[pos] += 1,
                    Cell::White => {
                        if self.clue(pos) != NO_CLUE {
                            let mut neighbor_pattern = 0u32;
                            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                                let d = DICTIONARY_NEIGHBOR_OFFSET[i];
                                if self.cell_checked(pos + d) != Cell::White {
                                    neighbor_pattern |= 1u32 << i;
                                }
                            }
                            let clue = self.clue(pos);
                            let affected_neighbors =
                                self.consecutive_dic.consult(clue, neighbor_pattern);

                            for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                                let d = DICTIONARY_NEIGHBOR_OFFSET[i];
                                let pos2 = pos + d;
                                if id.is_valid_p(pos2) {
                                    weight[pos2] += 1;
                                    for j in 0..CONSECUTIVE_DICTIONARY_ADJACENCY_SIZE {
                                        correction_value[pos2][j] += self
                                            .consecutive_dic
                                            .consult_removal(affected_neighbors, i, j);
                                    }
                                }
                            }
                        }
                    }
                    Cell::Undecided => (),
                }
            }
        }

        fn visit(
            cd: P,
            cd_parent: P,
            cell: &Grid<Cell>,
            id: &mut Grid<i32>,
            parent_id: &mut Grid<i32>,
            lowlink: &mut Grid<i32>,
            weight: &mut Grid<i32>,
            id_last: &mut i32,
        ) {
            if cell[cd] == Cell::White {
                return;
            }
            id[cd] = *id_last;
            lowlink[cd] = *id_last;
            *id_last += 1;

            if cd_parent != P(-1, -1) {
                parent_id[cd] = id[cd_parent];
            }

            // TODO: rewrite more clearly
            for &d in &CONSECUTIVE_DICTIONARY_ADJACENCY_OFFSET {
                let cd2 = cd + d;
                if cell.is_valid_p(cd2) {
                    if cd_parent == cd2 || cell[cd2] == Cell::White {
                        continue;
                    }
                    if id[cd2] == -1 {
                        visit(cd2, cd, cell, id, parent_id, lowlink, weight, id_last);
                        lowlink[cd] = cmp::min(lowlink[cd], lowlink[cd2]);
                        weight[cd] += weight[cd2];
                    } else {
                        lowlink[cd] = cmp::min(lowlink[cd], id[cd2]);
                    }
                }
            }
        }

        let mut total_weight = 0;

        'outer: for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if self.cell(pos) != Cell::White {
                    let mut id_last = 0;
                    visit(
                        pos,
                        P(-1, -1),
                        &self.cell,
                        &mut id,
                        &mut parent_id,
                        &mut lowlink,
                        &mut weight,
                        &mut id_last,
                    );
                    total_weight = weight[pos];
                    break 'outer;
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if self.cell(pos) != Cell::White && id[pos] == -1 {
                    self.inconsistent = true;
                    return;
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                // TODO: use `pos` instead of `cd`
                let cd = P(y, x);
                let cell = self.cell(cd);
                if id[cd] == -1 {
                    continue;
                }
                if cell == Cell::Undecided {
                    let mut local_weights = [0, 0, 0, 0];
                    let mut parent_dir = 4;
                    let mut parent_weight = 0;
                    for i in 0..4 {
                        let d = CONSECUTIVE_DICTIONARY_ADJACENCY_OFFSET[i];
                        let cd2 = cd + d;
                        if !id.is_valid_p(cd2) || id[cd2] == -1 {
                            continue;
                        }
                        if parent_id[cd] == id[cd2] {
                            // cd2 <- cd
                            parent_dir = i;
                            parent_weight += total_weight - weight[cd] - correction_value[cd][i];
                        } else if parent_id[cd2] == id[cd] {
                            // cd <- cd2
                            let w = weight[cd2] - correction_value[cd][i];
                            if lowlink[cd2] < id[cd] {
                                parent_weight += w;
                            } else {
                                local_weights[i] = w;
                            }
                        } else {
                            // non-DFS edge
                            if id[cd] < id[cd2] {
                                let mut closest = (0, 0);
                                for j in 0..4 {
                                    let d = CONSECUTIVE_DICTIONARY_ADJACENCY_OFFSET[j];
                                    let cd3 = cd + d;
                                    if !id.is_valid_p(cd3) || id[cd3] == -1 {
                                        continue;
                                    }
                                    if parent_id[cd3] == id[cd] && id[cd3] <= id[cd2] {
                                        closest = cmp::max(closest, (id[cd3], j));
                                    }
                                }
                                local_weights[closest.1] -= correction_value[cd][i];
                            } else {
                                parent_weight -= correction_value[cd][i];
                            }
                        }
                    }
                    if parent_dir != 4 {
                        local_weights[parent_dir] = parent_weight;
                    }
                    let mut nonzero = 0;
                    for i in 0..4 {
                        if local_weights[i] > 0 {
                            nonzero += 1;
                        }
                    }
                    if nonzero >= 2 {
                        self.decide(cd, Cell::Black);
                    }
                }
            }
        }
    }
    fn inspect(&mut self, loc: P) {
        if !self.cell.is_valid_p(loc) {
            return;
        }

        let clue = self.clue[loc];
        if clue != NO_CLUE {
            let mut neighbor = 0;
            let mut pow = 1;
            for i in 0..8 {
                let d = DICTIONARY_NEIGHBOR_OFFSET[i];
                neighbor += pow
                    * match self.cell_checked(loc + d) {
                        Cell::Undecided => 0,
                        Cell::Black => 1,
                        Cell::White => 2,
                    };
                pow *= 3;
            }
            let neighbor = self.dic.neighbor_pattern_raw(clue, neighbor);

            if neighbor == DICTIONARY_INCONSISTENT {
                self.inconsistent = true;
                return;
            }

            for i in 0..8 {
                let v = (neighbor >> (2 * i)) & 3;
                let d = DICTIONARY_NEIGHBOR_OFFSET[i];
                if v == 1 {
                    self.decide(loc + d, Cell::Black);
                } else if v == 2 {
                    self.decide(loc + d, Cell::White);
                }
            }
        }
    }
    pub fn solve(&mut self) {
        while !self.inconsistent {
            let decided_cells = self.decided_cells;
            self.inspect_connectivity();
            self.inspect_connectivity_advanced();
            if self.decided_cells == decided_cells {
                break;
            }
        }
    }
    pub fn trial_and_error(&mut self) {
        let height = self.height();
        let width = self.width();
        let mut updated = true;
        while updated {
            self.solve();
            if self.inconsistent {
                break;
            }
            updated = false;
            for y in 0..height {
                for x in 0..width {
                    let pos = P(y, x);
                    if self.cell(pos) == Cell::Undecided {
                        let checking_region = (pos + D(-2, -2), pos + D(3, 3));
                        let mut trial_black = self.clone();
                        trial_black.checking_region = Some(checking_region);
                        trial_black.decide(pos, Cell::Black);
                        trial_black.solve();

                        if trial_black.inconsistent() {
                            self.decide(pos, Cell::White);
                            self.solve();
                            updated = true;
                        }

                        let mut trial_white = self.clone();
                        trial_white.checking_region = Some(checking_region);
                        trial_white.decide(pos, Cell::White);
                        trial_white.solve();

                        if trial_white.inconsistent() {
                            self.decide(pos, Cell::Black);
                            self.solve();
                            updated = true;
                        }
                    }
                    if self.inconsistent {
                        break;
                    }
                }
            }
        }
    }
}

impl<'a, 'b> fmt::Display for Field<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let height = self.height();
        let width = self.width();
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                match self.cell(pos) {
                    Cell::Undecided => write!(f, ".... ")?,
                    Cell::Black => write!(f, "#### ")?,
                    Cell::White => {
                        let clue = self.clue(pos);
                        if clue == NO_CLUE {
                            write!(f, "____ ")?;
                        } else {
                            let Clue(id) = clue;
                            if id == 0 {
                                write!(f, "0____ ")?;
                            } else {
                                for i in 0..4 {
                                    let v = CLUE_VALUES[id as usize][i];
                                    if v == -1 {
                                        write!(f, "_")?;
                                    } else {
                                        write!(f, "{}", v)?;
                                    }
                                }
                                write!(f, " ")?;
                            }
                        }
                    }
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::clue_pattern_to_id;
    use super::*;

    #[test]
    fn test_tapa_field_clues() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        {
            let mut field = Field::new(5, 6, &dic, &consecutive_dic);
            field.add_clue(P(2, 1), clue_pattern_to_id(&[]).unwrap());
            field.add_clue(P(2, 3), clue_pattern_to_id(&[4]).unwrap());

            assert_eq!(field.cell(P(2, 0)), Cell::White);
            assert_eq!(field.cell(P(1, 4)), Cell::Black);
            assert_eq!(field.cell(P(2, 4)), Cell::Black);
            assert_eq!(field.cell(P(3, 4)), Cell::Black);
            assert_eq!(field.inconsistent(), false);
        }
        {
            let mut field = Field::new(5, 6, &dic, &consecutive_dic);
            field.add_clue(P(1, 1), clue_pattern_to_id(&[]).unwrap());
            field.add_clue(P(2, 2), clue_pattern_to_id(&[8]).unwrap());

            assert_eq!(field.inconsistent(), true);
        }
    }

    #[test]
    fn test_tapa_field_cluster() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        let mut field = Field::new(5, 6, &dic, &consecutive_dic);
        field.decide((P(1, 1)), Cell::Black);
        field.decide((P(1, 2)), Cell::Black);
        field.decide((P(2, 2)), Cell::Black);

        assert_eq!(field.cell(P(2, 1)), Cell::White);
        assert_eq!(field.inconsistent(), false);
    }

    #[test]
    fn test_tapa_field_connectivity() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        let mut field = Field::new(5, 6, &dic, &consecutive_dic);
        field.decide(P(0, 0), Cell::Black);
        field.decide(P(4, 5), Cell::Black);
        field.decide(P(1, 0), Cell::White);
        field.decide(P(2, 1), Cell::White);
        field.decide(P(0, 3), Cell::White);
        field.decide(P(0, 2), Cell::Undecided);
        field.decide(P(1, 1), Cell::Undecided);

        field.inspect_connectivity();

        assert_eq!(field.cell(P(0, 1)), Cell::Black);
        assert_eq!(field.cell(P(1, 2)), Cell::Black);
        assert_eq!(field.inconsistent(), false);
    }

    #[test]
    fn test_tapa_field_virtually_ignored_cells() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        let mut field = Field::new(5, 7, &dic, &consecutive_dic);
        field.add_clue(P(1, 3), clue_pattern_to_id(&[1, 2]).unwrap());
        field.add_clue(P(3, 3), clue_pattern_to_id(&[1, 3]).unwrap());
        field.decide(P(0, 0), Cell::Black);
        field.decide(P(0, 6), Cell::Black);

        field.inspect_connectivity_advanced();

        assert_eq!(field.cell(P(4, 2)), Cell::Black);
        assert_eq!(field.cell(P(4, 3)), Cell::Black);
        assert_eq!(field.cell(P(4, 4)), Cell::Black);
    }

    #[test]
    fn test_tapa_field_virtual_disconnection() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        let mut field = Field::new(5, 5, &dic, &consecutive_dic);
        field.add_clue(P(1, 2), clue_pattern_to_id(&[1, 2]).unwrap());
        field.decide(P(0, 1), Cell::Black);
        field.decide(P(0, 2), Cell::Black);
        field.decide(P(3, 2), Cell::White);
        field.decide(P(4, 4), Cell::Black);

        field.inspect_connectivity_advanced();

        assert_eq!(field.cell(P(4, 1)), Cell::Black);
        assert_eq!(field.cell(P(4, 2)), Cell::Black);
        assert_eq!(field.cell(P(4, 3)), Cell::Black);
        assert_eq!(field.inconsistent(), false);
    }

    #[test]
    fn test_tapa_field_problem() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        {
            let mut field = Field::new(6, 5, &dic, &consecutive_dic);
            field.add_clue(P(1, 0), clue_pattern_to_id(&[1, 3]).unwrap());
            field.add_clue(P(1, 2), clue_pattern_to_id(&[2, 4]).unwrap());
            field.add_clue(P(3, 1), clue_pattern_to_id(&[3, 3]).unwrap());
            field.add_clue(P(4, 3), clue_pattern_to_id(&[4]).unwrap());

            field.inspect_connectivity();
            field.inspect_connectivity();
            field.inspect_connectivity();

            let expected = [
                [1, 1, 1, 1, 1],
                [0, 1, 0, 0, 1],
                [1, 0, 1, 1, 1],
                [1, 0, 1, 0, 0],
                [1, 0, 1, 0, 0],
                [1, 1, 1, 1, 0],
            ];
            for y in 0..6 {
                for x in 0..5 {
                    assert_eq!(
                        field.cell(P(y, x)),
                        if expected[y as usize][x as usize] == 1 {
                            Cell::Black
                        } else {
                            Cell::White
                        }
                    );
                }
            }
            assert_eq!(field.inconsistent(), false);
            assert_eq!(field.decided_cells(), 30);
            assert_eq!(field.fully_solved(), true);
        }
        {
            let mut field = Field::new(2, 7, &dic, &consecutive_dic);
            field.add_clue(P(0, 0), clue_pattern_to_id(&[1]).unwrap());
            field.add_clue(P(1, 2), clue_pattern_to_id(&[3]).unwrap());
            field.add_clue(P(0, 5), clue_pattern_to_id(&[2]).unwrap());

            field.inspect_connectivity_clue_aware();
            field.inspect_connectivity_clue_aware();

            let expected = [[0, 1, 1, 1, 1, 0, 0], [0, 0, 0, 0, 1, 0, 0]];
            for y in 0..2 {
                for x in 0..7 {
                    assert_eq!(
                        field.cell(P(y, x)),
                        if expected[y as usize][x as usize] == 1 {
                            Cell::Black
                        } else {
                            Cell::White
                        }
                    );
                }
            }
            assert_eq!(field.inconsistent(), false);
        }
    }

    #[test]
    fn test_tapa_field_connectivity_clue_aware() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        {
            let mut field = Field::new(5, 6, &dic, &consecutive_dic);
            field.add_clue(P(1, 3), clue_pattern_to_id(&[]).unwrap());
            field.add_clue(P(4, 0), clue_pattern_to_id(&[1]).unwrap());
            field.add_clue(P(0, 0), clue_pattern_to_id(&[1]).unwrap());
            field.add_clue(P(4, 5), clue_pattern_to_id(&[1]).unwrap());

            field.inspect_connectivity_clue_aware();

            assert_eq!(field.cell(P(3, 1)), Cell::Black);
            assert_eq!(field.inconsistent(), false);
        }
        {
            let mut field = Field::new(2, 2, &dic, &consecutive_dic);
            field.add_clue(P(0, 0), clue_pattern_to_id(&[1]).unwrap());

            field.inspect_connectivity_clue_aware();

            assert_eq!(field.cell(P(0, 1)), Cell::Undecided);
            assert_eq!(field.cell(P(1, 0)), Cell::Undecided);
            assert_eq!(field.cell(P(1, 1)), Cell::Undecided);
            assert_eq!(field.inconsistent(), false);
        }
    }
}
