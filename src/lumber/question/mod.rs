use crate::ast::*;
use crate::parser::*;
use crate::{Binding, Value};
use std::collections::BTreeMap;
use std::convert::TryFrom;

mod builder;
pub use builder::QuestionBuilder;

/// A question ready to be asked to the Lumber program.
///
/// These can be constructed from strings using Question::from() or manually using the
/// [`QuestionBuilder`][].
pub struct Question {
    body: Body,
    variables: Vec<String>,
}

impl AsRef<Body> for Question {
    fn as_ref(&self) -> &Body {
        &self.body
    }
}

impl Question {
    /// Start building a new question, using the [`QuestionBuilder`][]. The type of answers must be
    /// provided. If dynamic bindings are desired, use [`Binding`][] as the `Answer` type.
    pub fn new() -> QuestionBuilder {
        QuestionBuilder::new()
    }

    /// Uses a binding to extract the answer to this question.
    pub fn answer(&self, binding: &Binding) -> Option<BTreeMap<&str, Option<Value>>> {
        self.body
            .identifiers()
            .map(|identifier| {
                Some((
                    self.variables[Into::<usize>::into(identifier)].as_str(),
                    binding.extract(binding.get(identifier)?).ok()?,
                ))
            })
            .collect()
    }
}

impl TryFrom<&str> for Question {
    type Error = crate::Error;

    /// A string using Lumber syntax can be converted directly into a question. It is not recommended
    /// to construct questions dynamically in this way, as the error will not be recoverable. Instead,
    /// for dynamically constructed questions, use the [`QuestionBuilder`][]
    ///
    /// For one-off statically determined questions, however, string conversions should be fine.
    ///
    /// # Errors
    ///
    /// Will return an error if the syntax is invalid.
    fn try_from(src: &str) -> crate::Result<Question> {
        let mut pairs = Parser::parse_question(src)?;
        let pair = pairs.next().unwrap();
        assert_eq!(Rule::question, pair.as_rule());
        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        let mut context = Context::default();
        let body = Body::new(pair, &mut context).unwrap();
        Ok(Question {
            body,
            variables: context.variables,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn question_from_str_single() {
        Question::try_from("hello(A)").unwrap();
    }

    #[test]
    fn question_from_str_scoped() {
        Question::try_from("hello::world(A)").unwrap();
    }

    #[test]
    #[should_panic]
    fn question_from_str_parent() {
        Question::try_from("^::hello(A)").unwrap();
    }

    #[test]
    #[should_panic]
    fn question_from_str_punctuated() {
        Question::try_from("hello(A).").unwrap();
    }

    #[test]
    fn question_from_str_multi() {
        Question::try_from(
            "hello(A) -> hello(B), hello(C); hello(C), hello(D) -> hello(E), F <- 3",
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn question_empty() {
        Question::try_from("").unwrap();
    }
}
