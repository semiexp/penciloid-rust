extern crate rand;

use super::super::Grid;
use super::*;

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use rand::Rng;

pub fn evaluate_parallel(
    n_threads: i32,
    problems: &Vec<Grid<Clue>>,
    param: EvaluatorParam,
) -> Vec<Option<f64>> {
    let res = Arc::new(Mutex::new(vec![None; problems.len()]));
    let checked = Arc::new(Mutex::new(0));

    let mut threads = vec![];

    for _ in 0..n_threads {
        let problems = problems.clone();
        let res = res.clone();
        let checked = checked.clone();
        let th = thread::spawn(move || loop {
            let id;
            {
                let mut handle = checked.lock().unwrap();
                if *handle >= problems.len() {
                    break;
                }
                id = *handle;
                *handle += 1;
            }
            let mut evaluator = Evaluator::new(&problems[id], param);
            (res.lock().unwrap())[id] = evaluator.evaluate();
        });
        threads.push(th);
    }

    for th in threads {
        th.join().unwrap();
    }

    let ret = res.lock().unwrap().clone();
    ret
}
pub fn evaluate_score(score: &[Option<f64>], expected: &[f64]) -> f64 {
    let mut sx = 0.0f64;
    let mut sxx = 0.0f64;
    let mut sy = 0.0f64;
    let mut sxy = 0.0f64;
    let mut n = 0.0f64;

    for i in 0..score.len() {
        if let Some(x) = score[i] {
            let x = x.ln();
            let y = expected[i].ln();

            sx += x;
            sxx += x * x;
            sy += y;
            sxy += x * y;
            n += 1.0f64;
        }
    }

    let a = (n * sxy - sx * sy) / (n * sxx - sx * sx);
    let b = (sxx * sy - sxy * sx) / (n * sxx - sx * sx);

    let mut df = 0.0f64;
    for i in 0..score.len() {
        if let Some(x) = score[i] {
            let x = x.ln();
            let y = expected[i].ln();

            let yh = a * x + b;
            df += (y - yh) * (y - yh);
        }
    }

    (df / n).sqrt()
}
fn param_value(p: &mut EvaluatorParam, idx: i32) -> &mut f64 {
    match idx {
        0 => &mut p.unique_elimination,
        1 => &mut p.small_large_elimination,
        2 => &mut p.small_large_elimination_easy,
        3 => &mut p.small_large_decision_remaining_cells_penalty,
        4 => &mut p.small_large_decision_all_cells_penalty,
        5 => &mut p.small_large_decision_easy_multiplier,
        6 => &mut p.small_large_decision_additive_penalty,
        7 => &mut p.small_large_decision_easy_additive_penalty,
        8 => &mut p.two_cells_propagation_half_elimination,
        9 => &mut p.two_cells_propagation_propagate_penalty,
        _ => unreachable!(),
    }
}

pub fn train(
    start: EvaluatorParam,
    problems: &Vec<Grid<Clue>>,
    expected: &Vec<f64>,
) -> EvaluatorParam {
    let n_threads = 10;

    let mut param = start;
    let mut current_score =
        evaluate_score(&evaluate_parallel(n_threads, problems, param), expected);
    let mut temp = 0.001f64;

    let mut rng = rand::thread_rng();

    for _ in 0..500 {
        let mut move_cand = vec![];
        for i in 0..10 {
            let step = if i == 3 || i == 4 { 0.01 } else { 0.1 };
            if !(*param_value(&mut param, i) - step < 1e-8f64) {
                move_cand.push((i, -step));
            }
            move_cand.push((i, step));
        }

        rng.shuffle(&mut move_cand);

        for mv in move_cand {
            let mut param2 = param;
            *param_value(&mut param2, mv.0) += mv.1;

            let score2 = evaluate_score(&evaluate_parallel(n_threads, problems, param2), expected);

            if current_score > score2 || rng.gen::<f64>() < ((current_score - score2) / temp).exp()
            {
                param = param2;
                current_score = score2;
                break;
            }
        }

        eprintln!("{:?}", param);
        eprintln!("{}\n", current_score);

        temp *= 0.995f64;
    }

    param
}
