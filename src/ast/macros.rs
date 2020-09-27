macro_rules! just {
    ($rule:expr, $pairs:expr) => {{
        let mut pairs = $pairs.into_iter();
        let pair = pairs.next().unwrap();
        assert!(pairs.peek().is_none());
        assert_eq!(pair.as_rule(), $rule);
        pair
    }};

    ($pairs:expr) => {{
        let mut pairs = $pairs.into_iter();
        let pair = pairs.next().unwrap();
        assert!(pairs.peek().is_none());
        pair
    }};
}
