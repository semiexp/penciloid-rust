use std::io::BufRead;

use super::Clue;
use common::{Grid, P};
use io::{next_valid_line, ReadError};

pub fn read_penciloid_problem<T: BufRead>(reader: &mut T) -> Result<Grid<Clue>, ReadError> {
    let mut buffer = String::new();

    let height;
    let width;
    let n_clue_cells;

    {
        next_valid_line(reader, &mut buffer)?;
        let mut header = buffer.split(' ');
        height = header
            .next()
            .ok_or(ReadError::InvalidFormat)?
            .trim()
            .parse::<i32>()
            .map_err(|_| ReadError::InvalidValue)?;
        width = header
            .next()
            .ok_or(ReadError::InvalidFormat)?
            .trim()
            .parse::<i32>()
            .map_err(|_| ReadError::InvalidValue)?;
        n_clue_cells = header
            .next()
            .ok_or(ReadError::InvalidFormat)?
            .trim()
            .parse::<i32>()
            .map_err(|_| ReadError::InvalidValue)?;
    }

    let mut ret = Grid::new(height, width, Clue::NoClue);

    for _ in 0..n_clue_cells {
        next_valid_line(reader, &mut buffer)?;
        let mut row = buffer.split(' ');
        let y = row
            .next()
            .ok_or(ReadError::InvalidFormat)?
            .trim()
            .parse::<i32>()
            .map_err(|_| ReadError::InvalidValue)?;
        let x = row
            .next()
            .ok_or(ReadError::InvalidFormat)?
            .trim()
            .parse::<i32>()
            .map_err(|_| ReadError::InvalidValue)?;
        let clue_horizontal = row
            .next()
            .ok_or(ReadError::InvalidFormat)?
            .trim()
            .parse::<i32>()
            .map_err(|_| ReadError::InvalidValue)?;
        let clue_vertical = row
            .next()
            .ok_or(ReadError::InvalidFormat)?
            .trim()
            .parse::<i32>()
            .map_err(|_| ReadError::InvalidValue)?;

        ret[P(y, x)] = Clue::Clue {
            horizontal: clue_horizontal,
            vertical: clue_vertical,
        };
    }

    Ok(ret)
}
