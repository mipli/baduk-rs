use std::fmt;

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct Position(usize, usize);

impl Position {
    pub fn x(&self) -> usize {
        self.0
    }

    pub fn y(&self) -> usize {
        self.1
    }
}

impl From<(usize, usize)> for Position {
    fn from(pos: (usize, usize)) -> Position {
        Position(pos.0, pos.1)
    }
}

impl From<(u8, u8)> for Position {
    fn from(pos: (u8, u8)) -> Position {
        Position(pos.0 as usize, pos.1 as usize)
    }
}

impl From<(i32, i32)> for Position {
    fn from(pos: (i32, i32)) -> Position {
        Position(pos.0 as usize, pos.1 as usize)
    }
}

impl From<&str> for Position {
    fn from(pos: &str) -> Position {
        let (letter, number) = pos.split_at(1);
        let mut x = letter.to_lowercase().as_bytes()[0] - 96;
        // The letter 'I' is skipped
        if x >= 9 {
            x -= 1;
        }
        let y = number.parse().unwrap();
        Position(x as usize, y)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.x(), self.y())
    }
}
