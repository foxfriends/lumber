use super::*;

yes!(handle_len0, Rule::handle, "hello/0");
yes!(handle_len2, Rule::handle, "hello/2");
yes!(handle_name, Rule::handle, "hello:to");
yes!(handle_name2, Rule::handle, "hello:to:from");
yes!(handle_name2_len2, Rule::handle, "hello:to:from/2");
yes!(handle_name_len2_name, Rule::handle, "hello:to/2:from");
yes!(handle_name_len2_name_len2, Rule::handle, "hello:to/2:from/2");
yes!(handle_special, Rule::handle, "hello:#'it's time'#/2");

no!(handle_scoped_len0, Rule::handle, "hello::world/0");
no!(handle_scoped_len2, Rule::handle, "hello::world/2");
no!(handle_scoped_name, Rule::handle, "hello::world:to");
no!(handle_scoped_name2, Rule::handle, "hello::world:to:from");
no!(handle_scoped_name2_len2, Rule::handle, "hello::world:to:from/2");
no!(handle_scoped_name_len2_name, Rule::handle, "hello::world:to/2:from");
no!(handle_scoped_name_len2_name_len2, Rule::handle, "hello::world:to/2:from/2");
