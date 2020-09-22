use super::*;

yes!(call_empty, Rule::call, "test!");
yes!(call_empty_scoped, Rule::call, "hello::test!");
yes!(call_patterns, Rule::call, "test!(a, b)");
yes!(call_patterns_scoped, Rule::call, "hello::test!(a, b)");
yes!(call_named_fields, Rule::call, "test!(left: a, right: b)");
yes!(call_fancy_scope, Rule::call, "^::^::test!");
no!(call_no_args, Rule::call, "test!()");
no!(call_no_excl, Rule::call, "test(3, 4)");
