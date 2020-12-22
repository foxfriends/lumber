use super::*;

yes!(expression_struct, Rule::expression, "hello[A, B]");
yes!(expression_var, Rule::expression, "Y");
yes!(expression_parenthesized, Rule::expression, "(3 + A)");
yes!(expression_prefix, Rule::expression, "+1");
yes!(expression_multi_prefix, Rule::expression, "- + - +1");
yes!(expression_infix, Rule::expression, "1+1");
yes!(expression_multi_infix, Rule::expression, "1 + - +1");
yes!(expression_long_operation, Rule::expression, "(3 + A * 3 + B)");
no!(expression_postfix, Rule::expression, "1+");
no!(expression_unifications, Rule::expression, "A =:= B, (A + 2)");
no!(expression_steps, Rule::expression, "A -> B");
no!(expression_conjunctions, Rule::expression, "A , B");
