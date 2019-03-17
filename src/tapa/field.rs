use super::super::{Coord, GraphSeparation, Grid, X, Y};
use super::{Cell, Clue, ConsecutiveRegionDictionary, Dictionary, CLUE_MAX, CLUE_VALUES,
            CONSECUTIVE_DICTIONARY_ADJACENCY_OFFSET, CONSECUTIVE_DICTIONARY_ADJACENCY_SIZE,
            DICTIONARY_INCONSISTENT, DICTIONARY_NEIGHBOR_OFFSET, DICTIONARY_NEIGHBOR_SIZE, NO_CLUE};
use std::fmt;
use std::cmp;

#[derive(Clone)]
pub struct Field<'a, 'b> {
    cell: Grid<Cell>,
    clue: Grid<Clue>,
    inconsistent: bool,
    decided_cells: i32,
    dic: &'a Dictionary,
    consecutive_dic: &'b ConsecutiveRegionDictionary,
    checking_region: Option<(Coord, Coord)>,
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
    pub fn clue(&self, loc: Coord) -> Clue {
        self.clue[loc]
    }
    pub fn add_clue(&mut self, loc: Coord, clue: Clue) {
        let current_clue = self.clue[loc];
        if current_clue != NO_CLUE {
            if current_clue != clue {
                self.inconsistent = true;
            }
            return;
        }

        self.clue[loc] = clue;
        self.decide(loc, Cell::White);
    }
    pub fn cell(&self, loc: Coord) -> Cell {
        self.cell[loc]
    }
    pub fn cell_checked(&self, loc: Coord) -> Cell {
        if self.cell.is_valid_coord(loc) {
            self.cell[loc]
        } else {
            Cell::White
        }
    }
    pub fn decide(&mut self, loc: Coord, v: Cell) {
        if let Some(((Y(y1), X(x1)), (Y(y2), X(x2)))) = self.checking_region {
            let (Y(y), X(x)) = loc;
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

        let (Y(y), X(x)) = loc;

        if v == Cell::Black {
            self.avoid_cluster((Y(y - 1), X(x - 1)), (Y(y - 1), X(x)), (Y(y), X(x - 1)));
            self.avoid_cluster((Y(y - 1), X(x + 1)), (Y(y - 1), X(x)), (Y(y), X(x + 1)));
            self.avoid_cluster((Y(y + 1), X(x - 1)), (Y(y + 1), X(x)), (Y(y), X(x - 1)));
            self.avoid_cluster((Y(y + 1), X(x + 1)), (Y(y + 1), X(x)), (Y(y), X(x + 1)));
        }

        for dy in -1..2 {
            for dx in -1..2 {
                self.inspect((Y(y + dy), X(x + dx)));
            }
        }
    }
    fn avoid_cluster(&mut self, loc1: Coord, loc2: Coord, loc3: Coord) {
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
                let c = self.cell((Y(y), X(x)));
                graph.set_weight(
                    (y * width + x) as usize,
                    if c == Cell::Black { 1 } else { 0 },
                );
                if c != Cell::White {
                    if self.cell_checked((Y(y + 1), X(x))) != Cell::White {
                        graph.add_edge((y * width + x) as usize, ((y + 1) * width + x) as usize);
                    }
                    if self.cell_checked((Y(y), X(x + 1))) != Cell::White {
                        graph.add_edge((y * width + x) as usize, (y * width + (x + 1)) as usize);
                    }
                }
            }
        }

        graph.build();

