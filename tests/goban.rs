#[cfg(test)]
mod goban_tests {
    use baduk_rs::{Error, Goban, Color};

    #[test]
    fn it_creates_empty_goban() {
        let goban = Goban::default();
        assert_eq!(goban.dimensions(), (19, 19));
        assert!(goban.is_empty());
    }

    #[test]
    fn it_can_parse_strings() {
        let goban: Goban = ".x..o....".parse().unwrap();
        assert_eq!(goban.dimensions(), (3, 3));
        assert!(!goban.is_empty());
    }

    #[test]
    fn it_requires_square_board() {
        let err = ".x.o....".parse::<Goban>();
        assert!(err.is_err());
        match err {
            Err(Error::InvalidInputSize) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn it_returns_error_on_invalid_position() {
        let goban = Goban::default();
        let err = goban.place_stone((25, 5), Color::White);
        assert!(err.is_err());
        match err {
            Err(e) => assert_eq!(e, Error::InvalidPosition((25, 5).into())),
            Ok(_) => assert!(false)
        }
    }

    #[test]
    fn it_can_place_a_stone() {
        let goban = Goban::default();
        let goban = goban.place_stone((1, 1), Color::White).unwrap();
        assert!(!goban.is_empty());
        assert_eq!(goban.get_stone((1, 1)), Some(&Color::White));
    }

    #[test]
    fn it_can_print() {
        let goban = Goban::new(3, 3);
        let goban = goban.place_stone((1, 1), Color::White).unwrap();
        let goban = goban.place_stone((2, 2), Color::Black).unwrap();
        assert_eq!(format!("{:?}", goban), "o..\n.x.\n...");
    }

    #[test]
    fn it_cannot_place_at_same_location() {
        let goban = Goban::default();
        let goban = goban.place_stone((1, 1), Color::White).unwrap();
        let err = goban.place_stone((1, 1), Color::White);
        assert!(err.is_err());
        match err {
            Err(e) => assert_eq!(e, Error::AlreadyOccupied((1, 1).into())),
            Ok(_) => assert!(false)
        }
    }

    #[test]
    fn it_can_count_liberties() {
        let goban = Goban::default();
        let goban = goban.place_stone((5, 3), Color::White).unwrap();
        assert_eq!(goban.count_liberties((5, 3)), Some(4));
        let goban = goban.place_stone((5, 4), Color::Black).unwrap();
        assert_eq!(goban.count_liberties((5, 3)), Some(3));
        let goban = goban.place_stone((4, 3), Color::White).unwrap();
        assert_eq!(goban.count_liberties((5, 3)), Some(5));

        let goban = goban.place_stone((1, 1), Color::Black).unwrap();
        assert_eq!(goban.count_liberties((1, 1)), Some(2));

        let goban = goban.place_stone((19, 19), Color::Black).unwrap();
        assert_eq!(goban.count_liberties((19, 19)), Some(2));
    }

    #[test]
    fn it_can_remove_a_dead_stone() {
        let goban = Goban::default();
        let goban = goban.place_stone((2, 3), Color::Black).unwrap();
        let goban = goban.place_stone((4, 3), Color::Black).unwrap();
        let goban = goban.place_stone((3, 2), Color::Black).unwrap();
        let goban = goban.place_stone((3, 3), Color::White).unwrap();
        let mut goban = goban.place_stone((3, 4), Color::Black).unwrap();
        assert_eq!(goban.count_liberties((3, 3)), Some(0));
        assert_eq!(goban.remove_dead_stones(Color::White).len(), 1);
    }

    #[test]
    fn it_can_remove_a_chain() {
        let mut goban: Goban = "
        .....
        .ooo.
        oxxxo
        .oxo.
        ..o.."
            .parse()
            .unwrap();
        assert_eq!(goban.count_liberties((3, 3)), Some(0));
        assert_eq!(goban.count_liberties((2, 2)), Some(5));
        assert_eq!(goban.remove_dead_stones(Color::Black).len(), 4);
        assert_eq!(goban.remove_dead_stones(Color::White).len(), 0);
    }

    #[test]
    fn it_can_remove_multiple_chains() {
        let mut goban: Goban = "
        ox...
        x.xx.
        .xoox
        .oxx.
        .o..."
            .parse()
            .unwrap();
        assert_eq!(goban.count_liberties((1, 1)), Some(0));
        assert_eq!(goban.count_liberties((3, 3)), Some(0));
        assert_eq!(goban.remove_dead_stones(Color::White).len(), 3);
        assert_eq!(goban.remove_dead_stones(Color::Black).len(), 0);
    }

    #[test]
    fn it_can_test_if_valid() {
        let goban: Goban = "
        o....
        x..x.
        .xoox
        .oxx.
        .o..."
            .parse()
            .unwrap();
        assert!(goban.is_valid());

        let goban: Goban = "
        ox...
        x.xx.
        .xoox
        .oxx.
        .o..."
            .parse()
            .unwrap();
        assert!(!goban.is_valid());
    }
}
