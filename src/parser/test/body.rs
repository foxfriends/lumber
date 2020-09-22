use super::*;

yes!(body_single, Rule::body, "hello");
yes!(body_steps, Rule::body, "hello(A) ! test(A)");
yes!(body_cases, Rule::body, "hello(A) ; test(A)");
yes!(body_terms, Rule::body, "hello(A) , test(A)");
yes!(body_conds, Rule::body, "hello(A) -> test(A)");
yes!(body_assumptions, Rule::body, "hello(A) <- test!(B)");
yes!(body_combination, Rule::body, "hello(A) -> test(A) ; hello(B) -> test(B) ! test(C) , test(D)");
yes!(body_nested, Rule::body, "(hello(A) -> test(A) ; hello(B) -> test(B)) ! (test(C), test(D))");
