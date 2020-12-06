use super::*;

test! {
    lumber_tests => r#"
    :- use(@core(equal/2)).
    hello(a, b).
    :- test(hello(a, b)).
    :- test(hello(a, B), equal(B, b)).
    :- test(hello(A, B), equal(A, a), equal(B, b)).
    "#
}

#[test]
#[rustfmt::skip]
fn lumber_tests_fail() {
    match Lumber::builder().test(true).build_from_str(r#"
        hello(a, b).
        :- test(hello(a, c)).
    "#) {
        Ok(..) => panic!("Expected tests to fail"),
        Err(error) => assert_eq!(error.kind, ErrorKind::Test),
    }
}

#[test]
#[rustfmt::skip]
fn lumber_tests_no_resolve() {
    match Lumber::builder().test(true).build_from_str(r#"
        :- test(hello(a, c)).
    "#) {
        Ok(..) => panic!("Expected program not to compile"),
        Err(error) => assert_eq!(error.kind, ErrorKind::Multiple),
    }
}

#[test]
#[rustfmt::skip]
fn lumber_tests_compile_without_test() {
    assert!(Lumber::builder().test(false).build_from_str(r#"
        hello(a, b).
        :- test(hello(a, c)).
    "#).is_ok());
}
