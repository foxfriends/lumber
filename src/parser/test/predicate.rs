use super::*;

yes!(predicate_empty, Rule::predicate, "test");
yes!(predicate_empty_scoped, Rule::predicate, "hello::test");
yes!(predicate_patterns, Rule::predicate, "test(a, b)");
yes!(predicate_patterns_scoped, Rule::predicate, "hello::test(a, b)");
yes!(predicate_named_fields, Rule::predicate, "test(left: a, right: b)");
yes!(predicate_fancy_scope, Rule::predicate, "^::^::test");
no!(predicate_no_args, Rule::predicate, "test()");
no!(predicate_call, Rule::predicate, "test!(3, 4)");
