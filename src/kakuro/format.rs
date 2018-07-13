use std::io::BufRead;

use super::Clue;
use common::{Grid, X, Y};
use format::{next_valid_line, Error};

pub fn read_penciloid_problem<T: BufRead>(reader: &mut T) -> Result<Grid<Clue>, Error> {
    let mut buffer = String::new();

    let height;
    let width;
    let n_clue_cells;

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
        n_clue_cells = try!(
            try!(header.next().ok_or(Error::Format))
                .trim()
                .parse::<i32>()
        );
    }

    let mut ret = Grid::new(height, width, Clue::NoClue);

    for _ in 0..n_clue_cells {
        try!(next_valid_line(reader, &mut buffer));
        let mut row = buffer.split(' ');
        let y = try!(try!(row.next().ok_or(Error::Format)).trim().parse::<i32>());
        let x = try!(try!(row.next().ok_or(Error::Format)).trim().parse::<i32>());
        let clue_horizontal = try!(try!(row.next().ok_or(Error::Format)).trim().parse::<i32>());
        let clue_vertical = try!(try!(row.next().ok_or(Error::Format)).trim().parse::<i32>());

        ret[(Y(y), X(x))] = Clue::Clue {
            horizontal: clue_horizontal,
            vertical: clue_vertical,
        };
    }

    Ok(ret)
}
