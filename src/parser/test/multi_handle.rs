use super::*;

yes!(multi_handle_single, Rule::multi_handle, "hello(world/2)");
yes!(multi_handle_double, Rule::multi_handle, "hello(world/2, hello/2)");
yes!(multi_handle_glob, Rule::multi_handle, "hello");
yes!(multi_handle_alias, Rule::multi_handle, "hello(alias(world/2, as: world/0:name:with))");
yes!(multi_handle_multi_alias, Rule::multi_handle, "hello(alias(hello/2, as: test/2), alias(test/2, as: hello/2))");

yes!(multi_handle_in_lib, Rule::multi_handle, "@std::list(len/2)");
yes!(multi_handle_in_parent, Rule::multi_handle, "^::test(len/2)");
yes!(multi_handle_in_root, Rule::multi_handle, "~::test(len/2)");
yes!(multi_handle_parent, Rule::multi_handle, "@std(len/2)");
yes!(multi_handle_lib, Rule::multi_handle, "^(len/2)");
yes!(multi_handle_root, Rule::multi_handle, "~(len/2)");
no!(multi_handle_nested_scope, Rule::multi_handle, "@std::list(len(test/2))");
no!(multi_handle_invalid_alias, Rule::multi_handle, "@std::list(alias(x/0, y/0))");
