use super::*;

yes!(expression_struct, Rule::expression, "hello(A, B)");
yes!(expression_var, Rule::expression, "Y");
yes!(expression_call, Rule::expression, "call!(test, A)");
yes!(expression_operation, Rule::expression, "(3 + A)");
yes!(expression_long_operation, Rule::expression, "(3 + A * 3 + B)");
no!(expression_assumptions, Rule::expression, "A <- B, (A + 2)");
no!(expression_steps, Rule::expression, "A -> B");
no!(expression_conjunctions, Rule::expression, "A , B");
