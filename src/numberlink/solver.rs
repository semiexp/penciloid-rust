use super::super::{Y, X, Grid};
use super::*;

/// Type for status of frontier.
///
/// - -1: Closed end
/// - 0, 1, ...: Open endpoint (the another end is also in the frontier)
///   The another end may be the endpoint itself.
/// - -2, -3, ...: Open endpoint (the another end is connected to a number cell)
type Frontier = Vec<i32>;
const CLOSED_END: i32 = -1;

pub fn solve(problem: &Grid<Clue>) -> Vec<LinePlacement> {
    let height = problem.height();
    let width = problem.width();
    let mut internal_problem = Grid::new(height, width, 0);
    for y in 0..height {
        for x in 0..width {
            let Clue(c) = problem[(Y(y), X(x))];
            if c > 0 {
                internal_problem[(Y(y), X(x))] = -c - 1;
            }
        }
    }

    let mut initial_frontier = vec![CLOSED_END; width as usize];
    for x in 0..width {
        initial_frontier[x as usize] = if problem[(Y(0), X(x))] == Clue(0) { x } else { internal_problem[(Y(0), X(x))] };
    }
    let mut place = LinePlacement::new(height, width);
    let mut answers = vec![];
    search(0, 0, &internal_problem, &initial_frontier, &mut place, &mut answers);

    answers
}

fn search(y: i32, x: i32, problem: &Grid<i32>, frontier: &Frontier, place: &mut LinePlacement, answers: &mut Vec<LinePlacement>) {
    let (y, x) = if x == problem.width() { (y + 1, 0) } else { (y, x) };
    if y == problem.height() {
        answers.push(place.clone());
        return;
    }

    for &(h, v) in &[(false, false), (false, true), (true, false), (true, true)] {
        if v && y == problem.height() - 1 { continue; }
        if h && x == problem.width() - 1 { continue; }

        let deg =
            if v { 1 } else { 0 } +
            if h { 1 } else { 0 } +
            if place.down((Y(y - 1), X(x))) { 1 } else { 0 } +
            if place.right((Y(y), X(x - 1))) { 1 } else { 0 } +
            if problem[(Y(y), X(x))] < 0 { 1 } else { 0 };
        if deg != 0 && deg != 2 { continue; }

        let mut new_frontier = frontier.clone();

        // Forbidden patterns related to isolated cells:
        // 
        // + X ; X +
        // |   ;   |
        // +-+ ; +-+
        if h && y > 0 && x < problem.width() - 1 {
            if place.isolated((Y(y - 1), X(x + 1))) && place.down((Y(y - 1), X(x))) { continue; }
            if place.isolated((Y(y - 1), X(x))) && place.down((Y(y - 1), X(x + 1))) { continue; }
        }

        // Forbidden patterns related to redundant paths:
        //
        // +-+ ; +-+ ; + + ; +-+
        // |   ;   | ; | | ; | |
        // +-+ ; +-+ ; +-+ ; + +
        if h && y > 0 && x < problem.width() - 1 {
            if place.right((Y(y - 1), X(x))) && (place.down((Y(y - 1), X(x))) || place.down((Y(y - 1), X(x + 1)))) { continue; }
            if place.down((Y(y - 1), X(x))) && place.down((Y(y - 1), X(x + 1))) { continue; }
        }
        if v && x > 0 {
            if place.right((Y(y), X(x - 1))) && place.down((Y(y), X(x - 1))) { continue; }
        }
        // +-+ ... +-+
        // |         |
        // + + ... + +
        if y > 0 && place.down((Y(y - 1), X(x))) {
            let mut w = 0;
            while x - 1 - w >= 0 && place.right((Y(y - 1), X(x - 1 - w))) {
                w += 1;
            }
            if w > 0 && place.down((Y(y - 1), X(x - w))) {
                let mut invalid = true;
                for x2 in (x - w)..x {
                    if place.right((Y(y), X(x2))) || (x2 != x && problem[(Y(y), X(x2))] < 0) {
                        invalid = false;
                        break;
                    }
                }
                if invalid {
                    continue;
                }
            }
        }

        if h {
            if join(&mut new_frontier, x, x + 1) { continue; }
        }

        if v {
            let below_cell = problem[(Y(y + 1), X(x))];
            if below_cell < 0 && connect_to_number(&mut new_frontier, x, below_cell) { continue; }
        } else {
            if forget(&mut new_frontier, x) { continue; }

            if y < problem.height() - 1 {
                let below_cell = problem[(Y(y + 1), X(x))];
                if below_cell < 0 {
                    new_frontier[x as usize] = below_cell;
                } else {
                    new_frontier[x as usize] = x;
                }
            }
        }

        if h { place.set_right((Y(y), X(x)), true); }
        if v { place.set_down((Y(y), X(x)), true); }

        search(y, x + 1, problem, &new_frontier, place, answers);

        if h { place.set_right((Y(y), X(x)), false); }
        if v { place.set_down((Y(y), X(x)), false); }
    }
}

