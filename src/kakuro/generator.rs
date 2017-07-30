use super::super::{Grid, Coord};
use super::*;

use rand::Rng;

pub fn generate<R: Rng>(has_clue: &Grid<bool>, dic: &Dictionary, rng: &mut R) -> Option<Grid<Clue>> {
    let height = has_clue.height();
    let width = has_clue.width();

    let mut answer = Grid::new(height, width, -1);
    let mut current_total_cands = (height * width * 9);
    for y in 0..height {
        for x in 0..width {
            if !has_clue[Coord { y: y, x: x }] {
                answer[Coord { y: y, x: x }] = (y + x) % MAX_VAL + 1;
            }
        }
    }

    let field_shape = FieldShape::new(has_clue);
    let n_step = height * width * 10;
    let mut temperature = 10.0f64;

    for step in 0..n_step {
        let mut move_cand: Vec<Vec<(usize, i32, i32)>> = vec![];

        let mut grp_val_loc = vec![[None; (MAX_VAL + 1) as usize]; field_shape.group_to_cells.len()];
        for y in 0..height {
            for x in 0..width {
                let loc = Coord { y: y, x: x };
                if has_clue[loc] {
                    continue;
                }
                let (g1, g2) = field_shape.cell_to_groups[loc];
                grp_val_loc[g1][answer[loc] as usize] = Some(has_clue.index(loc));
                grp_val_loc[g2][answer[loc] as usize] = Some(has_clue.index(loc));
            }
        }
        for y in 0..height {
            for x in 0..width {
                let loc = Coord { y: y, x: x };
                let c = has_clue.index(loc);
                if has_clue[loc] {
                    continue;
                }
                let (g1, g2) = field_shape.cell_to_groups[loc];
                for v in 1..(MAX_VAL + 1) {
                    if answer[loc] == v {
                        continue;
                    }
                    match (grp_val_loc[g1][v as usize], grp_val_loc[g2][v as usize]) {
                        (None, None) => move_cand.push(vec![(c, answer[loc], v)]),
                        (Some(c1), None) => {
                            if answer[loc] < v && grp_val_loc[field_shape.cell_to_groups[c1].1][answer[loc] as usize] == None {
                                move_cand.push(vec![
                                    (c, answer[loc], v),
                                    (c1, v, answer[loc]),
                                ]);
                            }
                        },
                        (None, Some(c2)) => {
                            if answer[loc] < v && grp_val_loc[field_shape.cell_to_groups[c2].0][answer[loc] as usize] == None {
                                move_cand.push(vec![
                                    (c, answer[loc], v),
                                    (c2, v, answer[loc]),
                                ]);
                            }
                        },
                        (Some(c1), Some(c2)) => {
                            if grp_val_loc[field_shape.cell_to_groups[c1].1][answer[loc] as usize] == None &&
                               grp_val_loc[field_shape.cell_to_groups[c2].0][answer[loc] as usize] == None {
                                move_cand.push(vec![
                                    (c, answer[loc], v),
                                    (c1, v, answer[loc]),
                                    (c2, v, answer[loc]),
                                ]);
                            }
                        },
                    }
                }
            }
        }

        rng.shuffle(&mut move_cand);

        for cand in &move_cand {
            for &(id, _, c) in cand {
                answer[id] = c;
            }
            let problem = answer_to_problem(&answer);
            let mut field = Field::new(&problem, dic);
            field.check_all();

            if field.inconsistent() {
                unreachable!();
            }
            if field.solved() {
                return Some(problem);
            }

            let total_cands = field.total_cands() as i32;
            if current_total_cands > total_cands || rng.next_f64() < ((current_total_cands - total_cands) as f64 / temperature).exp() {
                current_total_cands = total_cands;
                break;
            } else {
                for &(id, old, _) in cand {
                    answer[id] = old;
                }
            }
        }

        temperature *= 0.995;
    }

    None
}
