use std::ops::{Add, Sub, Mul, Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    pub y: i32,
    pub x: i32
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopCoord {
    pub y: i32,
    pub x: i32
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dir {
    pub y: i32,
    pub x: i32
}
impl Add<Dir> for Coord {
    type Output = Coord;
    fn add(self, rhs: Dir) -> Coord {
        Coord {
            y: self.y + rhs.y,
            x: self.x + rhs.x
        }
    }
}
impl Sub<Dir> for Coord {
    type Output = Coord;
    fn sub(self, rhs: Dir) -> Coord {
        Coord {
            y: self.y - rhs.y,
            x: self.x - rhs.x
        }
    }
}
impl Add<Dir> for LoopCoord {
    type Output = LoopCoord;
    fn add(self, rhs: Dir) -> LoopCoord {
        LoopCoord {
            y: self.y + rhs.y,
            x: self.x + rhs.x
        }
    }
}
impl Sub<Dir> for LoopCoord {
    type Output = LoopCoord;
    fn sub(self, rhs: Dir) -> LoopCoord {
        LoopCoord {
            y: self.y - rhs.y,
            x: self.x - rhs.x
        }
    }
}
impl Add<Dir> for Dir {
    type Output = Dir;
    fn add(self, rhs: Dir) -> Dir {
        Dir {
            y: self.y + rhs.y,
            x: self.x + rhs.x
        }
    }
}
impl Sub<Dir> for Dir {
    type Output = Dir;
    fn sub(self, rhs: Dir) -> Dir {
        Dir {
            y: self.y - rhs.y,
            x: self.x - rhs.x
        }
    }
}
impl Mul<i32> for Dir {
    type Output = Dir;
    fn mul(self, rhs: i32) -> Dir {
        Dir {
            y: self.y * rhs,
            x: self.x * rhs
        }
    }
}

#[derive(Debug, Clone)]
pub struct Grid<T: Clone> {
    height: i32,
    width: i32,
    data: Vec<T>
}
impl<T: Clone> Grid<T> {
    pub fn new(height: i32, width: i32, default: T) -> Grid<T> {
        Grid {
            height: height,
            width: width,
            data: vec![default; (height * width) as usize]
        }
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    fn index(&self, cd: Coord) -> usize {
        (cd.y * self.width + cd.x) as usize
    }
    fn index_loop(&self, cd: LoopCoord) -> usize {
        (cd.y * self.width + cd.x) as usize
    }
}
impl<T: Clone> Index<Coord> for Grid<T> {
    type Output = T;
    fn index<'a>(&'a self, idx: Coord) -> &'a T {
        let idx = self.index(idx);
        &self.data[idx]
    }
}
impl<T: Clone> IndexMut<Coord> for Grid<T> {
    fn index_mut<'a>(&'a mut self, idx: Coord) -> &'a mut T {
        let idx = self.index(idx);
        &mut self.data[idx]
    }
}
impl<T: Clone> Index<LoopCoord> for Grid<T> {
    type Output = T;
    fn index<'a>(&'a self, idx: LoopCoord) -> &'a T {
        let idx = self.index_loop(idx);
        &self.data[idx]
    }
}
impl<T: Clone> IndexMut<LoopCoord> for Grid<T> {
    fn index_mut<'a>(&'a mut self, idx: LoopCoord) -> &'a mut T {
        let idx = self.index_loop(idx);
        &mut self.data[idx]
    }
}
impl<T: Clone> Index<usize> for Grid<T> {
    type Output = T;
    fn index<'a>(&'a self, idx: usize) -> &'a T {
        &self.data[idx]
    }
}
impl<T: Clone> IndexMut<usize> for Grid<T> {
    fn index_mut<'a>(&'a mut self, idx: usize) -> &'a mut T {
        &mut self.data[idx]
    }
}
#[cfg(test)]
pub fn vec_to_grid<T>(v: &Vec<Vec<T>>) -> Grid<T> where T: Copy {
    if v.len() == 0 {
        panic!("Attempted to convert empty Vec to Grid");
    }
    let ref_len = v[0].len();
    for r in v {
        if r.len() != ref_len {
            panic!("Each element in v must contain the same number of elements");
        }
    }
    let mut flat = vec![];
    for r in v {
        for &x in r {
            flat.push(x);
        }
    }
    Grid {
        height: v.len() as i32,
        width: ref_len as i32,
        data: flat
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_types_operators() {
        assert_eq!(Coord { y: 1, x: 2 } + Dir { y: 3, x: 5 }, Coord { y: 4, x: 7 });
        assert_eq!(Coord { y: 1, x: 2 } - Dir { y: 3, x: 5 }, Coord { y: -2, x: -3 });
        assert_eq!(LoopCoord { y: 1, x: 2 } + Dir { y: 3, x: 5 }, LoopCoord { y: 4, x: 7 });
        assert_eq!(LoopCoord { y: 1, x: 2 } - Dir { y: 3, x: 5 }, LoopCoord { y: -2, x: -3 });
        assert_eq!(Dir { y: 1, x: 2 } + Dir { y: 3, x: 5 }, Dir { y: 4, x: 7 });
        assert_eq!(Dir { y: 1, x: 2 } - Dir { y: 3, x: 5 }, Dir { y: -2, x: -3 });
        assert_eq!(Dir { y: 1, x: 2 } * 3, Dir { y: 3, x: 6 });
    }

    #[test]
    fn test_grid() {
        let mut grid = Grid::new(3, 3, 0);
        assert_eq!(grid.height(), 3);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid[LoopCoord { y: 1, x: 1 }], 0);
        grid[LoopCoord { y: 1, x: 1 }] = 4;
        assert_eq!(grid[LoopCoord { y: 1, x: 1 }], 4);
        assert_eq!(grid[LoopCoord { y: 1, x: 0 }], 0);
        assert_eq!(grid[LoopCoord { y: 2, x: 0 }], 0);
        assert_eq!(grid[4], 4);
    }
}
