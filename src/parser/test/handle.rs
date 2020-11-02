use super::*;

yes!(handle_len0, Rule::handle, "hello/0");
yes!(handle_len2, Rule::handle, "hello/2");
yes!(handle_len0_f1, Rule::handle, "hello/0:to");
yes!(handle_len2_f1, Rule::handle, "hello/2:to");
yes!(handle_len2_f2, Rule::handle, "hello/2:to/2");
yes!(handle_len0_f2_f1, Rule::handle, "hello/0:to/2:from");
yes!(handle_len0_f2_f2, Rule::handle, "hello/0:to/2:from/2");
yes!(handle_special, Rule::handle, "hello/0:#'it's time'#/2");

no!(handle_no_len, Rule::handle, "hello");
no!(handle_no_len_field, Rule::handle, "hello:world");
no!(handle_field_len_1, Rule::handle, "hello/1:world/1");
no!(handle_field_len_0, Rule::handle, "hello/1:world/0");

no!(handle_scoped_len0, Rule::handle, "hello::world/0");
no!(handle_scoped_len2, Rule::handle, "hello::world/2");
no!(handle_scoped_name, Rule::handle, "hello::world/0:to");
no!(handle_scoped_name2, Rule::handle, "hello::world/0:to:from");
no!(handle_scoped_name2_len2, Rule::handle, "hello::world/0:to:from/2");
no!(handle_scoped_name_len2_name, Rule::handle, "hello::world/0:to/2:from");
no!(handle_scoped_name_len2_name_len2, Rule::handle, "hello::world/0:to/2:from/2");
