macro_rules! integer {
    ($value:expr) => {
        match &$value {
            Some(crate::Value::Integer(int)) => int.to_owned(),
            _ => return Box::new(std::iter::empty()),
        }
    };
}
