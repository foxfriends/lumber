use super::Value;
use crate::ast::*;
use crate::parser::*;
use std::collections::HashMap;

/// A question ready to be asked to the Lumber program.
pub struct Question(Body);

/// Describes a type that can be converted into a question to be asked of the Lumber program,
/// and the shape of the answers that to be expected.
pub trait IntoQuestion {
    /// The type of answers to this query.
    type Answer;

    /// Converts the value into a question.
    fn into_question(self) -> Question;
}

impl IntoQuestion for &str {
    type Answer = HashMap<String, Value>;

    /// A string can be converted into a question. Note that this will panic if the question
    /// is not valid. It is not recommended to construct questions dynamically in this way.
    fn into_question(self) -> Question {
        let mut pairs = Parser::parse_question(self).unwrap();
        let pair = pairs.next().unwrap();
        assert_eq!(Rule::question, pair.as_rule());
        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        Question(Body::new(pair, &mut Context::default()).unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn question_from_str_single() {
        "hello(A)".into_question();
    }

    #[test]
    fn question_from_str_scoped() {
        "hello::world(A)".into_question();
    }

    #[test]
    #[should_panic]
    fn question_from_str_parent() {
        "^::hello(A)".into_question();
    }

    #[test]
    #[should_panic]
    fn question_from_str_punctuated() {
        "hello(A).".into_question();
    }

    #[test]
    fn question_from_str_multi() {
        "hello(A) ! hello(B) ; hello(C) , hello(D) -> hello(E), F <- 3".into_question();
    }
}
