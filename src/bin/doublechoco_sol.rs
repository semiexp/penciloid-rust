extern crate puzrs;

use puzrs::doublechoco::*;
use puzrs::*;
use std::env;

fn parse_url(url: &str) -> (Grid<Color>, Grid<Clue>) {
    let tokens = url.split("/").collect::<Vec<_>>();
    let width = tokens[tokens.len() - 3].parse::<i32>().unwrap();
    let height = tokens[tokens.len() - 2].parse::<i32>().unwrap();
    let body = tokens[tokens.len() - 1].chars().collect::<Vec<char>>();

    let mut color = Grid::new(height, width, Color::White);
    let mut clue = Grid::new(height, width, NO_CLUE);

    let mut idx = 0usize;
    for i in 0..((height * width + 4) / 5) {
        let v = body[idx];
        idx += 1;
        let bits = if '0' <= v && v <= '9' {
            (v as i32) - ('0' as i32)
        } else {
            (v as i32) - ('a' as i32) + 10
        };
        for j in 0..5 {
            let p = i * 5 + j;
            let y = p / width;
            let x = p % width;
            if y < height {
                color[P(y, x)] = if (bits & (1 << (4 - j))) != 0 {
                    Color::Black
                } else {
                    Color::White
                };
            }
        }
    }
    fn convert_hex(v: char) -> i32 {
        if '0' <= v && v <= '9' {
            (v as i32) - ('0' as i32)
        } else {
            (v as i32) - ('a' as i32) + 10
        }
    }
    let mut pos = 0;
    while idx < body.len() {
        if 'g' <= body[idx] {
            pos += (body[idx] as i32) - ('f' as i32);
            idx += 1;
        } else {
            let val;
            if body[idx] == '-' {
                val = convert_hex(body[idx + 1]) * 16 + convert_hex(body[idx + 2]);
                idx += 3;
            } else {
                val = convert_hex(body[idx]);
                idx += 1;
            }
            clue[P(pos / width, pos % width)] = val;
            pos += 1;
        }
    }
    (color, clue)
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let url = &(args[1]);

    let (color, clue) = parse_url(url);
    let height = color.height();
    let width = color.width();

    let mut field = Field::new(&color, &clue);
    field.trial_and_error(2);

    assert_eq!(field.inconsistent(), false);
    for y in 0..(height * 2 + 1) {
        for x in 0..(width * 2 + 1) {
            match (y % 2, x % 2) {
                (0, 0) => print!("+"),
                (0, 1) => {
                    if y == 0 || y == height * 2 {
                        print!("-");
                    } else {
                        match field.border(LP(y - 1, x - 1)) {
                            Border::Undecided => print!(" "),
                            Border::Line => print!("-"),
                            Border::Blank => print!("x"),
                        }
                    }
                }
                (1, 0) => {
                    if x == 0 || x == width * 2 {
                        print!("|");
                    } else {
                        match field.border(LP(y - 1, x - 1)) {
                            Border::Undecided => print!(" "),
                            Border::Line => print!("|"),
                            Border::Blank => print!("x"),
                        }
                    }
                }
                (1, 1) => print!(" "),
                _ => unreachable!(),
            }
        }
        println!();
    }
}
