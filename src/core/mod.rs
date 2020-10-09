//! Implementation of the Lumber @core library, containing important built-in functions required
//! for the language to operate.

use crate::{Lumber, Value};

fn nop3(values: Vec<Option<Value>>) -> Vec<Vec<Option<Value>>> {
    vec![]
}

thread_local! {
    pub(crate) static LIB: Lumber<'static> = Lumber::builder()
        .core(false)
        .bind("add/3", nop3)
        .bind("sub/3", nop3)
        .bind("mul/3", nop3)
        .bind("div/3", nop3)
        .bind("rem/3", nop3)
        .bind("exp/3", nop3)
        .bind("eq/3", nop3)
        .bind("neq/3", nop3)
        .bind("lt/3", nop3)
        .bind("gt/3", nop3)
        .bind("leq/3", nop3)
        .bind("geq/3", nop3)
        .bind("or/3", nop3)
        .bind("and/3", nop3)
        .bind("dif/3", nop3)
        .bind("bitor/3", nop3)
        .bind("bitand/3", nop3)
        .bind("bitxor/3", nop3)
        .build_from_str(include_str!("core.lumber"))
        .unwrap();
}
