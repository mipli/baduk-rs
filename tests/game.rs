#[cfg(test)]
mod game_tests {
    use baduk_rs::{Game, Color, Error};
    use sgf_parser::{parse};

    #[test]
    fn it_can_create_new_game_from_sgf() {
        let tree = parse("(;B[aa];W[bb])").unwrap();
        let game: Game = (&tree).into();
        assert_eq!(game.count_moves(), 2);
        assert_eq!(game.komi(), 6.5f32);
        let captures = game.goban().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);
    }

    #[test]
    fn it_can_create_new_game_from_sgf_with_variation() {
        let tree = parse("(;B[aa];W[bb](;B[cc])(;B[kk]W[qq]))").unwrap();
        let game: Game = (&tree).into();
        assert_eq!(game.count_moves(), 4);
        assert_eq!(game.komi(), 6.5f32);
        assert_eq!(game.get_variation(0).unwrap().count_moves(), 1);
        assert_eq!(game.get_variation(1).unwrap().count_moves(), 2);
        let captures = game.goban().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);
    }

    #[test]
    fn it_can_create_new_game_from_sgf_with_added_stones() {
        let tree = parse("(;W[ba];W[ab];AB[aa])").unwrap();
        let game: Game = (&tree).into();
        assert_eq!(game.count_moves(), 2);
        assert_eq!(game.komi(), 6.5f32);
        let goban = game.goban().unwrap();
        assert!(!goban.is_valid());
        let captures = goban.captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);
    }

    #[test]
    fn it_can_create_new_game() {
        let game = Game::default();
        assert_eq!(game.count_moves(), 0);
        assert_eq!(game.komi(), 6.5f32);
        assert!(game.goban().unwrap().is_empty());
        let captures = game.goban().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);
    }

    #[test]
    fn it_can_play_moves() {
        let mut game = Game::default();
        let _ = game.play_move((4, 3), Color::Black);
        let _ = game.play_move((16, 16), Color::White);
        let _ = game.play_move((16, 4), Color::Black);
        let _ = game.play_move((3, 16), Color::White);
        assert_eq!(game.count_moves(), 4);
        assert!(!game.goban().unwrap().is_empty());
        let captures = game.goban().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);

        assert_eq!(game.goban().unwrap().get_stone((4, 3)), Some(&Color::Black));
        assert_eq!(game.goban().unwrap().get_stone((16, 16)), Some(&Color::White));
        assert_eq!(game.goban().unwrap().get_stone((16, 4)), Some(&Color::Black));
        assert_eq!(game.goban().unwrap().get_stone((3, 16)), Some(&Color::White));
    }

    #[test]
    fn it_can_capture_stones() {
        let mut game = Game::default();
        let _ = game.play_move((1, 1), Color::Black);
        let _ = game.play_move((1, 2), Color::White);
        let _ = game.play_move((4, 4), Color::Black);
        let _ = game.play_move((2, 1), Color::White);
        assert_eq!(game.count_moves(), 4);
        assert!(!game.goban().unwrap().is_empty());
        let captures = game.goban().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 1);

        assert_eq!(game.goban().unwrap().get_stone((1, 1)), None);
        assert_eq!(game.goban().unwrap().get_stone((1, 2)), Some(&Color::White));
        assert_eq!(game.goban().unwrap().get_stone((2, 1)), Some(&Color::White));
    }

    #[test]
    fn it_does_not_allow_suicide() {
        let mut game = Game::default();
        let _ = game.play_move((1, 2), Color::Black);
        let _ = game.play_move((3, 3), Color::White);
        let _ = game.play_move((2, 1), Color::Black);
        let err = game.play_move((1, 1), Color::White);
        match err {
            Err(Error::SuicidalMove) => assert!(true),
            _ => assert!(false),
        }

        assert_eq!(game.count_moves(), 3);
        assert!(!game.goban().unwrap().is_empty());
        let captures = game.goban().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 0);

    }

    #[test]
    fn it_handles_ko() {
        let mut game = Game::default();
        let _ = game.play_move((1, 1), Color::Black);
        let _ = game.play_move((1, 2), Color::White);
        let _ = game.play_move((2, 2), Color::Black);
        let _ = game.play_move((2, 1), Color::White);

        assert_eq!(game.count_moves(), 4);

        let captures = game.goban().unwrap().captures();
        assert_eq!(captures.white, 0);
        assert_eq!(captures.black, 1);

        let err = game.play_move((1, 1), Color::Black);
        match err {
            Err(Error::RetakingKo) => assert!(true),
            _ => assert!(false),
        }
        assert_eq!(game.count_moves(), 4);

        let _ = game.play_move((3, 1), Color::Black);
        let _ = game.play_move((6, 6), Color::White);
        let res = game.play_move((1, 1), Color::Black);
        assert!(res.is_ok());
        assert_eq!(game.count_moves(), 7);

        let captures = game.goban().unwrap().captures();
        assert_eq!(captures.white, 1);
        assert_eq!(captures.black, 1);
    }
}
