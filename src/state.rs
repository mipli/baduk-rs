use crate::{BadukError, BadukErrorKind, Position, Color};
use std::collections::HashSet;

type Intersection = Option<Color>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Captures {
    pub white: i32,
    pub black: i32
}

impl Default for Captures {
    fn default() -> Captures {
        Captures {
            white: 0,
            black: 0
        }
    }
}

impl Captures {
    fn capture_stones(&mut self, count: i32, color: Color) {
        match color {
            Color::Black => self.black += count,
            Color::White => self.white += count
        }
    }
}

#[derive(Debug)]
pub struct GameStateDifference {
    pub positions: Vec<Position>,
    pub captures: Captures
}

impl GameStateDifference {
    pub fn is_empty(&self) -> bool {
        self.positions.len() == 0 && self.captures.white == 0 && self.captures.black == 0
    }
}

#[derive(Clone)]
pub struct GameState {
    pub board: Vec<Intersection>,
    pub captures: Captures,
    pub width: u32,
    pub height: u32,
}

impl GameState {
    pub fn new(width: u32, height: u32) -> GameState {
        GameState {
            board: vec![None; (width * height) as usize],
            captures: Captures::default(),
            width,
            height,
        }
    }

    pub fn difference(&self, other: &GameState) -> Result<GameStateDifference, ()> {
        if self.width != other.width || self.height != other.height {
            return Err(());
        }
        let mut diff = vec![];
        for x in 1..=self.width {
            for y in 1..=self.height {
                if self.get_stone((x, y)) != other.get_stone((x, y)) {
                    diff.push((x, y).into());
                }
            }
        }
        Ok(GameStateDifference {
            positions: diff,
            captures: Captures {
                black: (self.captures.black - other.captures.black).abs(),
                white: (self.captures.white - other.captures.white).abs(),
            }
        })
    }

    pub fn capture_stones(&mut self, count: i32, color: Color) {
        self.captures.capture_stones(count, color);
    }

