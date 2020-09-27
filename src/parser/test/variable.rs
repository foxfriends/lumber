use super::*;

yes!(variable_alpha, Rule::variable, "Hello");
yes!(variable_alpha_numbers, Rule::variable, "Hello3");
yes!(variable_alpha_underscores, Rule::variable, "Hello_World");
no!(variable_lower, Rule::variable, "hello_world");
no!(variable_start_number, Rule::variable, "3Hello");
no!(variable_special, Rule::variable, "Hello.World");
no!(variable_quoted, Rule::variable, "#'hello'#");
no!(variable_under, Rule::variable, "_hello");
yes!(variable_title, Rule::variable, "Æther");
yes!(variable_greek, Rule::variable, "Λυμβερ");
yes!(variable_latin, Rule::variable, "Ɑɣɸ");
yes!(variable_cyrillic, Rule::variable, "Лумбэр");
