use super::*;

yes!(operator_named, Rule::operator, "`hello`");
yes!(operator_named_special, Rule::operator, "`#'it's an operator'#`");
yes!(operator_named_scoped, Rule::operator, "`hello::test`");
yes!(operator_symbolic, Rule::operator, "+");
yes!(operator_symbolic_longer, Rule::operator, "+*!%$&|");
no!(operator_symbolic_underscore, Rule::operator, "+_+");
no!(operator_assumption, Rule::operator, "<-");
no!(operator_implication, Rule::operator, "->");
