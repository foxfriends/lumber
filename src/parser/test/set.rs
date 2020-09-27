use super::*;

yes!(set_basic, Rule::set, "{1, 2, 3}");
yes!(set_tail, Rule::set, "{1, 2, 3 | A}");
yes!(set_nested, Rule::set, "{1, 2 | {3, 4}}");
yes!(set_patterns, Rule::set, "{1, \"hello\", A, B, _, test(a), {1}, [1]}");
yes!(set_empty, Rule::set, "{}");
no!(set_invalid_tail, Rule::set, "{1, 2, 3 | 3}");
no!(set_list_tail, Rule::set, "{1, 2, 3 | [4]}");
no!(set_multi_tail, Rule::set, "{1, 2, 3 | A | B}");
no!(set_no_head, Rule::set, "{| A | B}");
no!(set_list, Rule::set, "[1,2,3]");