        for y in 0..height {
            for x in 0..width {
                if self.cell((Y(y), X(x))) == Cell::Undecided {
                    let sep = graph.separate((y * width + x) as usize);
                    let mut nonzero = 0;
                    for v in sep {
                        if v > 0 {
                            nonzero += 1;
                        }
                    }
                    if nonzero >= 2 {
                        self.decide((Y(y), X(x)), Cell::Black);
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
                if self.cell((Y(y), X(x))) == Cell::White {
                    activated_cell[(Y(y), X(x))] = false;
                }
            }
        }

        let mut edge_down = Grid::new(height, width, true);
        let mut edge_right = Grid::new(height, width, true);

        for y in 0..height {
            for x in 0..width {
                let clue = self.clue((Y(y), X(x)));
                if clue != NO_CLUE {
                    let Clue(c) = clue;
                    if CLUE_MAX[c as usize] <= 1 {
                        // clues containing only 1's

                        for dy in -1..2 {
                            for dx in -1..2 {
                                let cd2 = (Y(y + dy), X(x + dx));
                                if edge_down.is_valid_coord(cd2) {
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

                    let mut virtually_ignored_cell_pattern = 0u32;
                    for i in 0..8 {
                        let (Y(dy), X(dx)) = DICTIONARY_NEIGHBOR_OFFSET[i];
                        let v = self.cell_checked((Y(y + dy), X(x + dx)));

                        if v == Cell::White {
                            virtually_ignored_cell_pattern |= (3 << (i * 2));
                        } else {
                            let mut is_degree_2;
                            if dy == 0 || dx == 0 {
                                is_degree_2 = self.cell_checked((Y(y + dy * 2), X(x + dx * 2)))
                                    == Cell::White;
                            } else {
                                is_degree_2 = self.cell_checked((Y(y + dy), X(x + dx * 2)))
                                    == Cell::White
                                    && self.cell_checked((Y(y + dy * 2), X(x + dx))) == Cell::White;
                            }
                            {
                                let (Y(dy), X(dx)) = DICTIONARY_NEIGHBOR_OFFSET[(i + 1) % 8];
                                if self.cell_checked((Y(y + dy), X(x + dx))) == Cell::White {
                                    is_degree_2 = false;
                                }
                                let (Y(dy), X(dx)) = DICTIONARY_NEIGHBOR_OFFSET[(i + 7) % 8];
                                if self.cell_checked((Y(y + dy), X(x + dx))) == Cell::White {
                                    is_degree_2 = false;
                                }
                            }

                            if is_degree_2 {
                                virtually_ignored_cell_pattern |= (1 << (i * 2));
                            } else {
                                if v == Cell::Black {
                                    virtually_ignored_cell_pattern |= (2 << (i * 2));
                                }
                            }
                        }
                    }

                    let res = self.dic
                        .virtually_ignored_cell(clue, virtually_ignored_cell_pattern);
                    if res != 0 {
                        for i in 0..8 {
                            if ((res >> i) & 1) != 0 {
                                let (Y(dy), X(dx)) = DICTIONARY_NEIGHBOR_OFFSET[i];
                                activated_cell[(Y(y + dy), X(x + dx))] = false;
                            }
                        }
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let c = self.cell((Y(y), X(x)));
                graph.set_weight(
                    (y * width + x) as usize,
                    if c == Cell::Black && activated_cell[(Y(y), X(x))] {
                        1
                    } else {
                        0
                    },
                );
                if c != Cell::White {
                    if edge_down[(Y(y), X(x))] && y < height - 1 && activated_cell[(Y(y + 1), X(x))]
                    {
                        graph.add_edge((y * width + x) as usize, ((y + 1) * width + x) as usize);
                    }
                    if edge_right[(Y(y), X(x))] && x < width - 1 && activated_cell[(Y(y), X(x + 1))]
                    {
                        graph.add_edge((y * width + x) as usize, (y * width + (x + 1)) as usize);
                    }
                }
            }
        }

        graph.build();

        let mut black_root = None;
        for y in 0..height {
            for x in 0..width {
                if self.cell((Y(y), X(x))) == Cell::Black && activated_cell[(Y(y), X(x))] {
                    let root = graph.union_root((y * width + x) as usize);
                    match black_root {
                        Some(b) => if b != root {
                            self.inconsistent = true;
                            return;
                        },
                        None => black_root = Some(root),
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                if self.cell((Y(y), X(x))) == Cell::Undecided && activated_cell[(Y(y), X(x))] {
                    let root = graph.union_root((y * width + x) as usize);
                    if let Some(b) = black_root {
                        if b != root {
                            self.decide((Y(y), X(x)), Cell::White);
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
                        self.decide((Y(y), X(x)), Cell::Black);
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
                let cd = (Y(y), X(x));
                match self.cell(cd) {
                    Cell::Black => weight[cd] += 1,
                    Cell::White => if self.clue(cd) != NO_CLUE {
                        let mut neighbor_pattern = 0u32;
                        for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                            let (Y(dy), X(dx)) = DICTIONARY_NEIGHBOR_OFFSET[i];
                            if self.cell_checked((Y(y + dy), X(x + dx))) != Cell::White {
                                neighbor_pattern |= 1u32 << i;
                            }
                        }
                        let clue = self.clue(cd);
                        let affected_neighbors =
                            self.consecutive_dic.consult(clue, neighbor_pattern);

                        for i in 0..DICTIONARY_NEIGHBOR_SIZE {
                            let (Y(dy), X(dx)) = DICTIONARY_NEIGHBOR_OFFSET[i];
                            let cd2 = (Y(y + dy), X(x + dx));
                            if id.is_valid_coord(cd2) {
                                weight[cd2] += 1;
                                for j in 0..CONSECUTIVE_DICTIONARY_ADJACENCY_SIZE {
                                    correction_value[cd2][j] += self.consecutive_dic
                                        .consult_removal(affected_neighbors, i, j);
                                }
                            }
                        }
                    },
                    Cell::Undecided => (),
                }
            }
        }

        fn visit(
            cd: Coord,
            cd_parent: Coord,
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

            if cd_parent != (Y(-1), X(-1)) {
                parent_id[cd] = id[cd_parent];
            }

            let (Y(y), X(x)) = cd;

            for &(Y(dy), X(dx)) in &CONSECUTIVE_DICTIONARY_ADJACENCY_OFFSET {
                let y2 = y + dy;
                let x2 = x + dx;
                if 0 <= y2 && y2 < cell.height() && 0 <= x2 && x2 < cell.width() {
                    let cd2 = (Y(y2), X(x2));
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
                if self.cell((Y(y), X(x))) != Cell::White {
                    let mut id_last = 0;
                    visit(
                        (Y(y), X(x)),
                        (Y(-1), X(-1)),
                        &self.cell,
                        &mut id,
                        &mut parent_id,
                        &mut lowlink,
                        &mut weight,
                        &mut id_last,
                    );
                    total_weight = weight[(Y(y), X(x))];
                    break 'outer;
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                if self.cell((Y(y), X(x))) != Cell::White && id[(Y(y), X(x))] == -1 {
                    self.inconsistent = true;
                    return;
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let cd = (Y(y), X(x));
                let cell = self.cell(cd);
                if id[cd] == -1 {
                    continue;
                }
                if cell == Cell::Undecided {
                    let mut local_weights = [0, 0, 0, 0];
                    let mut parent_dir = 4;
                    let mut parent_weight = 0;
                    for i in 0..4 {
                        let (Y(dy), X(dx)) = CONSECUTIVE_DICTIONARY_ADJACENCY_OFFSET[i];
                        let cd2 = (Y(y + dy), X(x + dx));
                        if !id.is_valid_coord(cd2) || id[cd2] == -1 {
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
                                    let (Y(dy), X(dx)) = CONSECUTIVE_DICTIONARY_ADJACENCY_OFFSET[j];
                                    let cd3 = (Y(y + dy), X(x + dx));
                                    if !id.is_valid_coord(cd3) || id[cd3] == -1 {
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
    fn inspect(&mut self, loc: Coord) {
        if !self.cell.is_valid_coord(loc) {
            return;
        }

        let (Y(y), X(x)) = loc;
        let cell = self.cell(loc);
        let clue = self.clue[loc];
        if clue != NO_CLUE {
            let mut neighbor = 0;
            let mut pow = 1;
            for i in 0..8 {
                let (Y(dy), X(dx)) = DICTIONARY_NEIGHBOR_OFFSET[i];
                neighbor += pow * match self.cell_checked((Y(y + dy), X(x + dx))) {
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
                let (Y(dy), X(dx)) = DICTIONARY_NEIGHBOR_OFFSET[i];
                if v == 1 {
                    self.decide((Y(y + dy), X(x + dx)), Cell::Black);
                } else if v == 2 {
                    self.decide((Y(y + dy), X(x + dx)), Cell::White);
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
                    if self.cell((Y(y), X(x))) == Cell::Undecided {
                        let checking_region = ((Y(y - 2), X(x - 2)), (Y(y + 3), X(x + 3)));
                        let mut trial_black = self.clone();
                        trial_black.checking_region = Some(checking_region);
                        trial_black.decide((Y(y), X(x)), Cell::Black);
                        trial_black.solve();

                        if trial_black.inconsistent() {
                            self.decide((Y(y), X(x)), Cell::White);
                            self.solve();
                            updated = true;
                        }

                        let mut trial_white = self.clone();
                        trial_white.checking_region = Some(checking_region);
                        trial_white.decide((Y(y), X(x)), Cell::White);
                        trial_white.solve();

                        if trial_white.inconsistent() {
                            self.decide((Y(y), X(x)), Cell::Black);
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
                match self.cell((Y(y), X(x))) {
                    Cell::Undecided => write!(f, ".... ")?,
                    Cell::Black => write!(f, "#### ")?,
                    Cell::White => {
                        let clue = self.clue((Y(y), X(x)));
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
    use super::*;
    use super::super::clue_pattern_to_id;

    #[test]
    fn test_tapa_field_clues() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        let mut field = Field::new(5, 6, &dic, &consecutive_dic);
        field.add_clue((Y(2), X(1)), clue_pattern_to_id(&[]).unwrap());
        field.add_clue((Y(2), X(3)), clue_pattern_to_id(&[4]).unwrap());

        assert_eq!(field.cell((Y(2), X(0))), Cell::White);
        assert_eq!(field.cell((Y(1), X(4))), Cell::Black);
        assert_eq!(field.cell((Y(2), X(4))), Cell::Black);
        assert_eq!(field.cell((Y(3), X(4))), Cell::Black);
        assert_eq!(field.inconsistent(), false);
    }

    #[test]
    fn test_tapa_field_cluster() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        let mut field = Field::new(5, 6, &dic, &consecutive_dic);
        field.decide(((Y(1), X(1))), Cell::Black);
        field.decide(((Y(1), X(2))), Cell::Black);
        field.decide(((Y(2), X(2))), Cell::Black);

        assert_eq!(field.cell((Y(2), X(1))), Cell::White);
        assert_eq!(field.inconsistent(), false);
    }

    #[test]
    fn test_tapa_field_connectivity() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        let mut field = Field::new(5, 6, &dic, &consecutive_dic);
        field.decide((Y(0), X(0)), Cell::Black);
        field.decide((Y(4), X(5)), Cell::Black);
        field.decide((Y(1), X(0)), Cell::White);
        field.decide((Y(2), X(1)), Cell::White);
        field.decide((Y(0), X(3)), Cell::White);
        field.decide((Y(0), X(2)), Cell::Undecided);
        field.decide((Y(1), X(1)), Cell::Undecided);

        field.inspect_connectivity();

        assert_eq!(field.cell((Y(0), X(1))), Cell::Black);
        assert_eq!(field.cell((Y(1), X(2))), Cell::Black);
        assert_eq!(field.inconsistent(), false);
    }

    #[test]
    fn test_tapa_field_virtually_ignored_cells() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        let mut field = Field::new(5, 7, &dic, &consecutive_dic);
        field.add_clue((Y(1), X(3)), clue_pattern_to_id(&[1, 2]).unwrap());
        field.add_clue((Y(3), X(3)), clue_pattern_to_id(&[1, 3]).unwrap());
        field.decide((Y(0), X(0)), Cell::Black);
        field.decide((Y(0), X(6)), Cell::Black);

        field.inspect_connectivity_advanced();

        assert_eq!(field.cell((Y(4), X(2))), Cell::Black);
        assert_eq!(field.cell((Y(4), X(3))), Cell::Black);
        assert_eq!(field.cell((Y(4), X(4))), Cell::Black);
    }

    #[test]
    fn test_tapa_field_problem() {
        let dic = Dictionary::new();
        let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

        {
            let mut field = Field::new(6, 5, &dic, &consecutive_dic);
            field.add_clue((Y(1), X(0)), clue_pattern_to_id(&[1, 3]).unwrap());
            field.add_clue((Y(1), X(2)), clue_pattern_to_id(&[2, 4]).unwrap());
            field.add_clue((Y(3), X(1)), clue_pattern_to_id(&[3, 3]).unwrap());
            field.add_clue((Y(4), X(3)), clue_pattern_to_id(&[4]).unwrap());

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
                        field.cell((Y(y), X(x))),
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
            field.add_clue((Y(0), X(0)), clue_pattern_to_id(&[1]).unwrap());
            field.add_clue((Y(1), X(2)), clue_pattern_to_id(&[3]).unwrap());
            field.add_clue((Y(0), X(5)), clue_pattern_to_id(&[2]).unwrap());

            field.inspect_connectivity_clue_aware();
            field.inspect_connectivity_clue_aware();

            let expected = [[0, 1, 1, 1, 1, 0, 0], [0, 0, 0, 0, 1, 0, 0]];
            for y in 0..2 {
                for x in 0..7 {
                    assert_eq!(
                        field.cell((Y(y), X(x))),
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
            field.add_clue((Y(1), X(3)), clue_pattern_to_id(&[]).unwrap());
            field.add_clue((Y(4), X(0)), clue_pattern_to_id(&[1]).unwrap());
            field.add_clue((Y(0), X(0)), clue_pattern_to_id(&[1]).unwrap());
            field.add_clue((Y(4), X(5)), clue_pattern_to_id(&[1]).unwrap());

            field.inspect_connectivity_clue_aware();

            assert_eq!(field.cell((Y(3), X(1))), Cell::Black);
            assert_eq!(field.inconsistent(), false);
        }
        {
            let mut field = Field::new(2, 2, &dic, &consecutive_dic);
            field.add_clue((Y(0), X(0)), clue_pattern_to_id(&[1]).unwrap());

            field.inspect_connectivity_clue_aware();

            assert_eq!(field.cell((Y(0), X(1))), Cell::Undecided);
            assert_eq!(field.cell((Y(1), X(0))), Cell::Undecided);
            assert_eq!(field.cell((Y(1), X(1))), Cell::Undecided);
            assert_eq!(field.inconsistent(), false);
        }
    }
}
