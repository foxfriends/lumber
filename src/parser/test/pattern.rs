use super::*;

yes!(pattern_bound, Rule::pattern, "!A");
yes!(pattern_bound_value, Rule::pattern, "!3");
yes!(pattern_bound_shorthand, Rule::pattern, "!");
yes!(pattern_unbound, Rule::pattern, "?A");
yes!(pattern_unbound_value, Rule::pattern, "?3");
yes!(pattern_unbound_shorthand, Rule::pattern, "?");

no!(pattern_bound_unbound, Rule::pattern, "!?_");
no!(pattern_unbound_bound, Rule::pattern, "?!_");
