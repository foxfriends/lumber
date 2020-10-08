use crate::*;
use std::convert::TryFrom;

#[test]
fn simple_query() {
    let src = r#"
    :- pub(hello/0).
    hello.
    "#;

    let program = Lumber::from_str(src).unwrap();
    assert!(program.check(&Question::try_from("hello").unwrap()));
    assert!(!program.check(&Question::try_from("wrong").unwrap()));
}
