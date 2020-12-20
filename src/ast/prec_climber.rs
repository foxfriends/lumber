//! [`PrecClimber`] implementation stolen and adapted from [`pest::prec_climber::PrecClimber`][]
//! so that it will can easily support user-defined operators, given that the operators are
//! declared before use.
use super::{Associativity, Operator};
use pest::{iterators::Pair, RuleType};
use std::iter::Peekable;

#[derive(Debug)]
pub(crate) struct PrecClimber;

