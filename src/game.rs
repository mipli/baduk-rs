use crate::{Goban, Error, Position, Color, SgfToken};
use sgf_parser::{GameTree as SgfTree};

type GameTreeIndex = usize;

struct GameTree {
    root: GameTreeNode,
    nodes: Vec<GameTreeNode>,
    active: GameTreeIndex,
}

struct GameTreeNode {
    parent: GameTreeIndex,
    nodeId: GameTreeIndex,
    goban: Goban,
    tokens: Vec<SgfToken>,
    children: Vec<GameTreeIndex>
}

#[derive(Debug)]
struct Move {
    position: Position,
    color: Color,
    captures: Vec<Position>
}

#[derive(Debug)]
enum Node {
    Move(Move),
    Empty
}

#[derive(Debug)]
pub struct Game {
    current: usize,
    selected_variation: usize,
    komi: f32,
    nodes: Vec<(Node, Goban)>,
    variations: Vec<Game>
}

impl Default for Game {
    fn default() -> Game {
        Game {
            current: 0,
            selected_variation: 0,
            komi: 6.5,
            nodes: vec![(Node::Empty, Goban::default())],
            variations: vec![]
        }
    }
}

impl Game {
    pub fn count_moves(&self) -> usize {
        let count = self.nodes
            .iter()
            .filter(|(node, _)| {
                match node {
                    Node::Move(_) => true,
                    _ => false
                }
            })
            .count();
        let variation_count = self
            .variations
            .iter()
            .map(|v| v.count_moves())
            .max()
            .unwrap_or(0);

        count + variation_count
    }

    pub fn komi(&self) -> f32 {
        self.komi
    }

    pub fn get_variation(&self, variation_index: usize) -> Option<&Game> {
        self.variations.get(variation_index)
    }

    pub fn goban(&self) -> Option<&Goban> {
        if self.current < self.nodes.len() {
            Some(&self.nodes[self.current as usize].1)
        } else {
            None
        }
    }

    pub fn add_stone(&mut self, pos: impl Into<Position>, color: Color) -> Result<&Goban, Error> {
        let pos = pos.into();
        if self.is_directly_retaking_ko(pos, color) {
            return Err(Error::RetakingKo);
        }
        let goban = self.goban().unwrap().add_stone(pos, color)?;
        self.nodes[self.current].1 = goban;
        Ok(&self.nodes[self.current].1)
    }

    pub fn play_move(&mut self, pos: impl Into<Position>, color: Color) -> Result<&Goban, Error> {
        let pos = pos.into();
        if self.is_directly_retaking_ko(pos, color) {
            return Err(Error::RetakingKo);
        }
        let mut goban = self.goban().unwrap().place_stone(pos, color)?;
        let removed = goban.remove_dead_stones(!color);
        let valid = goban.is_valid();
        if !valid {
            Err(Error::SuicidalMove)
        } else {
            goban.capture_stones(removed.len(), !color);

            self.nodes.push((Node::Move(Move {
                position: pos,
                color: color,
                captures: removed
            }), goban));
            self.current += 1;
            Ok(&self.nodes[self.current].1)
        }
    }

    fn is_directly_retaking_ko(&self, pos: impl Into<Position>, color: Color) -> bool {
        let pos = pos.into();
        match self.nodes.last() {
            Some((Node::Move(m), _)) => {
                m.color == !color && m.captures.len() == 1 && m.captures[0] == pos
            },
            _ => false,
        }
    }
}

impl From<&SgfTree> for Game {
    fn from(tree: &SgfTree) -> Game {
        let mut game = Game::default();
        tree.nodes.iter().for_each(|node| {
            node.tokens.iter().for_each(|token| {
                match token {
                    SgfToken::Move{color, coordinate} => {
                        match game.play_move(*coordinate, *color) {
                            Err(e) => {
                                println!("Error parsing sgf token: {:?}, {:?}", token, e);
                            },
                            _ => {}

                        }
                    },
                    SgfToken::Add{color, coordinate} => {
                        match game.add_stone(*coordinate, *color) {
                            Err(e) => {
                                println!("Error parsing sgf token: {:?}, {:?}", token, e);
                            },
                            _ => {}

                        }
                    },
                    _ => {}

                }
            });
        });
        tree.variations.iter().for_each(|variation| {
            let g: Game = variation.into();
            game.variations.push(g);
        });
        game
    }
}
