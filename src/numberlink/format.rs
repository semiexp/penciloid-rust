use std::io::BufRead;

use super::*;
use common::{Grid, X, Y};
use format::{next_valid_line, Error};

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
        let mut row = buffer.split(' ');

        for x in 0..width {
            let val = try!(row.next().ok_or(Error::Format)).trim();
            let num = val.parse::<i32>();

            if let Ok(num) = num {
                ret[(Y(y), X(x))] = Clue(num);
            }
        }
    }

    Ok(ret)
}
