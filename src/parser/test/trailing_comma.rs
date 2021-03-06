use super::*;

yes!(trailing_comma_list, Rule::list, "[1, 2, 3,]");
no!(trailing_comma_list_with_tail, Rule::list, "[1, 2, 3, ..A,]");
yes!(trailing_comma_set, Rule::set, "{1, 2, 3,}");
no!(trailing_comma_set_with_tail, Rule::set, "{1, 2, 3, ..A,}");
yes!(trailing_comma_record, Rule::record, "{a: 1, b: 2, c: 3,}");
no!(trailing_comma_record_with_tail, Rule::record, "{a: 1, b: 2, c: 3, ..A,}");
yes!(trailing_comma_head, Rule::head, "head(1, 2, 3,)");
yes!(trailing_comma_predicate, Rule::predicate, "pred(1, 2, 3,)");
yes!(trailing_comma_multi_handle, Rule::multi_handle, "multi(a/1, b/1, c/1,)");
no!(trailing_comma_struct, Rule::struct_, "struct(1,)");
