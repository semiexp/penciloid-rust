use super::super::{Grid, D, P};
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

pub struct GeneratorOption {
    pub clue_constraint: Grid<ClueConstraint>,
    pub max_clue: Option<i32>,
    pub symmetry: bool,
    pub use_trial_and_error: bool,
    pub allowed_clues: Option<u32>,
}

enum HasClueHistory {
    Update(P, bool),
    Checkpoint,
}
struct HasClue {
    has_clue: Grid<bool>,
    n_clues: i32,
    history: Vec<HasClueHistory>,
}

impl HasClue {
    fn new(height: i32, width: i32) -> HasClue {
        HasClue {
            has_clue: Grid::new(height, width, false),
            n_clues: 0,
            history: vec![],
        }
    }

    fn update(&mut self, loc: P, val: bool) {
        if self.has_clue[loc] == val {
            return;
        }
        if val {
            self.n_clues += 1;
        } else {
            self.n_clues -= 1;
        }
        self.history.push(HasClueHistory::Update(loc, !val));
        self.has_clue[loc] = val;
    }
    fn get(&self, loc: P) -> bool {
        self.has_clue[loc]
    }
    fn get_checked(&self, loc: P) -> bool {
        self.has_clue.is_valid_p(loc) && self.has_clue[loc]
    }
    fn add_checkpoint(&mut self) {
        self.history.push(HasClueHistory::Checkpoint);
    }
    fn rollback(&mut self) {
        loop {
            match self.history.pop() {
                Some(HasClueHistory::Update(cd, v)) => {
                    self.has_clue[cd] = v;
                    if v {
                        self.n_clues += 1;
                    } else {
                        self.n_clues -= 1;
                    }
                }
                _ => break,
            }
        }
    }
    fn forget_history(&mut self) {
        self.history.clear();
    }
}

pub fn generate<R: Rng>(
    opts: &GeneratorOption,
    dic: &Dictionary,
    consecutive_dic: &ConsecutiveRegionDictionary,
    rng: &mut R,
) -> Option<Grid<Clue>> {
    let height = opts.clue_constraint.height();
    let width = opts.clue_constraint.width();
    let mut has_clue = HasClue::new(height, width);

    let mut problem = Grid::new(height, width, NO_CLUE);
    let mut field = Field::new(height, width, dic, consecutive_dic);
    let mut current_energy = 0i32;

    for y in 0..height {
        for x in 0..width {
            let pos = P(y, x);
            if opts.clue_constraint[pos] == ClueConstraint::Forced {
                has_clue.update(pos, true);
            }
        }
    }

    let allowed_clues = opts.allowed_clues.unwrap_or((1 << 23) - 1);
    let n_step = height * width * 10;
    let mut temperature = 20f64;

    for s in 0..n_step {
        let mut update_cand = vec![];
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if field.cell(pos) == Cell::Black
                    || opts.clue_constraint[pos] == ClueConstraint::Prohibited
                {
                    continue;
                }
                if opts.symmetry {
                    let y2 = height - 1 - y;
                    let x2 = width - 1 - x;
                    if -1 <= y - y2
                        && y - y2 <= 1
                        && -1 <= x - x2
                        && x - x2 <= 1
                        && (y != y2 || x != x2)
                    {
                        continue;
                    }
                }
                let mut isok = true;
                for dy in -1..2 {
                    for dx in -1..2 {
                        if (dy != 0 || dx != 0) && has_clue.get_checked(pos + D(dy, dx)) {
                            isok = false;
                        }
                    }
                }
                let mut isok2 = false;
                for dy in -2..3 {
                    for dx in -2..3 {
                        let loc = pos + D(dy, dx);
                        if field.cell_checked(loc) == Cell::Undecided {
                            isok2 = true;
                        }
                    }
                }
                if (has_clue.get(pos) && problem[pos] == NO_CLUE)
                    || (isok && isok2 && (problem[pos] != NO_CLUE || rng.gen::<f64>() < 1.0))
                {
                    for v in (-1)..(CLUE_TYPES as i32) {
                        if v == -1 && opts.clue_constraint[pos] == ClueConstraint::Forced {
                            continue;
                        }
                        if v >= 0 && ((allowed_clues >> v) & 1) == 0 {
                            continue;
                        }
                        let next_clue = Clue(v);
                        if problem[pos] != next_clue {
                            update_cand.push((pos, next_clue));
                        }
                    }
                }
            }
        }

        rng.shuffle(&mut update_cand);

        let mut updated = false;

        for &(loc, clue) in &update_cand {
            let previous_clue = problem[loc];
            let P(y, x) = loc;

            has_clue.add_checkpoint();
            if opts.symmetry {
                let loc2 = P(height - 1 - y, width - 1 - x);
                if clue == NO_CLUE {
                    if problem[loc2] == NO_CLUE {
                        has_clue.update(loc, false);
                        has_clue.update(loc2, false);
                    }
                } else {
                    has_clue.update(loc, true);
                    has_clue.update(loc2, true);
                }
            } else {
                has_clue.update(loc, clue != NO_CLUE);
            }
            if let Some(max_clue) = opts.max_clue {
                if max_clue < has_clue.n_clues {
                    has_clue.rollback();
                    continue;
                }
            }

            problem[loc] = clue;
            let next_field = if previous_clue == NO_CLUE {
                let mut f = field.clone();
                f.add_clue(loc, clue);
                f.solve();
                if opts.use_trial_and_error {
                    f.trial_and_error();
                }
                f
            } else {
                solve_test(
                    &problem,
                    &has_clue,
                    opts.use_trial_and_error,
                    dic,
                    consecutive_dic,
                )
            };
            let energy = next_field.decided_cells() - 4 * has_clue.n_clues;

            let update = !next_field.inconsistent()
                && (current_energy < energy
                    || rng.gen::<f64>() < ((energy - current_energy) as f64 / temperature).exp());

            if update {
                current_energy = energy;
                field = next_field;

                let mut clue_filled = true;
                for y in 0..height {
                    for x in 0..width {
                        if has_clue.get(loc) && problem[loc] == NO_CLUE {
                            clue_filled = false;
                        }
                    }
                }
                if field.fully_solved() && clue_filled {
                    return Some(problem);
                }
                updated = true;
                has_clue.forget_history();
                break;
            }
            problem[loc] = previous_clue;
            has_clue.rollback();
        }

        if !updated {
            break;
        }

        temperature *= 0.995f64;
    }

    None
}

fn solve_test<'a, 'b>(
    problem: &Grid<Clue>,
    has_clue: &HasClue,
    use_trial_and_error: bool,
    dic: &'a Dictionary,
    consecutive_dic: &'b ConsecutiveRegionDictionary,
) -> Field<'a, 'b> {
    let height = problem.height();
    let width = problem.width();
    let mut ret = Field::new(height, width, dic, consecutive_dic);

    for y in 0..height {
        for x in 0..width {
            let pos = P(y, x);
            let clue = problem[pos];
            if clue != NO_CLUE {
                ret.add_clue(pos, clue);
            } else if has_clue.get(pos) {
                ret.decide(pos, Cell::White);
            }

            if ret.inconsistent() {
                return ret;
            }
        }
    }

    ret.solve();
    if use_trial_and_error {
        ret.trial_and_error();
    }

    ret
}
