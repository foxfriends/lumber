use super::*;

yes!(operator_symbolic, Rule::operator, "+");
yes!(operator_symbolic_longer, Rule::operator, "+*!%$&|");
no!(operator_symbolic_underscore, Rule::operator, "+_+");
no!(operator_bound, Rule::operator, "!");
no!(operator_unbound, Rule::operator, "?");
no!(operator_implication, Rule::operator, "->");
no!(operator_colon, Rule::operator, "+:+");
