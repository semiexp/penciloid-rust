extern crate puzrs;
extern crate rand;
extern crate getopts;

use getopts::Options;
use getopts::Matches;
use std::env;

use puzrs::*;
use std::io;
use std::time::Instant;
use rand::Rng;

use std::thread;
use std::sync::Mutex;
use std::io::Write;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

#[derive(Clone, Copy, Debug)]
struct GeneratorOption {
    height: i32,
    width: i32,
    jobs: i32,
    no_adjacent_clues: bool,
    symmetry_clue: bool,
    minimum_path_length: i32,
    empty_width: i32,
    max_clue: Option<i32>,
    corner: Option<(i32, i32)>,
}

fn run_generator(opts: GeneratorOption) {
    let height = opts.height;
    let width = opts.width;
    let mut ths = vec![];
    let gen_probs = std::sync::Arc::new(Mutex::new(0i64));

    for _ in 0..opts.jobs {
        let gen_probs = gen_probs.clone();
        let opts = opts;

        ths.push(thread::spawn(move || {
            let start = Instant::now();

            let mut generator = numberlink::PlacementGenerator::new(height, width);

            let mut rng = rand::thread_rng();
            loop {
                let mut end = Grid::new(height, width, numberlink::Endpoint::Any);
                
                let trans = move |x: i32, y: i32, d: i32| {
                    let mut x = x;
                    let mut y = y;
                    if d == 1 || d == 3 {
                        x = width - 1 - x;
                    }
                    if d == 2 || d == 3 {
                        y = height - 1 - y;
                    }
                    (Y(y), X(x))
                };
                let fr = opts.empty_width;
                for y in 0..height {
                    for x in 0..width {
                        if y < fr || y >= height - fr || x < fr || x >= width - fr {
                            end[(Y(y), X(x))] = numberlink::Endpoint::Prohibited;
                        }
                    }
                }
                if let Some((lo, hi)) = opts.corner {
                    if opts.symmetry_clue {
                        for d in 0..2 {
                            let i = rng.gen_range(lo, hi + 1);
                            let (d0, d1) = if d == 0 { (0, 3) } else { (1, 2) };

                            end[trans(i, i, d0)] = numberlink::Endpoint::Forced;
                            end[trans(i, i + 1, d0)] = numberlink::Endpoint::Prohibited;
                            end[trans(i + 1, i, d0)] = numberlink::Endpoint::Prohibited;
                            for j in 0..i {
                                end[trans(j, j, d0)] = numberlink::Endpoint::Prohibited;
                            }
                            end[trans(i, i, d1)] = numberlink::Endpoint::Forced;
                            end[trans(i, i + 1, d1)] = numberlink::Endpoint::Prohibited;
                            end[trans(i + 1, i, d1)] = numberlink::Endpoint::Prohibited;
                            for j in 0..i {
                                end[trans(j, j, d1)] = numberlink::Endpoint::Prohibited;
                            }
                        }
                    } else {
                        for d in 0..4 {
                            let i = rng.gen_range(lo, hi + 1);
                            end[trans(i, i, d)] = numberlink::Endpoint::Forced;
                            end[trans(i, i + 1, d)] = numberlink::Endpoint::Prohibited;
                            end[trans(i + 1, i, d)] = numberlink::Endpoint::Prohibited;
                            for j in 0..i {
                                end[trans(j, j, d)] = numberlink::Endpoint::Prohibited;
                            }
                        }
                    }
                }
                let opt = numberlink::GeneratorOption {
                    chain_threshold: opts.minimum_path_length,
                    endpoint_constraint: Some(&end),
                    forbid_adjacent_clue: opts.no_adjacent_clues,
                    symmetry_clue: opts.symmetry_clue,
                    clue_limit: opts.max_clue,
                };
                
                let placement = generator.generate(&opt, &mut rng);
                if let Some(placement) = placement {
                    // pretest
                    if !numberlink::uniqueness_pretest(&placement) { continue; }

                    let problem = numberlink::extract_problem(&placement, &mut rng);
                    let ans = numberlink::solve2(&problem, Some(2), false, true);

                    if ans.len() == 1 && !ans.found_not_fully_filled {
                        let stdin = io::stdout();
                        let handle = &mut stdin.lock();

                        let end = start.elapsed();
                        let cost_time = (end.as_secs() as f64 + end.subsec_nanos() as f64 / 1e9f64) / 60f64;
                        let mut cnt = gen_probs.lock().unwrap();
                        *cnt += 1;
                        eprintln!("{} problem(s) in {:.3}[min] ({:.3} [prob/min])", *cnt, cost_time, (*cnt) as f64 / cost_time);
                        
                        writeln!(handle, "{} {}", height, width).unwrap();
                        for y in 0..height {
                            for x in 0..width {
                                let numberlink::Clue(c) = problem[(Y(y), X(x))];
                                if c >= 1 {
                                    write!(handle, "{}{}", c, if x == width - 1 { '\n' } else { ' ' }).unwrap();
                                } else {
                                    write!(handle, ".{}", if x == width - 1 { '\n' } else { ' ' }).unwrap();
                                }
                            }
                        }
                        writeln!(handle).unwrap();
                    }
                }
            }
        }));
    }
    for th in ths {
        th.join().unwrap();
    }
}

