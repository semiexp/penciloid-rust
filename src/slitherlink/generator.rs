use super::super::{Grid, Y, X, Coord};
use grid_loop::{Edge, GridLoop, GridLoopField};
use super::*;

use rand::{Rng, distributions};

pub fn generate<R: Rng>(has_clue: &Grid<bool>, dic: &Dictionary, rng: &mut R) -> Option<Grid<Clue>> {
    let height = has_clue.height();
    let width = has_clue.width();
    let max_step = height * width * 10;

    let mut current_problem = Grid::new(height, width, NO_CLUE);
    let mut prev_decided_edges = 0;
    let temperature = 5.0f64;

    let mut unplaced_clues = 0;
    for y in 0..height {
        for x in 0..width {
            if has_clue[(Y(y), X(x))] {
                unplaced_clues += 1;
            }
        }
    }

    let mut last_field = Field::new(&current_problem, dic);

    for _ in 0..max_step {
        let mut pos_cand = vec![];
        for y in 0..height {
            for x in 0..width {
                let cd = (Y(y), X(x));
                if has_clue[cd] && (current_problem[cd] == NO_CLUE || has_undecided_nearby(&last_field, cd)) {
                    pos_cand.push(cd);
                }
            }
        }

        rng.shuffle(&mut pos_cand);

        let mut updated = false;
        for &pos in &pos_cand {
            let prev_clue = current_problem[pos];
            let is_zero_ok = !has_zero_nearby(&current_problem, pos);

            let mut new_clue_cand = vec![];
            for c in (if is_zero_ok { 0 } else { 1 })..4 {
                let c = Clue(c);
                if c != prev_clue {
                    new_clue_cand.push(c);
                }
            }

            rng.shuffle(&mut new_clue_cand);

            for &c in &new_clue_cand {
                current_problem[pos] = c;

                let mut field = Field::new(&current_problem, dic);
                field.check_all_cell();

                let isok;
                let current_decided_edges;
                if field.inconsistent() {
                    isok = false;
                    current_decided_edges = -1;
                } else {
                    current_decided_edges = field.grid_loop().num_decided_edges();

                    if prev_decided_edges < current_decided_edges {
                        isok = true;
                    } else {
                        isok = rng.next_f64() < ((current_decided_edges - prev_decided_edges) as f64 / temperature).exp()
                    }
                }

                if isok {
                    updated = true;
                    prev_decided_edges = current_decided_edges;
                    if prev_clue == NO_CLUE {
                        unplaced_clues -= 1;
                    }

                    if field.fully_solved() && unplaced_clues == 0 {
                        return Some(current_problem);
                    }

                    last_field = field;
                    break;
                } else {
                    current_problem[pos] = prev_clue;
                }
            }

            if updated {
                break;
            }
        }
    }

    None
}

fn has_undecided_nearby(field: &Field, clue_pos: Coord) -> bool {
    let (Y(y), X(x)) = clue_pos;
    let y = y * 2 + 1;
    let x = x * 2 + 1;

    let neighbor_size: i32 = 7;
    for dy in -neighbor_size..(neighbor_size + 1) {
        let dx_max = neighbor_size - dy.abs();
        for dx in -dx_max..(dx_max + 1) {
            let y2 = y + dy;
            let x2 = x + dx;
            
            let cd = (Y(y2), X(x2));
            if (dy & 1) != (dx & 1) {
                if field.get_edge_safe(cd) == Edge::Undecided {
                    return true;
                }
            }
        }
    }
    false
}

fn has_zero_nearby(problem: &Grid<Clue>, (Y(y), X(x)): Coord) -> bool {
    for dy in -1..2 {
        for dx in -1..2 {
            let cd = (Y(y + dy), X(x + dx));
            if problem.is_valid_coord(cd) && problem[cd] == Clue(0) {
                return true;
            }
        }
    }
    false
}
