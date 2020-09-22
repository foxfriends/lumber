use super::*;

yes!(rule_none, Rule::rule, "hello :- true.");
no!(rule_empty, Rule::rule, "hello() :- true.");
yes!(rule_one, Rule::rule, "hello(world) :- true.");
yes!(rule_multiple, Rule::rule, "hello(world, again) :- true.");
no!(rule_scoped, Rule::rule, "test::hello(a) :- true.");
