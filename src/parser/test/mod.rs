#![allow(unused_attributes)]
#![rustfmt::skip]

use super::*;

macro_rules! yes {
    ($name:ident, $rule:expr, $src:literal) => {
        #[test]
        fn $name() {
            assert_eq!(Parser::parse($rule, $src).unwrap().as_str(), $src);
        }
    };
}

macro_rules! no {
    ($name:ident, $rule:expr, $src:literal) => {
        #[test]
        fn $name() {
            let parsed = Parser::parse($rule, $src);
            assert!(parsed.is_err() || parsed.unwrap().as_str() != $src);
        }
    };
}

mod aggregation;
mod atom;
mod body;
mod call;
mod directive;
mod evaluation;
mod expression;
mod fact;
mod function;
mod handle;
mod list;
mod literal;
mod multi_handle;
mod operator;
mod predicate;
mod record;
mod rule;
mod scope;
mod r#struct;
mod variable;
mod wildcard;
mod set;
mod trailing_comma;
