use crate::{GameState, Captures, Error, Position, Color, SgfToken};
use sgf_parser::{GameTree as SgfTree};

pub type GameTreeIndex = usize;

#[derive(Debug, Clone)]
pub struct GameTreeNode {
    pub parent: Option<GameTreeIndex>,
    pub state: Option<GameState>,
    pub tokens: Vec<SgfToken>,
    pub children: Vec<GameTreeIndex>,
    // active: bool -> used to indicate active branch
}

impl GameTreeNode {
    fn new() -> GameTreeNode {
        GameTreeNode {
            parent: None,
            state: None,
            tokens: vec![],
            children: vec![],
        }
    }
}

#[derive(Debug)]
pub struct GameTree {
    pub root: GameTreeIndex,
    pub nodes: Vec<GameTreeNode>,
    pub current: GameTreeIndex,
}

impl Default for GameTree {
    fn default() -> GameTree {
        let root = GameTreeNode::new();
        GameTree {
            root: 0,
            current: 0,
            nodes: vec![root]
        }
    }
}

impl GameTree {
    pub fn new(width: usize, height: usize) -> GameTree {
        let mut root = GameTreeNode::new();
        root.state = Some(GameState::new(width, height));
        GameTree {
            root: 0,
            current: 0,
            nodes: vec![root]
        }
    }

    pub fn count_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn current_state(&self) -> Option<&GameState> {
        self.nodes[self.current].state.as_ref()
    }

    pub fn create_board(&mut self, size: usize) {
        self.nodes[self.current].state = Some(GameState::new(size, size));
    }

    fn add_node(&mut self, parent: GameTreeIndex, tokens: Vec<SgfToken>, state: GameState) -> GameTreeIndex {
        let new_id = self.nodes.len();
        let new_node = GameTreeNode {
            parent: Some(parent),
            state: Some(state),
            tokens,
            children: vec![],
        };
        self.nodes[parent].children.push(new_id);
        self.nodes.push(new_node);
        self.current = new_id;
        new_id
    }

    pub fn play_move(&mut self, pos: impl Into<Position>, color: Color) -> Result<GameTreeIndex, Error> {
        self.play_move_as_variation(pos, color, self.current)
    }

    pub fn play_move_as_variation(&mut self, pos: impl Into<Position>, color: Color, parent: GameTreeIndex) -> Result<GameTreeIndex, Error> {
        let pos = pos.into();
        let current_node = &self.nodes[parent];
        match current_node.state {
            None => Err(Error::MissingGoboard),
            Some(ref current_state) => {
                let mut state = current_state.place_stone(pos, color)?;
                let removed = state.remove_dead_stones(!color);
                let valid = state.is_valid();
                if !valid {
                    Err(Error::SuicidalMove)
                } else {
                    if self.is_directly_retaking_ko(color, &state) {
                        return Err(Error::RetakingKo);
                    }
                    state.capture_stones(removed.len() as i32, !color);
                    let tokens = vec![SgfToken::Move {
                        coordinate: pos.into(),
                        color,
                    }];
                    Ok(self.add_node(parent, tokens, state))
                }
            }
        }
    }

    pub fn add_stone(&mut self, pos: impl Into<Position>, color: Color) -> Result<GameTreeIndex, Error> {
        let pos = pos.into();
        let current_node = &self.nodes[self.current];

        match current_node.state {
            None => Err(Error::MissingGoboard),
            Some(ref current_state) => {
                let state = current_state.add_stone(pos, color)?;
                let tokens = vec![SgfToken::Add {
                    coordinate: pos.into(),
                    color,
                }];
                Ok(self.add_node(self.current, tokens, state))
            }
        }
    }

    fn is_directly_retaking_ko(&self, color: Color, new_state: &GameState) -> bool {
        if let Some(parent_id) = self.nodes[self.current].parent {
            if let Some(ref parent_state) = self.nodes[parent_id].state {
                if let Ok(diff) = parent_state.difference(new_state) {
                    if diff.positions.len() != 0 {
                        false
                    } else {
                        match color {
                            Color::Black => diff.captures.white == 0 && diff.captures.black == 1,
                            Color::White => diff.captures.black == 0 && diff.captures.white == 1,
                        }
                    }
                } else  {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn parse_sgf_token(&mut self, token: &SgfToken, node: GameTreeIndex) -> Result<GameTreeIndex, Error> {
        let index = match token {
            SgfToken::Move{color, coordinate} => {
                self.play_move_as_variation(*coordinate, *color, node)?
            },
            SgfToken::Add{color, coordinate} => {
                self.add_stone(*coordinate, *color)?
            },
            _ => node
        };
        Ok(index)
    }
}

impl From<&SgfTree> for GameTree {
    fn from(tree: &SgfTree) -> GameTree {
        let mut game = GameTree::new(19, 19);
        let current = game.current;
        parse_variation(&mut game, tree, current);
        game
    }
}

fn parse_variation(game: &mut GameTree, tree: &SgfTree, mut current: GameTreeIndex) {
    tree.nodes.iter().for_each(|node| {
        node.tokens.iter().for_each(|token| {
            match game.parse_sgf_token(token, current) {
                Err(e) => println!("Error parsing sgf token: {:?}, {:?}", token, e),
                Ok(i) => current = i
            }
        });
    });
    tree.variations.iter().for_each(|variation| {
        parse_variation(game, &variation, current);
    });
}
