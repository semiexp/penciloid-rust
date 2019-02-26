use super::super::{Coord, Grid, X, Y};
use super::{Cell, Clue, Dictionary, DICTIONARY_INCONSISTENT, DICTIONARY_NEIGHBOR_OFFSET, NO_CLUE};

pub struct Field<'a> {
    cell: Grid<Cell>,
    clue: Grid<Clue>,
    inconsistent: bool,
    dic: &'a Dictionary,
}

impl<'a> Field<'a> {
    pub fn new(height: i32, width: i32, dic: &'a Dictionary) -> Field {
        Field {
            cell: Grid::new(height, width, Cell::Undecided),
            clue: Grid::new(height, width, NO_CLUE),
            inconsistent: false,
            dic,
        }
    }
    pub fn inconsistent(&self) -> bool {
        self.inconsistent
    }
    pub fn set_inconsistent(&mut self) {
        self.inconsistent = true;
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
        let current_status = self.cell_checked(loc);
        if current_status != Cell::Undecided {
            if current_status != v {
                self.inconsistent = true;
            }
            return;
        }

        self.cell[loc] = v;

        let (Y(y), X(x)) = loc;
        for dy in -1..2 {
            for dx in -1..2 {
                self.inspect((Y(y + dy), X(x + dx)));
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
            let neighbor = self.dic.consult_raw(clue, neighbor);

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::clue_pattern_to_id;

    #[test]
    fn test_tapa_field() {
        let dic = Dictionary::complete();

        let mut field = Field::new(5, 6, &dic);
        field.add_clue((Y(2), X(1)), clue_pattern_to_id(&[]).unwrap());
        field.add_clue((Y(2), X(3)), clue_pattern_to_id(&[4]).unwrap());

        assert_eq!(field.cell((Y(2), X(0))), Cell::White);
        assert_eq!(field.cell((Y(1), X(4))), Cell::Black);
        assert_eq!(field.cell((Y(2), X(4))), Cell::Black);
        assert_eq!(field.cell((Y(3), X(4))), Cell::Black);
        assert_eq!(field.inconsistent(), false);
    }
}
