use std::io;
use std::fmt;
use std::error;
use std::num::ParseIntError;
use std::io::BufRead;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Format
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::Format => write!(f, "Format error"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Format => "Format error",
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Error {
        Error::Format
    }
}

fn is_comment(s: &String) -> bool {
    s.chars().next().unwrap() == '#'
}

pub fn next_valid_line(reader: &mut BufRead, buf: &mut String) -> io::Result<usize> {
    loop {
        buf.clear();
        let len = try!(reader.read_line(buf));

        if !buf.trim().is_empty() && !is_comment(buf) {
            return Ok(len);
        }
    }
}
