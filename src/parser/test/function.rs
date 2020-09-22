use super::*;

yes!(function_no_args, Rule::function, "test! <- 3.");
yes!(function_args, Rule::function, "test!(a, b) <- 3.");
yes!(function_named_args, Rule::function, "test!(a: A, b: B) <- 3.");
no!(function_empty, Rule::function, "test!() <- 3.");
no!(function_scoped, Rule::function, "hello::test!() <- 3.");
no!(function_no_excl, Rule::function, "test <- 3.");
