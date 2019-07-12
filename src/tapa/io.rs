use std::io::BufRead;

use super::*;
use common::{Grid, X, Y};
use io::{read_grid, ReadError};

pub fn read_penciloid_problem<T: BufRead>(reader: &mut T) -> Result<Grid<Clue>, ReadError> {
    read_grid(
        reader,
        |token: &str| {
            if token == "." {
                Ok(NO_CLUE)
            } else {
                let clue_pattern = token
                    .chars()
                    .collect::<Vec<char>>()
                    .into_iter()
                    .filter(|&c| '0' <= c && c <= '9')
                    .map(|c| (c as u8 - '0' as u8) as i32)
                    .collect::<Vec<i32>>();

                if clue_pattern.len() == 0 {
                    Ok(NO_CLUE)
                } else {
                    match clue_pattern_to_id(&clue_pattern) {
                        Some(c) => Ok(c),
                        None => Err(ReadError::InvalidValue),
                    }
                }
            }
        },
        NO_CLUE,
    )
}
