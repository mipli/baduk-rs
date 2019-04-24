#[cfg(test)]
mod game_tests {
    use baduk_rs::{GameTree, GameState, Color, Error};
    use sgf_parser::{parse};

    #[test]
    fn it_can_create_new_game() {
        let game = GameTree::default();
        assert_eq!(game.count_nodes(), 1);
        assert!(game.current_state().is_none());
    }

    #[test]
    fn it_can_create_new_game_from_game_state() {
        let state: GameState = "
        .x...
        x..x.
        .xoox
        .oxx.
        .o..."
            .parse()
            .unwrap();
        let tree: GameTree = state.into();
        assert_eq!(tree.nodes.len(), 1);
        if let Some(state) = tree.current_state() {
            assert_eq!(state.get_stone((2, 1)), Some(&Color::Black));
            assert_eq!(state.get_stone((2, 5)), Some(&Color::White));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn it_can_play_moves() {
        let mut game = GameTree::new(19, 19);
        let id = game.play_move((4, 3), Color::Black);
        assert_eq!(id, Ok(1));
        let id = game.play_move((16, 16), Color::White);
        assert_eq!(id, Ok(2));
        let id = game.play_move((16, 4), Color::Black);
        assert_eq!(id, Ok(3));
        let id = game.play_move((3, 16), Color::White);
        assert_eq!(id, Ok(4));
        assert_eq!(game.count_nodes(), 5);
        let state = game.current_state().unwrap();
        let captures = state.captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);

        assert_eq!(game.nodes[0].tokens.len(), 0);
        assert_eq!(game.nodes[1].tokens.len(), 1);
        assert_eq!(game.nodes[2].tokens.len(), 1);
        assert_eq!(game.nodes[3].tokens.len(), 1);
        assert_eq!(game.nodes[4].tokens.len(), 1);

        assert_eq!(state.get_stone((4, 3)), Some(&Color::Black));
        assert_eq!(state.get_stone((16, 16)), Some(&Color::White));
        assert_eq!(state.get_stone((16, 4)), Some(&Color::Black));
        assert_eq!(state.get_stone((3, 16)), Some(&Color::White));
    }

    #[test]
    fn it_can_capture_stones() {
        let mut game = GameTree::new(19, 19);
        let _ = game.play_move((1, 1), Color::Black);
        let _ = game.play_move((1, 2), Color::White);
        let _ = game.play_move((4, 4), Color::Black);
        let _ = game.play_move((2, 1), Color::White);
        assert_eq!(game.count_nodes(), 5);

        let state = game.current_state().unwrap();
        let captures = state.captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 1);

        assert_eq!(state.get_stone((1, 1)), None);
        assert_eq!(state.get_stone((1, 2)), Some(&Color::White));
        assert_eq!(state.get_stone((2, 1)), Some(&Color::White));
    }

