// https://www.red-bean.com/sgf/

use crate::{Game};

pub struct SgfParser {};

enum Token {
    EV, // Event
    GM, // Game Type, 1 == Go
    US, // Game/Program provider
    CP, // Copyright
    GN, // Game Name
    PW, // Player White
    WR, // White Rank
    PB, // Player Black,
    BR, // Black Rank
    PC, // Place for game
    DT, // Game Date
    SZ, // Size
    TM, // Time limit
    KM, // Komi
    C,  // Comment
    B,  // Black Move
    BL, // Black time left
    W,  // White Move
    WL  // White time left
}

impl SgfParser {
    pub parse(sgf: &str) -> Option<Game> {
        None
    }
}
