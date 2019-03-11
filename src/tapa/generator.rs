use super::super::{Grid, X, Y};
use super::*;

use rand::distributions::Distribution;
use rand::{distributions, Rng};
use std::cmp;

pub fn generate<R: Rng>(has_clue: &Grid<bool>, rng: &mut R) -> Option<Grid<Clue>> {
    let height = has_clue.height();
    let width = has_clue.width();
    let dic = Dictionary::complete();
    let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

    let mut problem = Grid::new(height, width, NO_CLUE);
    let mut current_progress = 0i32;

    let n_step = height * width * 10;
    let mut temperature = 10f64;

    for _ in 0..n_step {
        let mut update_cand = vec![];
        for y in 0..height {
            for x in 0..width {
                if has_clue[(Y(y), X(x))] {
                    for v in 0..CLUE_TYPES {
                        if v == 0 || v == 6 || v == 8 || v == 11 || v == 19 || v == 20 || v == 21
                            || v == 22
                        {
                            continue;
                        }
                        let next_clue = Clue(v as i32);
                        if problem[(Y(y), X(x))] != next_clue {
                            update_cand.push(((Y(y), X(x)), next_clue));
                        }
                    }
                }
            }
        }

        rng.shuffle(&mut update_cand);

        for &(loc, clue) in &update_cand {
            let previous_clue = problem[loc];

            problem[loc] = clue;
            let field = solve_test(&problem, &has_clue, &dic, &consecutive_dic);

            let update = !field.inconsistent()
                && (current_progress < field.decided_cells()
                    || rng.gen::<f64>()
                        < ((current_progress - field.decided_cells()) as f64 / temperature).exp());

            if update {
                let mut clue_filled = true;
                for y in 0..height {
                    for x in 0..width {
                        if has_clue[(Y(y), X(x))] && problem[(Y(y), X(x))] == NO_CLUE {
                            clue_filled = false;
                        }
                    }
                }
                if field.fully_solved() && clue_filled {
                    println!("{}", field);
                    return Some(problem);
                }
                current_progress = field.decided_cells();
                break;
            } else {
                problem[loc] = previous_clue;
            }
        }

        temperature *= 0.995f64;
    }

    None
}

fn solve_test<'a, 'b>(
    problem: &Grid<Clue>,
    has_clue: &Grid<bool>,
    dic: &'a Dictionary,
    consecutive_dic: &'b ConsecutiveRegionDictionary,
) -> Field<'a, 'b> {
    let height = problem.height();
    let width = problem.width();
    let mut ret = Field::new(height, width, dic, consecutive_dic);

    for y in 0..height {
        for x in 0..width {
            let clue = problem[(Y(y), X(x))];
            if clue != NO_CLUE {
                ret.add_clue((Y(y), X(x)), clue);
            } else if has_clue[(Y(y), X(x))] {
                ret.decide((Y(y), X(x)), Cell::White);
            }

            if ret.inconsistent() {
                return ret;
            }
        }
    }

    ret.solve();
    ret.trial_and_error();

    ret
}
