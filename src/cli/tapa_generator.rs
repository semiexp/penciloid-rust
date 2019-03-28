use super::*;
use {tapa, Grid, X, Y};
use rand;

use std::sync::{Arc, Mutex};
use std::io::{stdout, Write};
use std::thread;
use super::getopts::{Matches, Options};

#[derive(Clone, Copy, Debug)]
struct GeneratorOption {
    height: i32,
    width: i32,
    jobs: i32,
    symmetry: bool,
    max_clue: Option<i32>,
    trial_and_error: bool,
}

fn parse_options(matches: &Matches) -> Result<GeneratorOption, CliError> {
    let height = parse_integer_argument(matches, "height", None, Some(1))?;
    let width = parse_integer_argument(matches, "width", None, Some(1))?;
    let jobs = parse_integer_argument(matches, "jobs", Some(1), Some(1))?;
    let symmetry = matches.opt_present("symmetry");
    let trial_and_error = matches.opt_present("trial-and-error");

    let max_clue = matches
        .opt_str("max-clue")
        .map(|s| {
            s.parse::<i32>()
                .map_err(|_| CliError::UnrecognizedArgument("max-clue"))
                .and_then(|arg| {
                    if arg > 0 {
                        Ok(Some(arg))
                    } else {
                        Err(CliError::UnrecognizedArgument("max-clue"))
                    }
                })
        })
        .unwrap_or(Ok(None))?;
    Ok(GeneratorOption {
        height,
        width,
        jobs,
        symmetry,
        max_clue,
        trial_and_error,
    })
}

fn run_generator(opts: GeneratorOption) -> Result<(), CliError> {
    let height = opts.height;
    let width = opts.width;
    let mut ths = vec![];
    let gen_probs = Arc::new(Mutex::new(0i64));

    for _ in 0..opts.jobs {
        let gen_probs = gen_probs.clone();

        let opts = opts;

        ths.push(thread::spawn(move || {
            let start = Instant::now();
            let dic = tapa::Dictionary::new();
            let consecutive_dic = tapa::ConsecutiveRegionDictionary::new(&dic);

            let opts = tapa::GeneratorOption {
                clue_constraint: Grid::new(opts.height, opts.width, tapa::ClueConstraint::Any),
                symmetry: opts.symmetry,
                max_clue: opts.max_clue,
                use_trial_and_error: opts.trial_and_error,
            };

            let mut rng = rand::thread_rng();
            loop {
                let res = tapa::generate(&opts, &dic, &consecutive_dic, &mut rng);

                if let Some(problem) = res {
                    let stdin = io::stdout();
                    let handle = &mut stdin.lock();

                    let end = start.elapsed();
                    let cost_time =
                        (end.as_secs() as f64 + end.subsec_nanos() as f64 / 1e9f64) / 60f64;
                    let mut cnt = gen_probs.lock().unwrap();
                    *cnt += 1;
                    eprintln!(
                        "{} problem(s) in {:.3}[min] ({:.3} [prob/min])",
                        *cnt,
                        cost_time,
                        (*cnt) as f64 / cost_time
                    );

                    writeln!(handle, "{} {}", height, width).unwrap();
                    for y in 0..height {
                        for x in 0..width {
                            let tapa::Clue(c) = problem[(Y(y), X(x))];
                            if c == 0 {
                                write!(handle, "0...").unwrap();
                            } else if c > 0 {
                                for i in 0..4 {
                                    let v = tapa::CLUE_VALUES[c as usize][i];
                                    if v > 0 {
                                        write!(handle, "{}", v).unwrap();
                                    } else {
                                        write!(handle, ".").unwrap();;
                                    }
                                }
                            } else {
                                write!(handle, "....").unwrap();
                            }
                            write!(handle, "{}", if x == width - 1 { '\n' } else { ' ' }).unwrap();
                        }
                    }
                    writeln!(handle).unwrap();
                }
            }
        }));
    }
    for th in ths {
        th.join().unwrap();
    }
    Ok(())
}

pub fn tapa_generator_frontend(args: &[String], program: &str) -> Result<(), CliError> {
    let mut options = Options::new();
    options.optflag("", "help", "Show this help menu");
    options.optopt("h", "height", "Height of desired problems", "10");
    options.optopt("w", "width", "Width of desired problems", "10");
    options.optopt("j", "jobs", "Number of workers (threads)", "2");
    options.optflag("s", "symmetry", "Force symmetry");
    options.optflag("t", "trial-and-error", "Use trial and error");
    options.optopt("x", "max-clue", "Maximum value of clues", "10");

    let matches = options.parse(&args[..])?;

    if matches.opt_present("help") {
        let brief = format!("Usage: {} tapa-gen [options]", program);
        print!("{}", options.usage(&brief));
        return Ok(());
    }

    let opts = parse_options(&matches)?;
    run_generator(opts)
}