fn parse_options(matches: Matches) -> Result<GeneratorOption, &'static str> {
    let height = try!(
        matches.opt_str("h")
            .ok_or("'height' must be specified")
            .and_then(|arg| arg.parse::<i32>().map_err(|_| "Could not parse value for 'height'"))
            .and_then(|arg| if arg > 0 { Ok(arg) } else { Err("'height' must be a positive integer") }));
    let width = try!(
        matches.opt_str("w")
            .ok_or("'width' must be specified")
            .and_then(|arg| arg.parse::<i32>().map_err(|_| "Could not parse value for 'width'"))
            .and_then(|arg| if arg > 0 { Ok(arg) } else { Err("'width' must be a positive integer") }));
    let jobs = try!(
        matches.opt_str("j").map(|s|
            s.parse::<i32>()
                .map_err(|_| "Could not parse value for 'jobs'")
                .and_then(|arg| if arg > 0 { Ok(arg) } else { Err("'jobs' must be a positive integer") })
        ).unwrap_or(Ok(1)));
    let no_adjacent_clues = matches.opt_present("no-adjacent-clues");
    let symmetry_clue = matches.opt_present("symmetry");
    let minimum_path_length = try!(
        matches.opt_str("minimum-path-length").map(|s|
            s.parse::<i32>()
                .map_err(|_| "Could not parse value for 'minimum-path-length'")
                .and_then(|arg| if arg > 0 { Ok(arg) } else { Err("'minimum-path-length' must be a positive integer") })
        ).unwrap_or(Ok(1)));
    let empty_width = try!(
        matches.opt_str("empty-width").map(|s|
            s.parse::<i32>()
                .map_err(|_| "Could not parse value for 'empty-width'")
                .and_then(|arg| if arg > 0 { Ok(arg) } else { Err("'empty-width' must be a positive integer") })
        ).unwrap_or(Ok(1)));
    let max_clue = try!(
        matches.opt_str("max-clue").map(|s|
            s.parse::<i32>()
                .map_err(|_| "Could not parse value for 'max-clue'")
                .and_then(|arg| if arg > 0 { Ok(Some(arg)) } else { Err("'max-clue' must be a positive integer") })
        ).unwrap_or(Ok(None)));
    let corner = match matches.opt_str("corner") {
        Some(s) => {
            let split = s.split(",").collect::<Vec<&str>>();
            if split.len() != 2 {
                return Err("Could not parse value for 'corner'");
            }
            let lo = try!(split[0].parse::<i32>().map_err(|_| "Could not parse value for 'corner'"));
            let hi = try!(split[1].parse::<i32>().map_err(|_| "Could not parse value for 'corner'"));
            if !(1 <= lo && lo <= hi) {
                return Err("'corner' must be a valid range on positive integers");
            }
            Some((lo, hi))
        },
        None => None,
    };
    Ok(GeneratorOption {
        height,
        width,
        jobs,
        no_adjacent_clues,
        symmetry_clue,
        minimum_path_length,
        empty_width,
        max_clue,
        corner
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut options = Options::new();
    options.optflag("", "help", "Show this help menu");
    options.optopt("h", "height", "Height of desired problems", "10");
    options.optopt("w", "width", "Width of desired problems", "10");
    options.optopt("j", "jobs", "Number of workers (threads)", "2");
    options.optflag("a", "no-adjacent-clues", "Disallow adjacent clues");
    options.optflag("s", "symmetry", "Force symmetry");
    options.optopt("m", "minimum-path-length", "Minimum length of paths in the answer", "12");
    options.optopt("e", "empty-width", "Disallow clues on n cell(s) from the outer border", "1");
    options.optopt("c", "corner", "Put one clue within specified range from each corner", "1,3");
    options.optopt("x", "max-clue", "Maximum value of clues", "10");
    
    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("error: {}", f);
            return;
        },
    };

    if matches.opt_present("help") {
        print_usage(&program, options);
        return;
    }

    let opt = match parse_options(matches) {
        Ok(opt) => opt,
        Err(f) => {
            eprintln!("error: {}", f);
            return;
        },
    };
    run_generator(opt);
}
