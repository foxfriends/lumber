use super::*;

yes!(list_basic, Rule::list, "[1, 2, 3]");
yes!(list_tail, Rule::list, "[1, 2, 3 | A]");
yes!(list_nested, Rule::list, "[1, 2 | [3, 4]]");
yes!(list_patterns, Rule::list, "[1, \"hello\", A, B, _, test(a), [1]]");
yes!(list_empty, Rule::list, "[]");
no!(list_invalid_tail, Rule::list, "[1, 2, 3 | 3]");
no!(list_multi_tail, Rule::list, "[1, 2, 3 | A | B]");
no!(list_no_head, Rule::list, "[| A | B]");
