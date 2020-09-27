use super::*;

yes! {
    comment_line => r#"
    // hello.
    "#
}

yes! {
    comment_block => r#"
    /* hello. */
    "#
}

yes! {
    comment_block_multiline => r#"
    /**
     * hello.
     */
    "#
}

no! {
    comment_unclosed => r#"
    /*
    "#
}

yes! {
    comment_nested => r#"
    /* /* Hello */ */
    "#
}

no! {
    comment_nested_unbalanced => r#"
    /* /* Hello */
    "#
}

yes! {
    comment_block_in_line => r#"
    // /* hello */
    "#
}

yes! {
    comment_block_unclosed_in_line => r#"
    // /* hello
    "#
}

no! {
    comment_block_unopened => r#"
    // /* hello
    */
    "#
}

yes! {
    comment_in_definition => r#"
    test.
    hello /* hello */ :- test // what
        .
    "#
}

no! {
    comment_used => r#"
    // test.
    hello :- test.
    "#
}

yes! {
    comment_reference_undefined => r#"
    // test.
    "#
}
