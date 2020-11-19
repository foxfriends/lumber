use super::*;

yes!(struct_empty, Rule::struct_, "test");
yes!(struct_list, Rule::struct_, "test([3, 4])");
yes!(struct_list_shorthand, Rule::struct_, "test[3, 4]");
yes!(struct_single_var, Rule::struct_, "test(A)");
yes!(struct_single_value, Rule::struct_, "test(1)");
yes!(struct_single_wild, Rule::struct_, "test(_)");
yes!(struct_nested, Rule::struct_, "test(yes(A))");
yes!(struct_record, Rule::struct_, "test({ left: yes, right: _ })");
yes!(struct_record_shorthand, Rule::struct_, "test { left: yes, right: _ }");
no!(struct_scoped, Rule::struct_, "hello::test(ah)");