    pub fn captures(&self) -> &Captures {
        &self.captures
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn is_empty(&self) -> bool {
        self.board.iter().all(Option::is_none)
    }

    pub fn is_valid_position(&self, pos: impl Into<Position>) -> bool {
        let pos = pos.into();
        let x = pos.x();
        let y = pos.y();
        x >= 1 && x <= self.width && y >= 1 && y <= self.height
    }

    pub fn get_stone(&self, pos: impl Into<Position>) -> Option<&Color> {
        let pos: Position = pos.into();
        if !self.is_valid_position(pos) {
            return None;
        }
        let index = self.position_to_index(pos);
        self.board[index].as_ref()
    }

    pub fn place_stone(&self, pos: impl Into<Position>, color: Color) -> Result<GameState, BadukError> {
        let pos = pos.into();
        if !self.is_valid_position(pos) {
            return Err(BadukErrorKind::InvalidPosition(pos).into());
        }
        let index = self.position_to_index(pos);
        let mut state = (*self).clone();
        match state.board[index] {
            None => state.board[index] = Some(color),
            Some(_) => {
                return Err(BadukErrorKind::AlreadyOccupied(pos).into());
            }
        }
        Ok(state)
    }

    pub fn add_stone(&self, pos: impl Into<Position>, color: Color) -> Result<GameState, BadukError> {
        let pos = pos.into();
        let index = self.position_to_index(pos);
        let mut state = (*self).clone();
        state.board[index] = Some(color);
        Ok(state)
    }

    pub fn remove_stone(&mut self, pos: impl Into<Position>) -> Result<(), BadukError> {
        let pos = pos.into();
        if !self.is_valid_position(pos) {
            return Err(BadukErrorKind::InvalidPosition(pos).into());
        }
        let index = self.position_to_index(pos);
        self.board[index] = None;
        Ok(())
    }

    pub fn count_liberties(&self, pos: impl Into<Position>) -> Option<u32> {
        let pos = pos.into();
        if !self.is_valid_position(pos) {
            return None;
        }
        Some(self.get_chain(pos)?.iter().fold(0, |acc, pos| {
            acc + self.get_neighbours(*pos).iter().fold(0, |acc, pos| {
                if self.get_stone(*pos) == None {
                    acc + 1
                } else {
                    acc
                }
            })
        }))
    }

    // TODO: According to SGF spec, should only remove dead stones affected by last move
    pub fn remove_dead_stones(&mut self, color: Color) -> Vec<Position> {
        let mut dead_stones = vec![];
        for x in 1..=self.width {
            for y in 1..=self.height {
                if self.get_stone((x, y)) != Some(&color) {
                    continue;
                }
                if let Some(chain) = self.get_chain((x, y)) {
                    let liberties = chain.iter().fold(0, |acc, pos| {
                        acc + self.get_neighbours(*pos).iter().fold(0, |acc, pos| {
                            if self.get_stone(*pos) == None {
                                acc + 1
                            } else {
                                acc
                            }
                        })
                    });
                    if liberties == 0 {
                        chain.into_iter().for_each(|pos| {
                            dead_stones.push(pos);
                            let _ = self.remove_stone(pos);
                        });
                    }
                }
            }
        }
        dead_stones
    }

    pub fn is_valid(&self) -> bool {
        for x in 1..=self.width {
            for y in 1..=self.height {
                if self.count_liberties((x, y)) == Some(0) {
                    return false;
                }
            }
        }
        true
    }

    fn get_chain(&self, pos: impl Into<Position>) -> Option<Vec<Position>> {
        let pos = pos.into();
        let stone = self.get_stone(pos)?;
        let mut tried: HashSet<Position> = HashSet::default();
        let mut chain = vec![pos];
        let mut pool = vec![pos];

        tried.insert(pos);
        while let Some(pos) = pool.pop() {
            self.get_neighbours(pos).iter().for_each(|n| {
                match tried.get(n) {
                    None if self.get_stone(*n) == Some(stone) => {
                        chain.push(*n);
                        pool.push(*n);
                    }
                    _ => {}
                }
                tried.insert(*n);
            });
        }
        Some(chain)
    }

    #[inline(always)]
    fn position_to_index(&self, pos: impl Into<Position>) -> usize {
        let pos = pos.into();
        ((pos.x() - 1) + ((pos.y() - 1) * self.width)) as usize
    }

    fn get_neighbours(&self, pos: impl Into<Position>) -> Vec<Position> {
        let pos = pos.into();
        let mut neighbours = vec![];
        if pos.x() > 1 {
            neighbours.push((pos.x() - 1, pos.y()).into());
        }
        if pos.x() < self.width {
            neighbours.push((pos.x() + 1, pos.y()).into());
        }
        if pos.y() > 1 {
            neighbours.push((pos.x(), pos.y() - 1).into());
        }
        if pos.y() < self.height {
            neighbours.push((pos.x(), pos.y() + 1).into());
        }
        neighbours
    }
}

impl Default for GameState {
    fn default() -> GameState {
        GameState {
            board: vec![None; 361],
            captures: Captures::default(),
            width: 19,
            height: 19,
        }
    }
}

impl std::str::FromStr for GameState {
    type Err = BadukError;

    fn from_str(data: &str) -> Result<GameState, BadukError> {
        let board = data.chars().fold(vec![], |mut board, c| {
            match c {
                '.' => board.push(None),
                'x' => board.push(Some(Color::Black)),
                'o' => board.push(Some(Color::White)),
                _ => {}
            }
            board
        });
        let size = (board.len() as f64).sqrt() as u32;
        if size * size != (board.len() as u32) {
            Err(BadukErrorKind::InvalidInputSize.into())
        } else {
            Ok(GameState {
                board,
                captures: Captures::default(),
                width: size,
                height: size,
            })
        }
    }
}

impl std::fmt::Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let out =
            self.board
                .iter()
                .enumerate()
                .fold(String::new(), |mut out, (idx, intersection)| {
                    if idx > 0 && (idx as u32) % self.width == 0 {
                        out.push_str("\n");
                    }
                    let sym = match intersection {
                        None => ".".to_string(),
                        Some(Color::White) => "o".to_string(),
                        Some(Color::Black) => "x".to_string(),
                    };
                    out.push_str(&sym);
                    out
                });
        write!(f, "{}", out)
    }
}