/// Connect two open endpoints `i` and `j` in `frontier`.
/// Returns `true` iff this operation results in inconsistency.
fn join(frontier: &mut Frontier, i: i32, j: i32) -> bool {
    let i = i as usize;
    let j = j as usize;

    // Both endpoints must be open
    if frontier[i] == CLOSED_END || frontier[j] == CLOSED_END {
        return true;
    }

    let i_another = frontier[i];
    let j_another = frontier[j];
    frontier[i] = CLOSED_END;
    frontier[j] = CLOSED_END;

    match (i_another < 0, j_another < 0) {
        (true, true) => {
            if i_another != j_another {
                return true;
            }
        },
        (true, false) => frontier[j_another as usize] = i_another,
        (false, true) => frontier[i_another as usize] = j_another,
        (false, false) => {
            // Loops are not allowed
            if i_another as usize == j {
                return true;
            }
            frontier[i_another as usize] = j_another;
            frontier[j_another as usize] = i_another;
        }
    }
    false
}

/// Forget closed or isolated endpoint `i` in `frontier`.
/// Returns `true` iff this operation results in inconsistency.
fn forget(frontier: &mut Frontier, i: i32) -> bool {
    frontier[i as usize] != CLOSED_END && frontier[i as usize] != i
}

/// Connect endpoint `i` in `frontier` to a number cell with value `num` directly.
/// Returns `true` iff this operation results in inconsistency.
fn connect_to_number(frontier: &mut Frontier, i: i32, num: i32) -> bool {
    let i = i as usize;
    let another = frontier[i];
    if another == CLOSED_END {
        return true;
    }
    frontier[i] = CLOSED_END;

    if another < 0 {
        num != another
    } else {
        frontier[another as usize] = num;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_problem_test(input: &[&str]) {
        let height = (input.len() / 2 + 1) as i32;
        let width = (input[0].len() / 2 + 1) as i32;

        let mut problem = Grid::new(height, width, NO_CLUE);
        for y in 0..height {
            let mut row_iter = input[(y * 2) as usize].chars();

            for x in 0..width {
                if x > 0 { row_iter.next(); }
                let c = row_iter.next().unwrap();
                if '1' <= c && c <= '9' {
                    problem[(Y(y), X(x))] = Clue(((c as u8) - ('0' as u8)) as i32);
                }
            }
        }

        let ans = solve(&problem);
        assert_eq!(ans.len(), 1);

        let ans = &ans[0];

        for y in 0..(2 * height - 1) {
            let mut row_iter = input[y as usize].chars();

            for x in 0..(2 * width - 1) {
                let c = row_iter.next().unwrap();
                if y % 2 == 1 && x % 2 == 0 {
                    assert_eq!(ans.down((Y(y / 2), X(x / 2))), c == '|');
                } else if y % 2 == 0 && x % 2 == 1 {
                    assert_eq!(ans.right((Y(y / 2), X(x / 2))), c == '-', "{} {}", y, x);
                }
            }
        }
    }

    #[test]
    fn test_solver() {
        run_problem_test(&[
            "+-2 4-+-+",
            "|       |",
            "+ 1-+-1 4",
            "|        ",
            "+ 3 2-+-+",
            "| |     |",
            "+ +-+-3 +",
            "|       |",
            "+-+-+-+-+",
        ]);
    }
}
