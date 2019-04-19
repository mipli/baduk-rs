#[cfg(test)]
mod tests {
    use baduk_rs::Position;

    #[test]
    fn it_can_convert_from_usize_tuple() {
        let pos: Position = (2, 3).into();
        assert_eq!(pos.x(), 2);
        assert_eq!(pos.y(), 3);
    }

    #[test]
    fn it_can_convert_from_str() {
        let pos: Position = "A1".into();
        assert_eq!(pos.x(), 1);
        assert_eq!(pos.y(), 1);

        let pos: Position = "t19".into();
        assert_eq!(pos.x(), 19);
        assert_eq!(pos.y(), 19);

        let pos: Position = "J12".into();
        assert_eq!(pos.x(), 9);
        assert_eq!(pos.y(), 12);
    }
}
