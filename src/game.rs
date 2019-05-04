use crate::{GameState, BadukError, BadukErrorKind, Position, Color, SgfToken};
use sgf_parser::{GameTree as SgfTree, parse};
use std::convert::TryFrom;

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
    pub fn new(width: u32, height: u32) -> GameTree {
        let mut root = GameTreeNode::new();
        root.tokens.push(SgfToken::Size(width, height));
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

    pub fn get_node(&self, node: GameTreeIndex) -> Option<&GameTreeNode> {
        self.nodes.get(node)
    }

    pub fn current_state(&self) -> Option<&GameState> {
        self.nodes[self.current].state.as_ref()
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

    pub fn create_new_node(&mut self, parent: GameTreeIndex) -> GameTreeIndex {
        let new_id = self.nodes.len();
        let new_node = GameTreeNode {
            parent: Some(parent),
            state: self.nodes[parent].state.clone(),
            tokens: vec![],
            children: vec![],
        };
        self.nodes[parent].children.push(new_id);
        self.nodes.push(new_node);
        self.current = new_id;
        new_id
    }

    pub fn play_move(&mut self, pos: impl Into<Position>, color: Color) -> Result<GameTreeIndex, BadukError> {
        self.play_move_as_variation(pos, color, self.current)
    }

    pub fn play_move_as_variation(&mut self, pos: impl Into<Position>, color: Color, parent: GameTreeIndex) -> Result<GameTreeIndex, BadukError> {
        let pos = pos.into();
        let current_node = &self.nodes[parent];
        match current_node.state {
            None => Err(BadukErrorKind::MissingGoBoard.into()),
            Some(ref current_state) => {
                let mut state = current_state.place_stone(pos, color)?;
                let removed = state.remove_dead_stones(!color);
                let valid = state.is_valid();
                if !valid {
                    Err(BadukErrorKind::SuicidalMove.into())
                } else {
                    if self.is_directly_retaking_ko(color, &state) {
                        return Err(BadukErrorKind::RetakingKo.into());
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

    fn play_move_on_node(&mut self, pos: impl Into<Position>, color: Color, node: GameTreeIndex) -> Result<GameTreeIndex, BadukError> {
        let pos = pos.into();
        match self.nodes[node].state {
            None => Err(BadukErrorKind::MissingGoBoard.into()),
            Some(ref current_state) => {
                let mut state = current_state.place_stone(pos, color)?;
                let removed = state.remove_dead_stones(!color);
                let valid = state.is_valid();
                if !valid {
                    Err(BadukErrorKind::SuicidalMove.into())
                } else {
                    if self.is_directly_retaking_ko(color, &state) {
                        return Err(BadukErrorKind::RetakingKo.into());
                    }
                    state.capture_stones(removed.len() as i32, !color);
                    self.nodes[node].tokens.push(SgfToken::Move {
                        coordinate: pos.into(),
                        color,
                    });
                    self.nodes[node].state = Some(state);
                    Ok(node)
                }
            }
        }
    }

    pub fn add_stone(&mut self, pos: impl Into<Position>, color: Color) -> Result<GameTreeIndex, BadukError> {
        self.add_stone_on_node(pos, color, self.current)
    }

    pub fn add_stone_on_node(&mut self, pos: impl Into<Position>, color: Color, node: GameTreeIndex) -> Result<GameTreeIndex, BadukError> {
        let pos = pos.into();
        let current_node = &self.nodes[node];

        match current_node.state {
            None => Err(BadukErrorKind::MissingGoBoard.into()),
            Some(ref current_state) => {
                let state = current_state.add_stone(pos, color)?;
                self.nodes[node].tokens.push(
                    SgfToken::Add {
                        coordinate: pos.into(),
                        color,
                    });
                self.nodes[node].state = Some(state);
                Ok(self.current)
            }
        }
    }

    fn is_directly_retaking_ko(&self, color: Color, new_state: &GameState) -> bool {
        if let Some(parent_id) = self.nodes[self.current].parent {
            if let Some(ref parent_state) = self.nodes[parent_id].state {
                if let Ok(diff) = parent_state.difference(new_state) {
                    return match color {
                        Color::Black => diff.captures.white == 0 && diff.captures.black == 1,
                        Color::White => diff.captures.black == 0 && diff.captures.white == 1,
                    }
                }
            }
        }
        false
    }

    pub fn add_token(&mut self, node: GameTreeIndex, token: &SgfToken) -> GameTreeIndex {
        self.nodes[node].tokens.push(token.clone());
        node
    }

    pub fn set_size(&mut self, width: u32, height: u32, node: GameTreeIndex) -> Result<GameTreeIndex, BadukError> {
        if node != self.root {
            Err(BadukErrorKind::InvalidRootNode.into())
        } else {
            self.nodes[node].tokens.push(SgfToken::Size(width, height));
            self.nodes[node].state = Some(GameState::new(width, height));
            Ok(node)
        }
    }

    pub fn parse_sgf_token(&mut self, token: &SgfToken, node: GameTreeIndex) -> Result<GameTreeIndex, BadukError> {
        let index = match token {
            SgfToken::Move{color, coordinate} => {
                self.play_move_on_node(*coordinate, *color, node)?
            },
            SgfToken::Add{color, coordinate} => {
                self.add_stone_on_node(*coordinate, *color, node)?
            },
            SgfToken::Size(width, height) => {
                let _ = self.set_size(*width, *height, node)?;
                node
            },
            _ => {
                self.add_token(node, token);
                node
            }
        };
        Ok(index)
    }
}

impl From<&SgfTree> for GameTree {
    fn from(tree: &SgfTree) -> GameTree {
        let mut game = GameTree::default();
        parse_variation(&mut game, tree, None);
        game
    }
}

impl TryFrom<&str> for GameTree {
    type Error = BadukError;

    fn try_from(input: &str) -> Result<GameTree, BadukError> {
        let tree: SgfTree = parse(input).map_err(BadukError::invalid_root_node)?;
        Ok((&tree).into())
    }
}

fn parse_variation(game: &mut GameTree, tree: &SgfTree, mut current: Option<GameTreeIndex>) {
    tree.nodes.iter().for_each(|node| {
        current = match current {
            None => Some(game.current),
            Some(node) => Some(game.create_new_node(node))
        };
        node.tokens.iter().for_each(|token| {
            if let Err(e) = game.parse_sgf_token(token, current.expect("previous statement guarentees no-None value")) {
                println!("Error parsing sgf token: {:?}, {:?}", token, e);
            }
        });
    });
    tree.variations.iter().for_each(|variation| {
        parse_variation(game, &variation, current);
    });
}

impl From<GameState> for GameTree {
    fn from(state: GameState) -> GameTree {
        let mut game = GameTree::new(state.width, state.height);
        for x in 1..=state.width {
            for y in 1..=state.height {
                if let Some(color) = state.get_stone((x, y)) {
                    let _ = game.add_stone((x, y), *color);
                }
            }
        }
        game
    }
}
