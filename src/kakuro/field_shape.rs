use super::super::{Grid, Y, X};

#[derive(Clone, Copy)]
pub struct FieldShapeGrp {
    start: usize,
    end: usize,
    step: usize,
}
impl FieldShapeGrp {
    pub fn size(&self) -> usize {
        (self.end - self.start) / self.step
    }
}
impl Iterator for FieldShapeGrp {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let ret = self.start;
        self.start += self.step;

        if ret < self.end { Some(ret) } else { None }
    }
}
#[derive(Clone, Copy)]
pub enum ClueLocation {
    Horizontal(usize),
    Vertical(usize),
}
pub struct FieldShape {
    pub has_clue: Grid<bool>,
    pub cell_to_groups: Grid<(usize, usize)>,
    pub group_to_cells: Vec<FieldShapeGrp>,
    pub clue_locations: Vec<ClueLocation>,
}
impl FieldShape {
    pub fn new(has_clue: &Grid<bool>) -> FieldShape {
        let height = has_clue.height();
        let width = has_clue.width();
        let mut cell_to_groups = Grid::new(height, width, (0, 0));
        let mut group_to_cells = vec![];
        let mut clue_locations = vec![];
        let mut current_grp_id = 0;

        // compute horizontal groups
        for y in 0..height {
            let mut start = None;
            for x in 0..(width + 1) {
                if x == width || has_clue[(Y(y), X(x))] {
                    if let Some(s) = start {
                        group_to_cells.push(FieldShapeGrp {
                            start: s,
                            end: (y * width + x) as usize,
                            step: 1,
                        });
                        clue_locations.push(ClueLocation::Horizontal(s - 1));
                        current_grp_id += 1;
                    }
                    start = None;
                } else {
                    if start == None {
                        start = Some((y * width + x) as usize);
                    }
                    cell_to_groups[(Y(y), X(x))].0 = current_grp_id;
                }
            }
        }

        // compute vertical groups
        for x in 0..width {
            let mut start = None;
            for y in 0..(height + 1) {
                if y == height || has_clue[(Y(y), X(x))] {
                    if let Some(s) = start {
                        group_to_cells.push(FieldShapeGrp {
                            start: s,
                            end: (y * width + x) as usize,
                            step: width as usize,
                        });
                        clue_locations.push(ClueLocation::Vertical(s - width as usize));
                        current_grp_id += 1;
                    }
                    start = None;
                } else {
                    if start == None {
                        start = Some((y * width + x) as usize);
                    }
                    cell_to_groups[(Y(y), X(x))].1 = current_grp_id;
                }
            }
        }

        FieldShape {
            has_clue: has_clue.clone(),
            cell_to_groups: cell_to_groups,
            group_to_cells: group_to_cells,
            clue_locations: clue_locations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common;

    #[test]
    fn test_field_shape_grp() {
        {
            let mut n = 0;
            let mut sum = 0;
            let grp = FieldShapeGrp {
                start: 3,
                end: 6,
                step: 1,
            };
            for i in grp {
                n += 1;
                sum += i;
            }
            assert_eq!(n, 3);
            assert_eq!(sum, 3 + 4 + 5);
            assert_eq!(grp.start, 3);
        }
        {
            let mut n = 0;
            let mut sum = 0;
            let grp = FieldShapeGrp {
                start: 4,
                end: 16,
                step: 3,
            };
            for i in grp {
                n += 1;
                sum += i;
            }
            assert_eq!(n, 4);
            assert_eq!(sum, 4 + 7 + 10 + 13);
            assert_eq!(grp.start, 4);
        }
    }

    #[test]
    fn test_field_shape() {
        {
            let clues_vec = vec![
                vec![true , true , true , true ],
                vec![true , false, false, false],
                vec![true , false, false, false],
                vec![true , true , false, false],
                vec![true , false, false, false],
                vec![true , false, false, false],
            ];
            let clues = common::vec_to_grid(&clues_vec);
            let shape = FieldShape::new(&clues);

            assert_eq!(shape.group_to_cells.len(), 9);

            let (g1, g2) = shape.cell_to_groups[(Y(4), X(1))];
            let h1 = shape.clue_locations[g1 as usize];
            let h2 = shape.clue_locations[g2 as usize];
            assert!(match (h1, h2) {
                (ClueLocation::Horizontal(16), ClueLocation::Vertical(13)) |
                (ClueLocation::Vertical(13), ClueLocation::Horizontal(16)) => true,
                _ => false,
            });
        }
    }
}
