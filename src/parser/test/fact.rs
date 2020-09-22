use super::*;

yes!(fact_none, Rule::fact, "hello.");
no!(fact_empty, Rule::fact, "hello().");
yes!(fact_one, Rule::fact, "hello(world).");
yes!(fact_multiple, Rule::fact, "hello(world, again).");
no!(fact_scoped, Rule::fact, "aa::hello(b).");
