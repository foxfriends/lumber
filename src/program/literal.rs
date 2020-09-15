use crate::parser::Rule;
use ramp::{int::Int, rational::Rational};

/// A literal value, which cannot be further pattern matched.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Literal {
    /// A single integer.
    Integer(Int),
    /// A fractional number.
    Rational(Rational),
    /// A string.
    String(String),
}

impl Literal {
    pub(crate) fn new(pair: crate::Pair) -> Self {
        assert_eq!(pair.as_rule(), Rule::literal);
        let pair = just!(pair.into_inner());
        match pair.as_rule() {
            Rule::integer => {
                let pair = just!(pair.into_inner());
                match pair.as_rule() {
                    Rule::integer_10 => {
                        Self::Integer(Int::from_str_radix(pair.as_str(), 10).unwrap())
                    }
                    Rule::integer_2 => {
                        Self::Integer(Int::from_str_radix(&pair.as_str()[2..], 2).unwrap())
                    }
                    Rule::integer_16 => {
                        Self::Integer(Int::from_str_radix(&pair.as_str()[2..], 10).unwrap())
                    }
                    _ => unreachable!(),
                }
            }
            Rule::decimal => {
                let mut parts = pair.as_str().split('.');
                let unit = parts.next().unwrap();
                let fractional = parts.next().unwrap();
                let denominator = Int::from(10).pow(fractional.len());
                let numerator = format!("{}{}", unit, fractional).parse().unwrap();
                Literal::Rational(Rational::new(numerator, denominator))
            }
            Rule::string => {
                let string = pair.as_str().trim_matches('#');
                Self::String(string[1..string.len() - 1].to_owned())
            }
            _ => unreachable!(),
        }
    }
}
