use super::*;

yes!(literal_integer, Rule::literal, "123");
yes!(literal_integer_very_large, Rule::literal, "123456789123456789123456789123456789123456789");
yes!(literal_integer_zero, Rule::literal, "0");
no!(literal_integer_letter, Rule::literal, "123A");

yes!(literal_integer_hex, Rule::literal, "0xFFA0");
yes!(literal_integer_hex_zero_start, Rule::literal, "0x00FF");
yes!(literal_integer_hex_lower, Rule::literal, "0xffa0");
yes!(literal_integer_hex_mix, Rule::literal, "0xf0FfA");
no!(literal_integer_hex_over, Rule::literal, "0xFG");
no!(literal_integer_zero_start, Rule::literal, "0123");

yes!(literal_integer_bin, Rule::literal, "0b10");
yes!(literal_integer_bin_zero_start, Rule::literal, "0b0010");
no!(literal_integer_bin_over, Rule::literal, "0b021");

yes!(literal_decimal, Rule::literal, "123.456");
no!(literal_decimal_too_many, Rule::literal, "123.456.789");
no!(literal_decimal_hex, Rule::literal, "0x0ff.3");

yes!(literal_string, Rule::literal, "\"hello\"");
yes!(literal_string_strong, Rule::literal, "##\"he\"\"#llo\"##");

no!(literal_boolean, Rule::literal, "true");
