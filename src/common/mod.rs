use std::ops::{Add, Sub, Mul};

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

#[cfg(test)]
mod tests_common {
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
}
