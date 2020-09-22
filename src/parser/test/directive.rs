use super::*;

yes!(directive_nat, Rule::directive, ":- nat(hello/2).");
yes!(directive_mod, Rule::directive, ":- mod(hello).");
yes!(directive_use, Rule::directive, ":- use(hello).");
yes!(directive_pub, Rule::directive, ":- pub(hello/2).");
yes!(directive_mut, Rule::directive, ":- mut(hello/2).");
yes!(directive_inc, Rule::directive, ":- inc(hello/2).");
no!(directive_other, Rule::directive, ":- what.");
