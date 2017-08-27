use super::super::{Grid, Coord};
use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum EvCand {
    Ok,
    Elim(f64),
}

#[derive(Debug)]
enum Move {
    Decide(f64, usize, i32),
    Elim(f64, Vec<(usize, i32)>),
}

pub struct Evaluator {
    n_cells: usize,
    shape: FieldShape,
    val: Vec<i32>,
    clue: Vec<i32>,
    cand_score: Vec<[EvCand; (MAX_VAL + 1) as usize]>, // 1-origin for simplicity
    total_score: f64,
    inconsistent: bool,
    move_cand: Vec<Move>,
}

const UNIQUE_ELIMINATION: f64 = 0.8f64;
const SMALL_LARGE_ELIMINATION: f64 = 1.0f64;

impl Evaluator {
    pub fn new(problem: &Grid<Clue>) -> Evaluator {
        let n_cells = (problem.height() * problem.width()) as usize;

        let mut has_clue = Grid::new(problem.height(), problem.width(), false);
        for i in 0..n_cells {
            if let Clue::Clue { vertical: _, horizontal: _ } = problem[i] {
                has_clue[i] = true;
            }
        }
        let shape = FieldShape::new(&has_clue);
        let mut clue = vec![0; shape.group_to_cells.len()];
        for i in 0..shape.group_to_cells.len() {
            let clue_loc = shape.clue_locations[i];
            clue[i] = match clue_loc {
                ClueLocation::Horizontal(h) => match problem[h] {
                    Clue::Clue { horizontal: h, vertical: _ } => h,
                    _ => unreachable!(),
                },
                ClueLocation::Vertical(v) => match problem[v] {
                    Clue::Clue { horizontal: _, vertical: v } => v,
                    _ => unreachable!(),
                },
            }
        }
        Evaluator {
            n_cells: n_cells,
            shape: shape,
            val: vec![UNDECIDED; n_cells],
            clue: clue,
            cand_score: vec![[EvCand::Ok; (MAX_VAL + 1) as usize]; n_cells],
            total_score: 0.0f64,
            inconsistent: false,
            move_cand: vec![],
        }
    }
    pub fn evaluate(&mut self) -> Option<f64> {
        let mut n_undecided = 0;
        for i in 0..self.n_cells {
            if !self.shape.has_clue[i] {
                n_undecided += 1;
            }
        }
        let mut decided_locs = vec![];
        loop {
            self.move_cand.clear();

            // apply possible eliminations
            self.simple_elimination();
            self.unique_decision();
            self.unique_elimination();
            self.two_cells_propagation();
            self.naked_pair();

            self.remove_unnecessary_move();

            if self.move_cand.len() == 0 {
                break;
            }

            let mut lowest_decision = (1e20f64, 0, 0);
            let mut lowest_elim = (1e20f64, vec![]);
            for mv in &self.move_cand {
                match mv {
                    &Move::Decide(score, loc, v) => if score < lowest_decision.0 {
                        lowest_decision = (score, loc, v);
                    },
                    &Move::Elim(score, ref elims) => if score < lowest_elim.0 {
                        lowest_elim = (score, elims.clone());
                    }
                }
            }
            if lowest_elim.0 < lowest_decision.0 * 11.0f64 {
                // use elim
                for (loc, v) in lowest_elim.1 {
                    self.cand_score[loc][v as usize] = EvCand::Elim(lowest_elim.0);
                }
            } else {
                // use lowest_decision
                self.apply_locality(&decided_locs);

                let mut move_score = 0.0f64;
                for mv in &self.move_cand {
                    if let &Move::Decide(score, _, _) = mv {
                        move_score += score.powf(-2.0f64);
                    }
                }
                move_score = move_score.powf(-1.0 / 2.0f64);
                move_score *= (n_undecided as f64).powf(0.3);

                self.total_score += move_score;
                self.val[lowest_decision.1] = lowest_decision.2;
                n_undecided -= 1;
                decided_locs.push(lowest_decision.1);
            }
        }
        if n_undecided == 0 {
            Some(self.total_score)
        } else {
            None
        }
    }
    fn remove_unnecessary_move(&mut self) {
        let mut new_cand = vec![];
        for m in &self.move_cand {
            match m {
                &Move::Decide(score, loc, v) => 
                    if self.val[loc] == UNDECIDED {
                        new_cand.push(Move::Decide(score, loc, v))
                    },
                &Move::Elim(score, ref elims) => {
                    let mut new_elims = vec![];
                    for &(loc, v) in elims {
                        let isok = match self.cand_score[loc][v as usize] {
                            EvCand::Ok => true,
                            EvCand::Elim(sc) => score < sc,
                        };
                        if isok {
                            new_elims.push((loc, v));
                        }
                    }
                    if new_elims.len() > 0 {
                        new_cand.push(Move::Elim(score, new_elims));
                    }
                },
            }
        }
        self.move_cand = new_cand;
    }
    fn apply_locality(&mut self, decided_locs: &Vec<usize>) {
        let mut last_moves = vec![];
        let last_cnt = if decided_locs.len() < 2 { decided_locs.len() } else { 2 };

        for i in 0..last_cnt {
            let loc = decided_locs[decided_locs.len() - 1 - i];
            let y = loc as i32 / self.shape.has_clue.width();
            let x = loc as i32 % self.shape.has_clue.width();
            last_moves.push((y, x));
        }
        for mut m in &mut self.move_cand {
            match m {
                &mut Move::Decide(ref mut score, loc, _) => {
                    let y = loc as i32 / self.shape.has_clue.width();
                    let x = loc as i32 % self.shape.has_clue.width();
                    for i in 0..last_cnt {
                        let dist = (last_moves[i].0 - y).abs() + (last_moves[i].1 - x).abs();

                        let mut dim = 1.0;
                        if dist < 5 {
                            //print!("{} -> ", score);
                            dim *= 1.0f64 - (7 - dist) as f64 / 10.0f64;
                            //println!("{}", score);
                        }
                        *score *= dim;
                    }
                },
                _ => (),
            }
        }
    }
    fn simple_elimination(&mut self) {
        for gi in 0..self.shape.group_to_cells.len() {
            let mut used = Cand(0);
            for c in self.shape.group_to_cells[gi] {
                if self.val[c] != UNDECIDED {
                    used |= Cand::singleton(self.val[c]);
                }
            }
            let rem_sum = self.clue[gi] - used.cand_sum();
            let rem_cells = self.shape.group_to_cells[gi].size() as i32 - used.count_set_cands();

            if rem_cells == 0 {
                continue;
            }
            if rem_cells == 1 {
                let mut ep = 0;
                for c in self.shape.group_to_cells[gi] {
                    if self.val[c] == UNDECIDED {
                        ep = c;
                    }
                }
                self.move_cand.push(Move::Decide(4.0, ep, rem_sum));
                continue;
            }

            let mut allowed = !used;
            let mut required = Cand(0);
            // eliminate too large candidates
            {
                let (low, high) = allowed.take_smallest_k(rem_cells - 1);
                let sum_small = low.cand_sum();
                let kth_smallest = high.smallest_set_cand();
                let max_allowed = rem_sum - sum_small;

                if max_allowed < kth_smallest {
                    self.inconsistent = true;
                    return;
                }
                if max_allowed == kth_smallest + 1 {
                    allowed = allowed.exclude(kth_smallest);
                }
                allowed = allowed.limit_upper_bound(max_allowed);
                if max_allowed == kth_smallest {
                    required |= low | Cand::singleton(kth_smallest);
                } else if max_allowed == kth_smallest + 1 {
                    required |= low | Cand::singleton(kth_smallest + 1);
                }
            }
            {
                let (high, low) = allowed.take_largest_k(rem_cells - 1);
                let sum_large = high.cand_sum();
                let kth_largest = low.largest_set_cand();
                let min_allowed = rem_sum - sum_large;

                if min_allowed > kth_largest {
                    self.inconsistent = true;
                    return;
                }
                if min_allowed == kth_largest - 1 {
                    allowed = allowed.exclude(kth_largest);
                }
                allowed = allowed.limit_lower_bound(min_allowed);
                if min_allowed == kth_largest {
                    required |= high | Cand::singleton(kth_largest);
                } else if min_allowed == kth_largest - 1 {
                    required |= high | Cand::singleton(kth_largest - 1);
                }
            }
            
            let mut elims = vec![];
            for n in 1..(MAX_VAL + 1) {
                if !allowed.is_set(n) {
                    for c in self.shape.group_to_cells[gi] {
                        if self.val[c] == UNDECIDED {
                            elims.push((c, n));
                        }
                    }
                }
            }
            self.move_cand.push(Move::Elim(SMALL_LARGE_ELIMINATION, elims));

            for n in 1..(MAX_VAL + 1) {
                if required.is_set(n) {
                    let mut cost = 0.0f64;
                    let mut pos = None;
                    let mut twice = false;
                    for c in self.shape.group_to_cells[gi] {
                        if self.val[c] == UNDECIDED {
                            if let EvCand::Elim(s) = self.cand_score[c][n as usize] {
                                cost += s;
                            } else {
                                if pos == None {
                                    pos = Some(c);
                                } else {
                                    twice = true;
                                    break;
                                }
                            }
                        } else if self.val[c] == n {
                            twice = true;
                            break;
                        }
                    }
                    if !twice {
                        if let Some(c) = pos {
                            let multiplier = 1.0f64 + rem_cells as f64 / 10.0f64 + self.shape.group_to_cells[gi].size() as f64 / 15.0f64;
                            self.move_cand.push(Move::Decide((cost + 3.0) * multiplier, c, n));
                        } else {
                            self.inconsistent = true;
                            return;
                        }
                    }
                }
            }
        }
    }
    fn unique_elimination(&mut self) {
        for i in 0..self.n_cells {
            if !self.shape.has_clue[i] && self.val[i] != UNDECIDED {
                let (g1, g2) = self.shape.cell_to_groups[i];
                let mut mv = vec![];
                for c in self.shape.group_to_cells[g1] {
                    if c != i {
                        mv.push((c, self.val[i]));
                    }
                }
                for c in self.shape.group_to_cells[g2] {
                    if c != i {
                        mv.push((c, self.val[i]));
                    }
                }
                self.move_cand.push(Move::Elim(UNIQUE_ELIMINATION, mv));
            }
        }
    }
    fn unique_decision(&mut self) {
        for i in 0..self.n_cells {
            if !self.shape.has_clue[i] {
                let mut cost = 0.0f64;
                let mut cand = -1;
                for v in 1..(MAX_VAL + 1) {
                    match self.cand_score[i][v as usize] {
                        EvCand::Ok => if cand == -1 {
                            cand = v;
                        } else {
                            cand = -2;
                        },
                        EvCand::Elim(c) => cost += c,
                    }
                }
                if cand == -1 {
                    self.inconsistent = true;
                    return;
                } else if cand != -2 {
                    self.move_cand.push(Move::Decide(cost, i, cand));
                }
            }
        }
    }
    fn two_cells_propagation(&mut self) {
        for gi in 0..self.shape.group_to_cells.len() {
            let mut used_mask = 0u32; // bit indices are 1-origin
            let mut rem_sum = self.clue[gi];
            let mut undet1 = None;
            let mut undet2 = None;
            let mut more_than_two = false;
            for c in self.shape.group_to_cells[gi] {
                if self.val[c] == UNDECIDED {
                    if undet1 == None {
                        undet1 = Some(c);
                    } else if undet2 == None {
                        undet2 = Some(c);
                    } else {
                        more_than_two = true;
                    }
                } else {
                    rem_sum -= self.val[c];
                }
            }
            if !more_than_two && undet2 != None {
                if let (Some(c1), Some(c2)) = (undet1, undet2) {
                    if rem_sum % 2 == 0 {
                        let half = rem_sum / 2;
                        if 1 <= half && half <= MAX_VAL {
                            self.move_cand.push(Move::Elim(3.0f64, vec![(c1, half), (c2, half)]));
                        }
                    }
                    for n in 1..(MAX_VAL + 1) {
                        if let EvCand::Elim(s) = self.cand_score[c1][n as usize] {
                            let n2 = rem_sum - n;
                            if 1 <= n2 && n2 <= MAX_VAL {
                                self.move_cand.push(Move::Elim(s + 5.0f64, vec![(c2, n2)]));
                            }
                        }
                        if let EvCand::Elim(s) = self.cand_score[c2][n as usize] {
                            let n2 = rem_sum - n;
                            if 1 <= n2 && n2 <= MAX_VAL {
                                self.move_cand.push(Move::Elim(s + 5.0f64, vec![(c1, n2)]));
                            }
                        }
                    }
                } else {
                    unreachable!();
                }
            }
        }
    }
    fn naked_pair(&mut self) {
        for gi in 0..self.shape.group_to_cells.len() {
            let mut two_cand_cells = vec![];
            for c1 in self.shape.group_to_cells[gi] {
                let mut oks = 0u32;
                let mut score = 0.0f64;
                for n in 1..(MAX_VAL + 1) {
                    if let EvCand::Elim(s) = self.cand_score[c1][n as usize] {
                        score += s;
                    } else {
                        oks |= 1u32 << n;
                    }
                }
                if oks.count_ones() == 2 {
                    two_cand_cells.push((c1, oks, score));
                }
            }
            for i in 0..two_cand_cells.len() {
                for j in (i + 1)..two_cand_cells.len() {
                    if two_cand_cells[i].1 == two_cand_cells[j].1 {
                        let mut elims = vec![];
                        let x = two_cand_cells[i].1.trailing_zeros() as i32;
                        let y = (two_cand_cells[i].1 ^ (1u32 << x)).trailing_zeros() as i32;
                        for c in self.shape.group_to_cells[gi] {
                            if two_cand_cells[i].0 != c && two_cand_cells[j].0 != c {
                                elims.push((c, x));
                                elims.push((c, y));
                            }
                        }
                        self.move_cand.push(Move::Elim(two_cand_cells[i].2 + two_cand_cells[j].2, elims));
                    }
                }
            }
        }
    }
}
