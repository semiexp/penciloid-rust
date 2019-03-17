use super::super::{Grid, X, Y};
use super::*;

use rand::distributions::Distribution;
use rand::{distributions, Rng};
use std::cmp;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClueConstraint {
    Any,
    Forced,
    Prohibited,
}

pub fn generate<R: Rng>(
    clue_constraint: &Grid<ClueConstraint>,
    max_clue: Option<i32>,
    rng: &mut R,
) -> Option<Grid<Clue>> {
    let height = clue_constraint.height();
    let width = clue_constraint.width();
    let mut has_clue = Grid::new(height, width, false);

    let dic = Dictionary::new();
    let consecutive_dic = ConsecutiveRegionDictionary::new(&dic);

    let mut problem = Grid::new(height, width, NO_CLUE);
    let mut field = Field::new(height, width, &dic, &consecutive_dic);
    let mut current_energy = 0i32;
    let mut n_clues = 0;

    for y in 0..height {
        for x in 0..width {
            if clue_constraint[(Y(y), X(x))] == ClueConstraint::Forced {
                has_clue[(Y(y), X(x))] = true;
                n_clues += 1;
            }
        }
    }

    let n_step = height * width * 10;
    let mut temperature = 20f64;

    for s in 0..n_step {
        let mut update_cand = vec![];
        for y in 0..height {
            for x in 0..width {
                if field.cell((Y(y), X(x))) == Cell::Black
                    || clue_constraint[(Y(y), X(x))] == ClueConstraint::Prohibited
                {
                    continue;
                }
                let y2 = height - 1 - y;
                let x2 = width - 1 - x;
                if -1 <= y - y2 && y - y2 <= 1 && -1 <= x - x2 && x - x2 <= 1
                    && (y != y2 || x != x2)
                {
                    continue;
                }
                let mut isok = true;
                for dy in -1..2 {
                    for dx in -1..2 {
                        if (dy != 0 || dx != 0) && has_clue.is_valid_coord((Y(y + dy), X(x + dx)))
                            && has_clue[(Y(y + dy), X(x + dx))]
                        {
                            isok = false;
                        }
                    }
                }
                let mut isok2 = false;
                for dy in -2..3 {
                    for dx in -2..3 {
                        let loc = (Y(y + dy), X(x + dx));
                        if field.cell_checked(loc) == Cell::Undecided {
                            isok2 = true;
                        }
                    }
                }
                if (has_clue[(Y(y), X(x))] && problem[(Y(y), X(x))] == NO_CLUE)
                    || (isok && isok2
                        && (problem[(Y(y), X(x))] != NO_CLUE || rng.gen::<f64>() < 1.0))
                {
                    for v in (-1)..(CLUE_TYPES as i32) {
                        if v == -1 && clue_constraint[(Y(y), X(x))] == ClueConstraint::Forced {
                            continue;
                        }
                        if v == 0 || v == 21 || v == 22 {
                            continue;
                        }
                        let next_clue = Clue(v);
                        if problem[(Y(y), X(x))] != next_clue {
                            update_cand.push(((Y(y), X(x)), next_clue));
                        }
                    }
                }
            }
        }

        rng.shuffle(&mut update_cand);

        let mut applicable_cands = vec![];

        for &(loc, clue) in &update_cand {
            let previous_clue = problem[loc];
            let mut n_clues2 = n_clues;
            let (Y(y), X(x)) = loc;
            let loc2 = (Y(height - 1 - y), X(width - 1 - x));
            if clue == NO_CLUE {
                if loc == loc2 {
                    n_clues2 -= 1;
                } else if problem[loc2] == NO_CLUE {
                    n_clues2 -= 2;
                }
            } else {
                if !has_clue[loc] {
                    n_clues2 += 1;
                }
                if loc != loc2 && !has_clue[loc2] {
                    n_clues2 += 1;
                }
            }
            if let Some(max_clue) = max_clue {
                if max_clue < n_clues2 {
                    continue;
                }
            }

            problem[loc] = clue;
            let field = if previous_clue == NO_CLUE {
                let mut f = field.clone();
                f.add_clue(loc, clue);
                f.solve();
                f.trial_and_error();
                f
            } else {
                solve_test(&problem, &has_clue, &dic, &consecutive_dic)
            };
            let energy = field.decided_cells() - 4 * n_clues2;

            let update = !field.inconsistent()
                && (current_energy < energy
                    || rng.gen::<f64>() < ((energy - current_energy) as f64 / temperature).exp());

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
                    return Some(problem);
                }

                applicable_cands.push((loc, clue, energy, n_clues2, field));
            }
            problem[loc] = previous_clue;

            if applicable_cands.len() >= 1 {
                break;
            }
        }

        if applicable_cands.len() >= 1 {
            let mut best_cand = applicable_cands.swap_remove(0);
            while applicable_cands.len() > 0 {
                let cand2 = applicable_cands.swap_remove(0);
                if best_cand.3 < cand2.3 {
                    best_cand = cand2;
                }
            }

            let (loc, clue, energy, n_clues2, field2) = best_cand;

            current_energy = energy;
            n_clues = n_clues2;
            problem[loc] = clue;
            field = field2;

            let (Y(y), X(x)) = loc;
            let loc2 = (Y(height - 1 - y), X(width - 1 - x));
            if clue == NO_CLUE {
                if problem[loc2] == NO_CLUE {
                    has_clue[loc] = false;
                    has_clue[loc2] = false;
                }
            } else {
                has_clue[loc] = true;
                has_clue[loc2] = true;
            }
        } else {
            break;
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
