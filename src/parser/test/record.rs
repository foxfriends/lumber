use super::*;

yes!(record_empty, Rule::record, "{:}");
yes!(record_field, Rule::record, "{ a: 1 }");
yes!(record_fields_multiple, Rule::record, "{ a: 1, b: 2 }");
yes!(record_tail, Rule::record, "{ a: 1, b: 4, ..R }");
yes!(record_tail_wildcard, Rule::record, "{ a: 1, b: 3, .._ }");
yes!(record_tail_implicit_wildcard, Rule::record, "{ a: 1, b: 3, .. }");
yes!(record_nested, Rule::record, "{ a: 1, b: 3, ..{ c: 3 } }");
no!(record_empty_no_colon, Rule::record, "{}");
no!(record_fields_long, Rule::record, "{ a: 1, 2, b: 3, 4 }");
no!(record_set, Rule::record, "{ a, b }");
no!(record_unnamed_prefix_field, Rule::record, "{ a, b: 3 }");
no!(record_only_tail, Rule::record, "{ ..A }");
no!(record_invalid_tail, Rule::record, "{ a: 3, ..3 }");
no!(record_after_tail, Rule::record, "{ a: 3, ..A, B }");
no!(record_set_tail, Rule::record, "{ a: 3, ..{A} }");
no!(record_list_tail, Rule::record, "{ a: 3, ..[A] }");
no!(record_multi_tail, Rule::record, "{ a: 3, ..B, ..C }");
