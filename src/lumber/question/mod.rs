use super::{Binding, FromBinding};
use crate::ast::*;
use crate::parser::*;

mod builder;
pub use builder::QuestionBuilder;

/// A question ready to be asked to the Lumber program.
///
/// These can be constructed from structs or strings using the [`IntoQuestion`][] trait
/// or manually using the [`QuestionBuilder`][].
pub struct Question(Body);

impl Into<Body> for Question {
    fn into(self) -> Body {
        self.0
    }
}

impl Question {
    /// Start building a new question, using the [`QuestionBuilder`][]. The type of answers must be
    /// provided. If dynamic bindings are desired, use [`Binding`][] as the `Answer` type.
    pub fn new<Answer>() -> QuestionBuilder<Answer> {
        QuestionBuilder::new()
    }
}

/// Describes a type that can be converted into a question to be asked of the Lumber program,
/// and the shape of the answers that are to be expected.
pub trait IntoQuestion {
    /// The type of answers to this query.
    type Answer: FromBinding;

    /// Converts the value into a question.
    fn into_question(self) -> Question;
}

impl IntoQuestion for &str {
    type Answer = Binding;

    /// A string using Lumber syntax can be converted directly into a question. It is not recommended
    /// to construct questions dynamically in this way, as it will cause a panic if the syntax is invalid.
    /// Instead, use the derives to build them from a struct.
    ///
    /// For one-off statically determined questions, however, string conversions should be fine.
    ///
    /// # Panics
    ///
    /// Will panic if the syntax is invalid.
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
        "hello(A) -> hello(B), hello(C); hello(C), hello(D) -> hello(E), F <- 3".into_question();
    }

    #[test]
    #[should_panic]
    fn question_empty() {
        "".into_question();
    }
}
