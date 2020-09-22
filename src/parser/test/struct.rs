use super::*;

yes!(struct_empty, Rule::struct_, "test");
yes!(struct_values, Rule::struct_, "test(3, 4)");
yes!(struct_patterns, Rule::struct_, "test(A, _)");
yes!(struct_nested, Rule::struct_, "test(yes(A), no(E))");
yes!(struct_named_fields, Rule::struct_, "test(left: yes, right: _)");
no!(struct_scoped, Rule::struct_, "hello::test(ah)");
