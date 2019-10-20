use super::super::grid_loop::GridLoopField;
use super::super::{Grid, D, P};
use super::*;

use rand::Rng;
use FOUR_NEIGHBOURS;

pub struct GeneratorOption {
    pub disallow_dead_ends: bool,
    pub disallow_adjacent_clues: bool,
    pub technique: Technique,
    pub search_depth: i32,
    pub clue_lower_bound: Option<i32>,
    pub clue_upper_bound: Option<i32>,
}

fn check_clue_range(i: i32, options: &GeneratorOption) -> bool {
    if let Some(n) = options.clue_lower_bound {
        if !(n <= i) {
            return false;
        }
    }
    if let Some(n) = options.clue_upper_bound {
        if !(i <= n) {
            return false;
        }
    }
    true
}

pub fn generate<R: Rng>(
    height: i32,
    width: i32,
    options: &GeneratorOption,
    rng: &mut R,
) -> Option<Grid<Clue>> {
    let mut problem = Grid::new(height, width, Clue::NoClue);
    let mut current_score = 0i32;

    let n_steps = height * width * 10;
    let mut temperature = 10f64;
    let mut n_clues = 0;
    let mut has_clue = Grid::new(height, width, false);

    for _ in 0..n_steps {
        let mut update_cand = vec![];
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);

                if options.disallow_adjacent_clues {
                    let mut flg = false;
                    for dy in -1..2 {
                        for dx in -1..2 {
                            if (dy != 0 || dx != 0)
                                && problem.get_or_default_p(pos + D(dy, dx), Clue::NoClue)
                                    != Clue::NoClue
                            {
                                flg = true;
                            }
                        }
                    }
                    if flg {
                        continue;
                    }
                }
                for i in 0..((y + 3) / 2) {
                    if !check_clue_range(i, options) {
                        continue;
                    }
                    update_cand.push((pos, Clue::Up(i)));
                }
                for i in 0..((x + 3) / 2) {
                    if !check_clue_range(i, options) {
                        continue;
                    }
                    update_cand.push((pos, Clue::Left(i)));
                }
                for i in 0..((height - y) / 2) {
                    if !check_clue_range(i, options) {
                        continue;
                    }
                    update_cand.push((pos, Clue::Down(i)));
                }
                for i in 0..((width - x + 2) / 2) {
                    if !check_clue_range(i, options) {
                        continue;
                    }
                    update_cand.push((pos, Clue::Right(i)));
                }
                if problem[pos] != Clue::NoClue {
                    update_cand.push((pos, Clue::NoClue));
                }
            }
        }
        rng.shuffle(&mut update_cand);

        let mut updated = false;

        for &(loc, clue) in &update_cand {
            let previous_clue = problem[loc];
            has_clue[loc] = clue != Clue::NoClue;
            if options.disallow_dead_ends && has_dead_end_nearby(loc, &has_clue) {
                has_clue[loc] = previous_clue != Clue::NoClue;
                continue;
            }

            problem[loc] = clue;

            let mut next_field = Field::new(&problem);
            next_field.set_technique(options.technique);
            next_field.trial_and_error(options.search_depth);

            let new_n_clues = n_clues - if previous_clue == Clue::NoClue { 0 } else { 1 }
                + if clue == Clue::NoClue { 0 } else { 1 };

            let score = next_field.grid_loop().num_decided_edges() - new_n_clues * 25;
            let update = !next_field.inconsistent()
                && (current_score < score
                    || rng.gen::<f64>() < ((score - current_score) as f64 / temperature).exp());
            if update {
                current_score = score;
                updated = true;
                n_clues = new_n_clues;

                if next_field.fully_solved() {
                    return Some(problem);
                }
                break;
            } else {
                problem[loc] = previous_clue;
                has_clue[loc] = previous_clue != Clue::NoClue;
            }
        }

        if !updated {
            break;
        }

        temperature *= 0.995f64;
    }

    None
}

fn has_dead_end_nearby(loc: P, has_clue: &Grid<bool>) -> bool {
    return is_dead_end(loc, has_clue)
        || is_dead_end(loc + D(-1, 0), has_clue)
        || is_dead_end(loc + D(1, 0), has_clue)
        || is_dead_end(loc + D(0, -1), has_clue)
        || is_dead_end(loc + D(0, 1), has_clue);
}
fn is_dead_end(pos: P, has_clue: &Grid<bool>) -> bool {
    if !has_clue.is_valid_p(pos) {
        return false;
    }
    let mut neighbour_count = 0;
    for &d in &FOUR_NEIGHBOURS {
        if !has_clue.get_or_default_p(pos + d, true) {
            neighbour_count += 1;
        }
    }
    return neighbour_count <= 1;
}
