use std::error;
use std::fmt;

use crate::Position;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidPosition(Position),
    AlreadyOccupied(Position),
    InvalidInputSize,
    SuicidalMove,
    RetakingKo
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        use std::error::Error;

        match *self {
            InvalidPosition(pos) => write!(f, "{}: ({})", self.description(), pos),
            AlreadyOccupied(pos) => write!(f, "{}: ({})", self.description(), pos),
            InvalidInputSize => write!(f, "{}", self.description()),
            SuicidalMove => write!(f, "{}", self.description()),
            RetakingKo => write!(f, "{}", self.description())
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;

        match *self {
            InvalidPosition(_) => "Cannot place stone at position",
            AlreadyOccupied(_) => "Position was already occupied",
            InvalidInputSize => "Input was not square size",
            SuicidalMove => "Suicide is not allowed",
            RetakingKo => "Cannot retake ko at once"
        }
    }
}
