use derive_more::*;

use crate::{Position};
use std::error::Error;

/// SGF parsing, or traversal, related errors
#[derive(Debug, Display)]
#[display(fmt = "{}", kind)]
pub struct BadukError {
    pub kind: BadukErrorKind,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

/// Describes what kind of error we're dealing with
#[derive(Debug, Display, Eq, PartialEq)]
pub enum BadukErrorKind {
    #[display(fmt = "Invalid position")]
    InvalidPosition(Position),
    #[display(fmt = "Position already occupied")]
    AlreadyOccupied(Position),
    #[display(fmt = "Invalid input size")]
    InvalidInputSize,
    #[display(fmt = "Suicidal move")]
    SuicidalMove,
    #[display(fmt = "Illegal retaking of ko")]
    RetakingKo,
    #[display(fmt = "No go board defined")]
    MissingGoBoard,
    #[display(fmt = "Invalid root node")]
    InvalidRootNode,
    #[display(fmt = "Invalid input")]
    InvalidInput,
}

impl Error for BadukError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|boxed| boxed.as_ref() as &(dyn Error + 'static))
    }
}

impl From<BadukErrorKind> for BadukError {
    fn from(kind: BadukErrorKind) -> BadukError {
        BadukError { kind, source: None }
    }
}

impl BadukError {
    pub fn invalid_position(pos: Position, err: impl Error + Send + Sync + 'static) -> Self {
        BadukError {
            kind: BadukErrorKind::InvalidPosition(pos),
            source: Some(Box::new(err)),
        }
    }

    pub fn already_occupied(pos: Position, err: impl Error + Send + Sync + 'static) -> Self {
        BadukError {
            kind: BadukErrorKind::AlreadyOccupied(pos),
            source: Some(Box::new(err)),
        }
    }

    pub fn invalid_input(err: impl Error + Send + Sync + 'static) -> Self {
        BadukError {
            kind: BadukErrorKind::InvalidInput,
            source: Some(Box::new(err)),
        }
    }

    pub fn invalid_root_node(err: impl Error + Send + Sync + 'static) -> Self {
        BadukError {
            kind: BadukErrorKind::InvalidRootNode,
            source: Some(Box::new(err)),
        }
    }

    pub fn missing_go_board(err: impl Error + Send + Sync + 'static) -> Self {
        BadukError {
            kind: BadukErrorKind::MissingGoBoard,
            source: Some(Box::new(err)),
        }
    }

    pub fn retaking_ko(err: impl Error + Send + Sync + 'static) -> Self {
        BadukError {
            kind: BadukErrorKind::RetakingKo,
            source: Some(Box::new(err)),
        }
    }

    pub fn suicidal_move(err: impl Error + Send + Sync + 'static) -> Self {
        BadukError {
            kind: BadukErrorKind::SuicidalMove,
            source: Some(Box::new(err)),
        }
    }

    pub fn invalid_input_size(err: impl Error + Send + Sync + 'static) -> Self {
        BadukError {
            kind: BadukErrorKind::InvalidInputSize,
            source: Some(Box::new(err)),
        }
    }
}