    #[test]
    fn it_does_not_allow_suicide() {
        let mut game = GameTree::new(19, 19);
        let _ = game.play_move((1, 2), Color::Black);
        let _ = game.play_move((3, 3), Color::White);
        let _ = game.play_move((2, 1), Color::Black);
        let err = game.play_move((1, 1), Color::White);
        match err {
            Err(Error::SuicidalMove) => assert!(true),
            _ => assert!(false),
        }

        assert_eq!(game.count_nodes(), 4);
        let captures = game.current_state().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);
    }

    #[test]
    fn it_handles_ko() {
        let mut game = GameTree::new(5, 5);
        let _ = game.play_move((2, 1), Color::Black);
        let _ = game.play_move((1, 2), Color::Black);
        let _ = game.play_move((2, 3), Color::Black);
        let _ = game.play_move((3, 1), Color::White);
        let _ = game.play_move((4, 2), Color::White);
        let _ = game.play_move((3, 3), Color::White);
        let _ = game.play_move((3, 2), Color::Black);

        let captures = game.current_state().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);

        let _ = game.play_move((2, 2), Color::White);

        let captures = game.current_state().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 1);

        let err = game.play_move((3, 2), Color::Black);
        match err {
            Err(Error::RetakingKo) => assert!(true),
            _ => assert!(false),
        }

        let _ = game.play_move((4, 4), Color::Black);
        let _ = game.play_move((5, 5), Color::White);
        let res = game.play_move((3, 2), Color::Black);
        assert!(res.is_ok());

        let captures = game.current_state().unwrap().captures();
        assert_eq!(captures.white, 1);
        assert_eq!(captures.black, 1);
    }

    #[test]
    fn it_handles_almost_ko() {
        let mut game = GameTree::new(7, 7);
        let _ = game.play_move((1, 2), Color::Black);
        let _ = game.play_move((2, 1), Color::Black);
        let _ = game.play_move((3, 1), Color::Black);
        let _ = game.play_move((2, 3), Color::Black);
        let _ = game.play_move((3, 3), Color::Black);

        let _ = game.play_move((2, 2), Color::White);
        let _ = game.play_move((3, 2), Color::White);
        let _ = game.play_move((4, 1), Color::White);
        let _ = game.play_move((4, 3), Color::White);
        let _ = game.play_move((5, 2), Color::White);

        let _ = game.play_move((4, 2), Color::Black);

        let captures = game.current_state().unwrap().captures();
        assert_eq!(captures.white, 2);
        assert_eq!(captures.black, 0);

        let res = game.play_move((3, 2), Color::White);
        assert!(res.is_ok());

        let captures = game.current_state().unwrap().captures();
        assert_eq!(captures.white, 2);
        assert_eq!(captures.black, 1);

        let res = game.play_move((2, 2), Color::White);
        assert!(res.is_ok());

        let _ = game.play_move((4, 2), Color::Black);

        let captures = game.current_state().unwrap().captures();
        assert_eq!(captures.white, 4);
        assert_eq!(captures.black, 1);
    }

    #[test]
    fn it_can_create_new_game_from_sgf() {
        let tree = parse("(;B[aa];W[bb])").unwrap();
        let game: GameTree = (&tree).into();
        assert_eq!(game.count_nodes(), 3);
        let captures = game.current_state().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);
    }

    #[test]
    fn it_can_create_new_game_from_sgf_with_added_stones() {
        let tree = parse("(;W[ba];W[ab]AB[aa])").unwrap();
        let game: GameTree = (&tree).into();
        assert_eq!(game.count_nodes(), 3);
        let state = game.current_state().unwrap();
        println!("{:?}", state);
        assert!(!state.is_valid());
        let captures = state.captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);
    }

    #[test]
    fn it_can_play_a_variant_move() {
        let tree = parse("(;B[ba];W[ab])").unwrap();
        let mut game: GameTree = (&tree).into();
        let var_root = game.play_move_as_variation((1, 1), Color::Black, 1).unwrap();
        let _ = game.play_move_as_variation((4, 4), Color::Black, var_root);
        assert_eq!(game.count_nodes(), 5);
        assert_eq!(game.nodes[0].children.len(), 1);
        assert_eq!(game.nodes[1].children.len(), 2);
        assert_eq!(game.nodes[2].children.len(), 0);
        assert_eq!(game.nodes[3].children.len(), 1);
        assert_eq!(game.nodes[4].children.len(), 0);
    }

    #[test]
    fn it_can_create_new_game_from_sgf_with_variation() {
        let tree = parse("(;B[aa];W[bb](;B[cc])(;B[kk]W[qq]))").unwrap();
        let game: GameTree = (&tree).into();
        assert_eq!(game.count_nodes(), 6);
        assert_eq!(game.nodes[0].children.len(), 1);
        assert_eq!(game.nodes[1].children.len(), 1);
        assert_eq!(game.nodes[2].children.len(), 2);
        assert_eq!(game.nodes[3].children.len(), 0);
        assert_eq!(game.nodes[4].children.len(), 1);
        assert_eq!(game.nodes[4].children.len(), 1);
        assert_eq!(game.nodes[5].children.len(), 0);
    }
}
