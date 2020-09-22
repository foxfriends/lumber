use super::*;

yes!(bare_atom_basic, Rule::atom, "hello");
yes!(bare_atom_underscoe, Rule::atom, "hello_world");
yes!(bare_atom_uppercase_middle, Rule::atom, "helloWorld");
yes!(bare_atom_greek, Rule::atom, "λυμβερ");
yes!(bare_atom_latin, Rule::atom, "ɑɣɸ");
yes!(bare_atom_cyrillic, Rule::atom, "лумбэр");

no!(bare_atom_hyphen, Rule::atom, "hello-world");
no!(bare_atom_dot, Rule::atom, "hello.world");
no!(bare_atom_start_underscore, Rule::atom, "_hello");
no!(bare_atom_start_hyphen, Rule::atom, "-hello");
no!(bare_atom_start_dot, Rule::atom, ".hello");
no!(bare_atom_start_upper, Rule::atom, "Hello");
no!(bare_atom_upper_greek, Rule::atom, "Λυμβερ");
no!(bare_atom_upper_latin, Rule::atom, "Ɑɣɸ");
no!(bare_atom_upper_cyrillic, Rule::atom, "Лумбэр");

yes!(quoted_atom, Rule::atom, "'hello'");
yes!(quoted_atom_upper, Rule::atom, "'HELLO'");
yes!(quoted_atom_special, Rule::atom, "'he.l-l.o'");
yes!(quoted_atom_quotes, Rule::atom, "#'he'lo'#");
yes!(quoted_atom_hashes, Rule::atom, "##'he'#' ## ### o'##");
no!(quoted_atom_empty, Rule::atom, "''");
no!(quoted_atom_strong_empty, Rule::atom, "##''##");
no!(quoted_atom_mismatched_front, Rule::atom, "#'hello'##");
no!(quoted_atom_mismatched_back, Rule::atom, "##'hello'#");
