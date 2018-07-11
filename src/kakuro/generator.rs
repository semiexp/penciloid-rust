use super::super::Grid;
use super::*;

use rand::{Rng, distributions};
use rand::distributions::IndependentSample;
use std;

pub fn generate<R: Rng>(has_clue: &Grid<bool>, dic: &Dictionary, rng: &mut R) -> Option<Grid<Clue>> {
    let height = has_clue.height();
    let width = has_clue.width();

    let mut answer = Grid::new(height, width, -1);
    let mut current_total_cands = height * width * 9;
    for y in 0..height {
        for x in 0..width {
            if !has_clue[(Y(y), X(x))] {
                answer[(Y(y), X(x))] = (y + x) % MAX_VAL + 1;
            }
        }
    }

    let field_shape = FieldShape::new(has_clue);
    let n_step = height * width * 10;
    let mut temperature = 10.0f64;

    for _ in 0..n_step {
        let mut move_cand: Vec<Vec<(usize, i32, i32)>> = vec![];

        let mut grp_val_loc = vec![[None; (MAX_VAL + 1) as usize]; field_shape.group_to_cells.len()];
        for y in 0..height {
            for x in 0..width {
                let loc = (Y(y), X(x));
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
                let loc = (Y(y), X(x));
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
            if current_total_cands > total_cands || rng.gen::<f64>() < ((current_total_cands - total_cands) as f64 / temperature).exp() {
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

fn check_connectivity(grid: &Grid<bool>) -> i32 { // returns the sum of sizes of non-largest components
    fn dfs(y: i32, x: i32, grid: &Grid<bool>, vis: &mut Grid<bool>) -> i32 {
        if vis[(Y(y), X(x))] || grid[(Y(y), X(x))] { return 0; }
        vis[(Y(y), X(x))] = true;
        let mut ret = 1;
        if y > 0 { ret += dfs(y - 1, x, grid, vis); }
        if x > 0 { ret += dfs(y, x - 1, grid, vis); }
        if y + 1 < grid.height() { ret += dfs(y + 1, x, grid, vis); }
        if x + 1 < grid.width() { ret += dfs(y, x + 1, grid, vis); }
        ret
    }
    let mut vis = Grid::new(grid.height(), grid.width(), false);
    let mut sum = 0;
    let mut largest = 0;
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            if !grid[(Y(y), X(x))] && !vis[(Y(y), X(x))] {
                let sz = dfs(y, x, &grid, &mut vis);
                sum += sz;
                largest = std::cmp::max(largest, sz);
            }
        }
    }
    return sum - largest;
}
pub fn disconnectivity_score(grid: &Grid<bool>) -> i32 {
    let mut grid = grid.clone();
    let mut ret = 0;
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            if !grid[(Y(y), X(x))] {
                grid[(Y(y), X(x))] = true;
                ret += check_connectivity(&grid);
                grid[(Y(y), X(x))] = false;
            }
        }
    }
    ret
}
pub fn generate_placement<'a, T: Rng>(height: i32, width: i32, rng: &'a mut T) -> Option<Grid<bool>> {
    let height = height + 1;
    let width = width + 1;

    let mut placement = Grid::new(height, width, false);
    for y in 0..height {
        placement[(Y(y), X(0))] = true;
        placement[(Y(y), X(width - 1))] = true;
    }
    for x in 0..width {
        placement[(Y(0), X(x))] = true;
        placement[(Y(height - 1), X(x))] = true;
    }
    loop {
        let mut cand = vec![];
        for y in 0..height {
            for x in 0..width {
                let loc = (Y(y), X(x));
                if placement[loc] { continue; }

                if x >= 2 && !placement[(Y(y), X(x - 1))] && placement[(Y(y), X(x - 2))] { continue; }
                if x < width - 2 && !placement[(Y(y), X(x + 1))] && placement[(Y(y), X(x + 2))] { continue; }
                if y >= 2 && !placement[(Y(y - 1), X(x))] && placement[(Y(y - 2), X(x))] { continue; }
                if y < height - 2 && !placement[(Y(y + 1), X(x))] && placement[(Y(y + 2), X(x))] { continue; }
                if height % 2 == 1 && width % 2 == 1 && !placement[(Y(height / 2), X(width / 2))] {
                    if y == height / 2 && (x == width / 2 - 1 || x == width / 2 + 1) { continue; }
                    if x == width / 2 && (y == height / 2 - 1 || y == height / 2 + 1) { continue; }
                }

                let mut wl = 0;
                let mut wr = 0;
                let mut wu = 0;
                let mut wd = 0;
                for d in 0..height {
                    if placement[(Y(y), X(x - d))] {
                        wl = d;
                        break;
                    }
                }
                for d in 0..height {
                    if placement[(Y(y), X(x + d))] {
                        wr = d;
                        break;
                    }
                }
                for d in 0..height {
                    if placement[(Y(y - d), X(x))] {
                        wu = d;
                        break;
                    }
                }
                for d in 0..height {
                    if placement[(Y(y + d), X(x))] {
                        wd = d;
                        break;
                    }
                }
                let mut weight = std::cmp::max(wl + wr, wu + wd);
                let adj = 
                    if wl == 1 { 1 } else { 0 } + 
                    if wr == 1 { 1 } else { 0 } + 
                    if wu == 1 { 1 } else { 0 } + 
                    if wd == 1 { 1 } else { 0 };
                if y == 1 || x == 1 || y == height - 2 || x == width - 2 {
                    weight *= 4;
                } else if adj <= 1 {
                    weight *= 2;
                }
                cand.push(distributions::Weighted { weight: weight as u32, item: (y, x) });
            }
        }

        if cand.len() == 0 {
            return None;
        }
        let wc = distributions::WeightedChoice::new(&mut cand);
        let mut upd = false;

        for _ in 0..10 {
            let (y, x) = wc.ind_sample(rng);
            placement[(Y(y), X(x))] = true;
            placement[(Y(height - 1 - y), X(width - 1 - x))] = true;

            if check_connectivity(&placement) == 0 {
                upd = true;
                break;
            }
        }

        if !upd {
            return None;
        }

        let mut isok = true;
        for y in 0..height {
            let mut con = 0;
            for x in 0..width {
                if placement[(Y(y), X(x))] {
                    con = 0;
                } else {
                    con += 1;
                    if con >= 10 {
                        isok = false;
                    }
                }
            }
        }
        for x in 0..width {
            let mut con = 0;
            for y in 0..height {
                if placement[(Y(y), X(x))] {
                    con = 0;
                } else {
                    con += 1;
                    if con >= 10 {
                        isok = false;
                    }
                }
            }
        }
        if isok {
            break;
        }
    }

    let mut ret = Grid::new(height - 1, width - 1, false);
    for y in 0..(height - 1) {
        for x in 0..(width - 1) {
            let loc = (Y(y), X(x));
            ret[loc] = placement[loc];
        }
    }
    Some(ret)
}
