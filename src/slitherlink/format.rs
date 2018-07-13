use std::io::BufRead;

use common::{Grid, X, Y};
use format::{next_valid_line, Error};

use super::*;

pub fn read_penciloid_problem<T: BufRead>(reader: &mut T) -> Result<Grid<Clue>, Error> {
    let mut buffer = String::new();

    let height;
    let width;

    {
        try!(next_valid_line(reader, &mut buffer));
        let mut header = buffer.split(' ');
        height = try!(
            try!(header.next().ok_or(Error::Format))
                .trim()
                .parse::<i32>()
        );
        width = try!(
            try!(header.next().ok_or(Error::Format))
                .trim()
                .parse::<i32>()
        );
    }

    let mut ret = Grid::new(height, width, NO_CLUE);

    for y in 0..height {
        try!(next_valid_line(reader, &mut buffer));
        let mut row_iter = buffer.chars();

        for x in 0..width {
            let c = try!(row_iter.next().ok_or(Error::Format));
            match c {
                '0' | '1' | '2' | '3' => ret[(Y(y), X(x))] = Clue((c as u8 - '0' as u8) as i32),
                _ => (),
            }
        }
    }

    Ok(ret)
}
