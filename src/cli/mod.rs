extern crate getopts;

macro_rules! run_timed {
    ($timer: ident, $flag: ident, $e: expr) => {
        if $flag {
            let start = Instant::now();
            let ret = $e;
            let end = start.elapsed();
            let cost_time = end.as_secs() as f64 + end.subsec_nanos() as f64 / 1e9f64;

            let mut timer_lock = $timer.lock().unwrap();
            *timer_lock += cost_time;

            ret
        } else {
            $e
        }
    };
}

use std::env;
use std::io;
use std::error;
use std::fmt::{self, Debug, Display};
use std::time::Instant;

pub mod nl_generator;
pub mod tapa_generator;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Puzzle {
    Numberlink,
    Slitherlink,
    Kakuro,
    Tapa,
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Mode {
    Solver,
    Generator,
}

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Getopts(getopts::Fail),
    InvalidSubcommand,
    MissingOption(&'static str),
    UnrecognizedArgument(&'static str),
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}
impl From<getopts::Fail> for CliError {
    fn from(err: getopts::Fail) -> CliError {
        CliError::Getopts(err)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref err) => Display::fmt(err, f),
            CliError::Getopts(ref err) => Display::fmt(err, f),
            CliError::InvalidSubcommand => write!(f, "invalid subcommand"),
            CliError::MissingOption(opt) => write!(f, "missing a required option '{}'", opt),
            CliError::UnrecognizedArgument(opt) => {
                write!(f, "unrecognized argument for option '{}'", opt)
            }
        }
    }
}

impl error::Error for CliError {
    fn description(&self) -> &str {
        match *self {
            CliError::Io(ref err) => err.description(),
            CliError::Getopts(ref err) => err.description(),
            CliError::InvalidSubcommand => "invalid subcommand",
            CliError::MissingOption(_) => "missing a required option",
            CliError::UnrecognizedArgument(_) => "unrecognized argument",
        }
    }
}

fn parse_subcommand(subcommand: &str) -> Result<(Puzzle, Mode), CliError> {
    let tokens: Vec<&str> = subcommand.split('-').collect();

    if tokens.len() != 2 {
        return Err(CliError::InvalidSubcommand);
    }

    let puzzle = match tokens[0].to_ascii_lowercase().as_str() {
        "nl" | "numberlink" => Some(Puzzle::Numberlink),
        "sl" | "slitherlink" => Some(Puzzle::Slitherlink),
        "kk" | "kakuro" => Some(Puzzle::Kakuro),
        "tp" | "tapa" => Some(Puzzle::Tapa),
        _ => None,
    };
    let mode = match tokens[1].to_ascii_lowercase().as_str() {
        "sol" | "solver" => Some(Mode::Solver),
        "gen" | "generator" => Some(Mode::Generator),
        _ => None,
    };

    return match (puzzle, mode) {
        (Some(puzzle), Some(mode)) => Ok((puzzle, mode)),
        _ => Err(CliError::InvalidSubcommand),
    };
}

fn parse_integer_argument(
    matches: &getopts::Matches,
    name: &'static str,
    default: Option<i32>,
    lower_bound: Option<i32>,
) -> Result<i32, CliError> {
    let res = matches
        .opt_str(name)
        .map(|s| {
            s.parse::<i32>()
                .map(Option::Some)
                .map_err(|_| CliError::UnrecognizedArgument(name))
        })
        .unwrap_or(Ok(default))?
        .ok_or(CliError::MissingOption(name))?;
    if !lower_bound.map(|lb| res >= lb).unwrap_or(true) {
        return Err(CliError::UnrecognizedArgument(name));
    }
    Ok(res)
}

pub fn run_cli() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    if args.len() < 2 {
        println!("error: no subcommand given");
        return;
    }
    // parse subcommand
    let subcommand = parse_subcommand(&args[1]);

    let result = subcommand.and_then(|subcommand| match subcommand {
        (Puzzle::Numberlink, Mode::Generator) => {
            nl_generator::nl_generator_frontend(&args[2..], &program)
        },
        (Puzzle::Tapa, Mode::Generator) => {
            tapa_generator::tapa_generator_frontend(&args[2..], &program)
        },
        _ => unimplemented!(),
    });
    if result.is_err() {
        println!("error: {}", result.unwrap_err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontend_parse_subcommand() {
        assert_eq!(
            parse_subcommand("nl-gen").unwrap(),
            (Puzzle::Numberlink, Mode::Generator)
        );
        assert_eq!(
            parse_subcommand("numberlink-generator").unwrap(),
            (Puzzle::Numberlink, Mode::Generator)
        );
        assert_eq!(
            parse_subcommand("slitherlink-sol").unwrap(),
            (Puzzle::Slitherlink, Mode::Solver)
        );
        assert_eq!(
            parse_subcommand("sl-gen").unwrap(),
            (Puzzle::Slitherlink, Mode::Generator)
        );
        assert_eq!(
            parse_subcommand("kakuro-gen").unwrap(),
            (Puzzle::Kakuro, Mode::Generator)
        );
        assert!(parse_subcommand("nosuchpuzzle-gen").is_err());
        assert!(parse_subcommand("a-b-c").is_err());
    }
}
