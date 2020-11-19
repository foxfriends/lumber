use super::*;

yes!(aggregation_set, Rule::aggregation, "{ X : between(3, 10, X) }");
yes!(aggregation_list, Rule::aggregation, "[ X : between(3, 10, X) ]");
yes!(aggregation_complex_binding, Rule::aggregation, "{ pair[X, Y] : between(X, Y, 5) }");
yes!(aggregation_long_body, Rule::aggregation, "{ X : left(X), right(X), between(10, 50, X) }");
no!(aggregation_tuple, Rule::aggregation, "( X : between(3, 10, X) )");
no!(aggregation_wrong_separator, Rule::aggregation, "{ X | between(3, 10, X) }");
