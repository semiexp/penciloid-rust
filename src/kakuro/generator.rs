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
        let mut move_cand: Vec<Vec<(i32, i32, i32, i32)>> = vec![];

        let mut grp_has_val = vec![0; field_shape.group_to_cells.len()];
        for y in 0..height {
            for x in 0..width {
                let loc = Coord { y: y, x: x };
                if has_clue[loc] {
                    continue;
                }
                let (g1, g2) = field_shape.cell_to_groups[loc];
                grp_has_val[g1 as usize] |= 1 << answer[loc];
                grp_has_val[g2 as usize] |= 1 << answer[loc];
            }
        }
        for y in 0..height {
            for x in 0..width {
                let loc = Coord { y: y, x: x };
                if has_clue[loc] {
                    continue;
                }
                let (g1, g2) = field_shape.cell_to_groups[loc];
                for c in 1..MAX_VAL {
                    if answer[loc] == c {
                        continue;
                    }
                    if (grp_has_val[g1 as usize] & (1 << c)) != 0 {
                        continue;
                    }
                    if (grp_has_val[g2 as usize] & (1 << c)) != 0 {
                        continue;
                    }
                    move_cand.push(vec![(y, x, answer[loc], c)]);
                }
            }
        }

        rng.shuffle(&mut move_cand);

        for cand in &move_cand {
            for &(y, x, _, c) in cand {
                answer[Coord { y: y, x: x }] = c;
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
                for &(y, x, old, _) in cand {
                    answer[Coord { y: y, x: x }] = old;
                }
            }
        }

        temperature *= 0.995;
    }

    None
}
