use super::*;

yes!(scope_single, Rule::scope, "hello");
yes!(scope_double, Rule::scope, "hello::world");
yes!(scope_library, Rule::scope, "@lib::hello::world");
yes!(scope_root, Rule::scope, "~::hello::world");
yes!(scope_parent, Rule::scope, "^::^::hello::world");
yes!(scope_special_chars, Rule::scope, "'two words'::#'it's ok'#");
no!(scope_no_upper, Rule::scope, "::world");
no!(scope_no_child, Rule::scope, "world::");
no!(scope_parent_no_atom, Rule::scope, "^");
no!(scope_parent_multi_no_atom, Rule::scope, "^::^");
no!(scope_lib_no_atom, Rule::scope, "@lib");
no!(scope_root_no_atom, Rule::scope, "~");
