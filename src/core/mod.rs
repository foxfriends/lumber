//! Implementation of the Lumber @core library, containing important built-in functions required
//! for the language to operate.

use crate::Program;

fn nop3() {}

thread_local! {
    #[rustfmt::skip]
    pub(crate) static LIB: Program<'static> = Program::builder()
        .no_core()
        .bind("add/3", nop3).unwrap()
        .bind("sub/3", nop3).unwrap()
        .bind("mul/3", nop3).unwrap()
        .bind("div/3", nop3).unwrap()
        .bind("rem/3", nop3).unwrap()
        .bind("exp/3", nop3).unwrap()
        .bind("eq/3", nop3).unwrap()
        .bind("neq/3", nop3).unwrap()
        .bind("lt/3", nop3).unwrap()
        .bind("gt/3", nop3).unwrap()
        .bind("leq/3", nop3).unwrap()
        .bind("geq/3", nop3).unwrap()
        .bind("or/3", nop3).unwrap()
        .bind("and/3", nop3).unwrap()
        .bind("dif/3", nop3).unwrap()
        .bind("bitor/3", nop3).unwrap()
        .bind("bitand/3", nop3).unwrap()
        .bind("bitxor/3", nop3).unwrap()
        .build_from_str(include_str!("core.lumber"))
        .unwrap();
}
