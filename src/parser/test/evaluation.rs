use super::*;

yes!(evaluation_single, Rule::evaluation, "3");
yes!(evaluation_assumptions, Rule::evaluation, "A <- B, C <- 4, A * C + B");
no!(evaluation_assumptions_no_body, Rule::evaluation, "A <- B, C <- 4");
no!(evaluation_steps, Rule::evaluation, "check(A, B) -> 4");
