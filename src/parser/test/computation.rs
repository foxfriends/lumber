use super::*;

yes!(computation_struct, Rule::computation, "hello(A, B)");
yes!(computation_var, Rule::computation, "Y");
yes!(computation_call, Rule::computation, "call!(test, A)");
yes!(computation_operation, Rule::computation, "3 + A");
yes!(computation_chained, Rule::computation, "3 + A * 3 + B");
no!(computation_assumptions, Rule::computation, "A <- B, A + 2");
no!(computation_steps, Rule::computation, "A ! B");
no!(computation_conjunctions, Rule::computation, "A , B");
