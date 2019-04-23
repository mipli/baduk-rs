use crate::{Goban, Captures, Error, Position, Color, SgfToken};
use sgf_parser::{GameTree as SgfTree};

pub type GameTreeIndex = usize;

#[derive(Debug, Clone)]
pub struct GameTreeNode {
    pub parent: Option<GameTreeIndex>,
    pub state: Option<Goban>,
    pub tokens: Vec<SgfToken>,
    pub children: Vec<GameTreeIndex>,
    pub performed_move: Option<PerformedMove>,
    // active: bool -> used to indicate active branch
}

#[derive(Debug, Clone)]
pub struct PerformedMove {
    pub position: Position,
    pub color: Color,
    pub captures: Vec<Position>
}

impl GameTreeNode {
    fn new() -> GameTreeNode {
        GameTreeNode {
            parent: None,
            state: None,
            tokens: vec![],
            children: vec![],
            performed_move: None,
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
        root.state = Some(Goban::new(width, height));
        GameTree {
            root: 0,
            current: 0,
            nodes: vec![root]
        }
    }

    pub fn count_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn current_state(&self) -> Option<&Goban> {
        self.nodes[self.current].state.as_ref()
    }

    pub fn create_board(&mut self, size: usize) {
        self.nodes[self.current].state = Some(Goban::new(size, size));
    }

    fn add_move(&mut self, performed_move: PerformedMove, parent: GameTreeIndex, state: Goban) -> GameTreeIndex {
        let new_id = self.nodes.len();
        let new_node = GameTreeNode {
            parent: Some(parent),
            state: Some(state),
            tokens: vec![SgfToken::Move {
                color: performed_move.color,
                coordinate: (performed_move.position.x() as u8, performed_move.position.y() as u8)
            }],
            children: vec![],
            performed_move: Some(performed_move),
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

        if self.is_directly_retaking_ko(pos, color) {
            return Err(Error::RetakingKo);
        }
        match current_node.state {
            None => Err(Error::MissingGoboard),
            Some(ref current_state) => {
                let mut state = current_state.place_stone(pos, color)?;
                let removed = state.remove_dead_stones(!color);
                let valid = state.is_valid();
                if !valid {
                    Err(Error::SuicidalMove)
                } else {
                    state.capture_stones(removed.len(), !color);
                    let performed_move = PerformedMove {
                        position: pos,
                        color: color,
                        captures: removed,
                    };
                    Ok(self.add_move(performed_move, parent, state))
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
                let performed_move = PerformedMove {
                    position: pos,
                    color: color,
                    captures: vec![],
                };
                Ok(self.add_move(performed_move, self.current, state))
            }
        }
    }

    fn is_directly_retaking_ko(&self, pos: Position, color: Color) -> bool {
        match self.nodes[self.current].performed_move {
            Some(ref performed_move) => {
                performed_move.color == !color && performed_move.captures.len() == 1 && performed_move.captures[0] == pos
            },
            None => {
               false
            }
        }
    }

    pub fn consume_sgf_token(&mut self, token: &SgfToken, node: GameTreeIndex) -> Result<GameTreeIndex, Error> {
        let index = match token {
            SgfToken::Move{color, coordinate} => {
                self.play_move_as_variation(*coordinate, *color, node)?
            },
            SgfToken::Add{color, coordinate} => {
                self.add_stone(*coordinate, *color)?
            },
            _ => node
        };
        self.nodes[node].tokens.push(token.clone());
        Ok(index)
    }
}

impl From<&SgfTree> for GameTree {
    fn from(tree: &SgfTree) -> GameTree {
        let mut game = GameTree::new(19, 19);
        let current = game.current;
        play_variation(&mut game, tree, current);
        game
    }
}

fn play_variation(game: &mut GameTree, tree: &SgfTree, mut current: GameTreeIndex) {
    tree.nodes.iter().for_each(|node| {
        node.tokens.iter().for_each(|token| {
            match game.consume_sgf_token(token, current) {
                Err(e) => println!("Error parsing sgf token: {:?}, {:?}", token, e),
                Ok(i) => current = i
            }
        });
    });
    tree.variations.iter().for_each(|variation| {
        play_variation(game, &variation, current);
    });
}
