use super::super::{Grid, Y, X, Coord};
use grid_loop::{Edge, GridLoop, GridLoopField};
use super::*;

use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct Symmetry {
    pub dyad: bool,         // 180-degree symmetry
    pub tetrad: bool,       // 90-degree symmetry
    pub horizontal: bool,   // horizontal line symmetry
    pub vertical: bool,     // vertical line symmetry
}

pub fn generate<R: Rng>(has_clue: &Grid<bool>, dic: &Dictionary, rng: &mut R) -> Option<Grid<Clue>> {
    let height = has_clue.height();
    let width = has_clue.width();
    let max_step = height * width * 10;

    let mut current_problem = Grid::new(height, width, NO_CLUE);
    let mut prev_score = 0;
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

            let mut common;
            if prev_clue == NO_CLUE {
                common = last_field.clone();
            } else {
                current_problem[pos] = NO_CLUE;
                common = Field::new(&current_problem, dic);
                common.check_all_cell();
            }

            for &c in &new_clue_cand {
                current_problem[pos] = c;

                let mut field = common.clone();
                field.add_clue(pos, c);

                if field.inconsistent() {
                    continue;
                }

                let current_score = field.grid_loop().num_decided_edges() - count_prohibited_patterns(has_clue, &field, &current_problem) * 10;

                if prev_score >= current_score {
                    if !(rng.next_f64() < ((current_score - prev_score) as f64 / temperature).exp()) {
                        continue;
                    }
                }

                let mut field_inout_test = field.clone();
                GridLoop::apply_inout_rule(&mut field_inout_test);
                GridLoop::check_connectability(&mut field_inout_test);
                if field_inout_test.inconsistent() {
                    continue;
                }

                updated = true;
                prev_score = current_score;
                if prev_clue == NO_CLUE {
                    unplaced_clues -= 1;
                }

                if field.fully_solved() && unplaced_clues == 0 {
                    return Some(current_problem);
                }

                last_field = field;
                break;
            }

            if updated {
                break;
            } else {
                current_problem[pos] = prev_clue;
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
fn count_prohibited_patterns(has_clue: &Grid<bool>, field: &Field, problem: &Grid<Clue>) -> i32 {
    let neighbor = [
        (Y(1), X(0)),
        (Y(0), X(1)),
        (Y(-1), X(0)),
        (Y(0), X(-1)),
    ];
    let mut ret = 0;
    for y in 0..has_clue.height() {
        for x in 0..has_clue.width() {
            if has_clue[(Y(y), X(x))] && field.get_clue((Y(y), X(x))) == NO_CLUE && has_zero_nearby(problem, (Y(y), X(x))) {
                if
                    field.get_edge((Y(2 * y + 0), X(2 * x + 1))) == Edge::Blank &&
                    field.get_edge((Y(2 * y + 1), X(2 * x + 0))) == Edge::Blank &&
                    field.get_edge((Y(2 * y + 2), X(2 * x + 1))) == Edge::Blank &&
                    field.get_edge((Y(2 * y + 1), X(2 * x + 2))) == Edge::Blank {
                    ret += 1;
                    continue;
                }
            }
            if y > 0 && field.get_clue((Y(y - 1), X(x))) != NO_CLUE { continue; }
            if x > 0 && field.get_clue((Y(y), X(x - 1))) != NO_CLUE { continue; }
            if y < has_clue.height() - 1 && field.get_clue((Y(y + 1), X(x))) != NO_CLUE { continue; }
            if x < has_clue.width() - 1 && field.get_clue((Y(y), X(x + 1))) != NO_CLUE { continue; }

            if field.get_clue((Y(y), X(x))) == Clue(2) {
                if
                    field.get_edge_safe((Y(2 * y + 1 + 2), X(2 * x + 1 + 1))) == Edge::Blank &&
                    field.get_edge_safe((Y(2 * y + 1 + 1), X(2 * x + 1 + 2))) == Edge::Blank &&
                    field.get_edge_safe((Y(2 * y + 1 - 2), X(2 * x + 1 - 1))) == Edge::Blank &&
                    field.get_edge_safe((Y(2 * y + 1 - 1), X(2 * x + 1 - 2))) == Edge::Blank {
                    ret += 1;
                    continue;
                }
                if
                    field.get_edge_safe((Y(2 * y + 1 - 2), X(2 * x + 1 + 1))) == Edge::Blank &&
                    field.get_edge_safe((Y(2 * y + 1 - 1), X(2 * x + 1 + 2))) == Edge::Blank &&
                    field.get_edge_safe((Y(2 * y + 1 + 2), X(2 * x + 1 - 1))) == Edge::Blank &&
                    field.get_edge_safe((Y(2 * y + 1 + 1), X(2 * x + 1 - 2))) == Edge::Blank {
                    ret += 1;
                    continue;
                }
            } else if field.get_clue((Y(y), X(x))) == NO_CLUE {
                let mut n_in = 0;
                let mut n_blank = 0;

                for d in 0..4 {
                    let (Y(dy1), X(dx1)) = neighbor[d];
                    let (Y(dy2), X(dx2)) = neighbor[(d + 1) % 4];
                    let edge1 = field.get_edge_safe((Y(2 * y + 1 + dy1 * 2 + dy2), X(2 * x + 1 + dx1 * 2 + dx2)));
                    let edge2 = field.get_edge_safe((Y(2 * y + 1 + dy2 * 2 + dy1), X(2 * x + 1 + dx2 * 2 + dx1)));

                    match (edge1, edge2) {
                        (Edge::Blank, Edge::Blank) => n_blank += 1,
                        (Edge::Blank, Edge::Line) | (Edge::Line, Edge::Blank) => n_in += 1,
                        _ => (),
                    }
                }

                if n_in >= 1 && n_blank >= 2 {
                    ret += 1;
                }
            }
        }
    }
    ret
}

pub fn generate_placement<R: Rng>(height: i32, width: i32, num_clues: i32, symmetry: Symmetry, rng: &mut R) -> Grid<bool> {
    let mut num_clues = num_clues;
    let mut symmetry = symmetry;

    symmetry.dyad |= symmetry.tetrad;
    symmetry.tetrad &= (height == width);
    
    let mut grp_ids = Grid::new(height, width, false);
    let mut last_id = 0;

    let mut clue_positions: Vec<Vec<Coord>> = vec![];

    for y in 0..height {
        for x in 0..width {
            if !grp_ids[(Y(y), X(x))] {
                let mut sto = vec![];
                update_grp(y, x, last_id, symmetry, &mut grp_ids, &mut sto);
                clue_positions.push(sto);
            }
        }
    }

    let mut ret = Grid::new(height, width, false);
    while clue_positions.len() > 0 && num_clues > 0 {
        let mut scores = vec![];
        let mut scores_total = 0.0f64;

        for pos in &clue_positions {
            let (Y(y), X(x)) = pos[0];
            let mut score_base = 0.0f64;

            for dy in -2..3 {
                for dx in -2..3 {
                    let cd2 = (Y(y + dy), X(x + dx));
                    if ret.is_valid_coord(cd2) && ret[cd2] {
                        let dist = dy.abs() + dx.abs();
                        score_base += 5.0f64 - (dist as f64);
                        if dist == 1 {
                            score_base += 2.0f64;
                        }
                    }
                }
            }

            let pos_score = 64.0f64 * 2.0f64.powf((16.0f64 - score_base) / 2.0f64) + 4.0f64;
            scores.push(pos_score);
            scores_total += pos_score;
        }

        let mut thresh = rng.gen_range(0.0f64, scores_total);
        for i in 0..clue_positions.len() {
            if thresh < scores[i] {
                for &c in &(clue_positions[i]) {
                    ret[c] = true;
                    num_clues -= 1;
                }
                clue_positions.swap_remove(i);
                break;
            } else {
                thresh -= scores[i];
            }
        }
    }

    ret
}

fn update_grp(y: i32, x: i32, id: i32, symmetry: Symmetry, grp_ids: &mut Grid<bool>, sto: &mut Vec<Coord>) {
    if grp_ids[(Y(y), X(x))] {
        return;
    }
    grp_ids[(Y(y), X(x))] = true;
    sto.push((Y(y), X(x)));

    if symmetry.tetrad {
        update_grp(grp_ids.height() - 1 - x, y, id, symmetry, grp_ids, sto);
    } else if symmetry.dyad {
        update_grp(grp_ids.height() - 1 - y, grp_ids.width() - 1 - x, id, symmetry, grp_ids, sto);
    }
    if symmetry.horizontal {
        update_grp(grp_ids.height() - 1 - y, x, id, symmetry, grp_ids, sto);
    }
    if symmetry.vertical {
        update_grp(y, grp_ids.width() - 1 - x, id, symmetry, grp_ids, sto);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;

    fn run_placement_test<R: Rng>(placement: Vec<Vec<bool>>, dic: &Dictionary, rng: &mut R) {
        let placement = ::common::vec_to_grid(&placement);
        let mut succeeded = false;

        for _ in 0..10 {
            let problem = generate(&placement, dic, rng);

            if let Some(problem) = problem {
                succeeded = true;

                assert_eq!(problem.height(), placement.height());
                assert_eq!(problem.width(), placement.width());
                
                for y in 0..placement.height() {
                    for x in 0..placement.width() {
                        let clue = problem[(Y(y), X(x))];
                        assert_eq!(placement[(Y(y), X(x))], clue != NO_CLUE);

                        if clue == Clue(0) {
                            for dy in -1..2 {
                                for dx in -1..2 {
                                    let y2 = y + dy;
                                    let x2 = x + dx;

                                    if 0 <= y2 && y2 < placement.height() && 0 <= x2 && x2 < placement.width() && (dy, dx) != (0, 0) {
                                        assert!(problem[(Y(y2), X(x2))] != Clue(0));
                                    }
                                }
                            }
                        }
                    }
                }

                let mut field = Field::new(&problem, &dic);
                field.check_all_cell();
                assert!(!field.inconsistent());
                assert!(field.fully_solved());

                break;
            }
        }

        assert!(succeeded);
    }

    #[test]
    fn test_generator() {
        let mut rng = rand::thread_rng();
        let dic = Dictionary::complete();

        run_placement_test(vec![
            vec![true , true , true , true , true ],
            vec![true , false, false, false, true ],
            vec![true , false, false, false, true ],
            vec![true , false, false, false, true ],
            vec![true , true , true , true , true ],
        ], &dic, &mut rng);

        run_placement_test(vec![
            vec![true , false, true , true , true ],
            vec![false, false, false, false, true ],
            vec![true , false, false, false, true ],
            vec![true , false, false, false, false],
            vec![true , true , true , false, true ],
        ], &dic, &mut rng);
    }
}
