//! Implementation of the Lumber @core library, containing important built-in functions required
//! for the language to operate.

use crate::Lumber;
use std::path::PathBuf;

native_function! {
    fn add(lhs, rhs, out) {
        use crate::Value::*;
        match (lhs, rhs, out) {
            (Some(Integer(lhs)), Some(Integer(rhs)), None)   => answer![lhs, rhs, lhs + rhs],
            (Some(Integer(lhs)), Some(Rational(rhs)), None)  => answer![lhs, rhs, lhs + rhs],
            (Some(Rational(lhs)), Some(Integer(rhs)), None)  => answer![lhs, rhs, lhs + rhs],
            (Some(Rational(lhs)), Some(Rational(rhs)), None) => answer![lhs, rhs, lhs + rhs],
            (Some(String(lhs)), Some(String(rhs)), None)     => answer![lhs, rhs, lhs + &rhs],
            _ => {}
        }
    }
}

native_function! {
    fn sub(lhs, rhs, out) {
        use crate::Value::*;
        match (lhs, rhs, out) {
            (Some(Integer(lhs)), Some(Integer(rhs)), None)   => answer![lhs, rhs, lhs - rhs],
            (Some(Integer(lhs)), Some(Rational(rhs)), None)  => answer![lhs, rhs, ramp::rational::Rational::from(lhs) - rhs],
            (Some(Rational(lhs)), Some(Integer(rhs)), None)  => answer![lhs, rhs, lhs - ramp::rational::Rational::from(rhs)],
            (Some(Rational(lhs)), Some(Rational(rhs)), None) => answer![lhs, rhs, lhs - rhs],
            _ => {}
        }
    }
}

native_function! {
    fn mul(lhs, rhs, out) {
        use crate::Value::*;
        match (lhs, rhs, out) {
            (Some(Integer(lhs)), Some(Integer(rhs)), None)   => answer![lhs, rhs, lhs * rhs],
            (Some(Integer(lhs)), Some(Rational(rhs)), None)  => answer![lhs, rhs, lhs * rhs],
            (Some(Rational(lhs)), Some(Integer(rhs)), None)  => answer![lhs, rhs, lhs * rhs],
            (Some(Rational(lhs)), Some(Rational(rhs)), None) => answer![lhs, rhs, lhs * rhs],
            _ => {}
        }
    }
}

native_function! {
    fn div(lhs, rhs, out) {
        use crate::Value::*;
        match (lhs, rhs, out) {
            (Some(Integer(lhs)), Some(Integer(rhs)), None)   => answer![lhs, rhs, lhs / rhs],
            (Some(Integer(lhs)), Some(Rational(rhs)), None)  => answer![lhs, rhs, lhs / rhs],
            (Some(Rational(lhs)), Some(Integer(rhs)), None)  => answer![lhs, rhs, lhs / rhs],
            (Some(Rational(lhs)), Some(Rational(rhs)), None) => answer![lhs, rhs, lhs / rhs],
            _ => {}
        }
    }
}

native_function! {
    fn rem(lhs, rhs, out) {
        use crate::Value::*;
        match (lhs, rhs, out) {
            (Some(Integer(lhs)), Some(Integer(rhs)), None) => answer![lhs, rhs, lhs % rhs],
            _ => {}
        }
    }
}

native_function! {
    fn bitor(lhs, rhs, out) {
        use crate::Value::*;
        match (lhs, rhs, out) {
            (Some(Integer(lhs)), Some(Integer(rhs)), None) => answer![lhs, rhs, lhs | rhs],
            _ => {}
        }
    }
}

native_function! {
    fn bitand(lhs, rhs, out) {
        use crate::Value::*;
        match (lhs, rhs, out) {
            (Some(Integer(lhs)), Some(Integer(rhs)), None) => answer![lhs, rhs, lhs & rhs],
            _ => {}
        }
    }
}

native_function! {
    fn bitxor(lhs, rhs, out) {
        use crate::Value::*;
        match (lhs, rhs, out) {
            (Some(Integer(lhs)), Some(Integer(rhs)), None) => answer![lhs, rhs, lhs ^ rhs],
            _ => {}
        }
    }
}

native_function! {
    fn leq(lhs, rhs) {
        use crate::Value::*;
        match (lhs, rhs) {
            (Some(Integer(lhs)), Some(Integer(rhs)))   if lhs <= rhs => answer![lhs, rhs],
            (Some(Rational(lhs)), Some(Rational(rhs))) if lhs <= rhs => answer![lhs, rhs],
            (Some(String(lhs)), Some(String(rhs)))     if lhs <= rhs => answer![lhs, rhs],
            _ => {}
        }
    }
}

native_function! {
    fn geq(lhs, rhs) {
        use crate::Value::*;
        match (lhs, rhs) {
            (Some(Integer(lhs)), Some(Integer(rhs)))   if lhs >= rhs => answer![lhs, rhs],
            (Some(Rational(lhs)), Some(Rational(rhs))) if lhs >= rhs => answer![lhs, rhs],
            (Some(String(lhs)), Some(String(rhs)))     if lhs >= rhs => answer![lhs, rhs],
            _ => {}
        }
    }
}

native_function! {
    fn lt(lhs, rhs) {
        use crate::Value::*;
        match (lhs, rhs) {
            (Some(Integer(lhs)), Some(Integer(rhs)))   if lhs < rhs => answer![lhs, rhs],
            (Some(Rational(lhs)), Some(Rational(rhs))) if lhs < rhs => answer![lhs, rhs],
            (Some(String(lhs)), Some(String(rhs)))     if lhs < rhs => answer![lhs, rhs],
            _ => {}
        }
    }
}

native_function! {
    fn gt(lhs, rhs) {
        use crate::Value::*;
        match (lhs, rhs) {
            (Some(Integer(lhs)), Some(Integer(rhs)))   if lhs > rhs => answer![lhs, rhs],
            (Some(Rational(lhs)), Some(Rational(rhs))) if lhs > rhs => answer![lhs, rhs],
            (Some(String(lhs)), Some(String(rhs)))     if lhs > rhs => answer![lhs, rhs],
            _ => {}
        }
    }
}

thread_local! {
    pub(crate) static LIB: Lumber<'static> = Lumber::builder()
        .core(false)
        .bind("add/3", add)
        .bind("sub/3", sub)
        .bind("mul/3", mul)
        .bind("div/3", div)
        .bind("rem/3", rem)
        .bind("bitor/3", bitor)
        .bind("bitand/3", bitand)
        .bind("bitxor/3", bitxor)
        .bind("leq/2", leq)
        .bind("geq/2", geq)
        .bind("lt/2", lt)
        .bind("gt/2", gt)
        .build(PathBuf::from(file!()).parent().unwrap(), include_str!("core.lumber"))
        .unwrap();
}
