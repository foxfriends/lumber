use super::{Answer, Value};
use crate::ast::{self, Context};
use crate::parser::*;
use crate::program::evaltree::Body;
use crate::program::Binding;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

/// A question ready to be asked to the Lumber program.
///
/// These can be constructed from strings using [`Question::try_from`][].
#[derive(Clone, Debug)]
pub struct Question {
    body: Body,
    pub(crate) initial_binding: Binding,
}

impl Question {
    pub(crate) fn new(body: impl Into<Body>) -> Self {
        let body = body.into();
        let initial_binding = Binding::new(&body);
        Self {
            body,
            initial_binding,
        }
    }

    /// Sets the value of a variable before unification begins.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::{Value, Question};
    /// # use std::convert::TryFrom;
    /// let mut question = Question::try_from("greeting(A, B)").unwrap();
    /// question.set("A", Value::from("hello"));
    /// ```
    ///
    /// # Panics
    ///
    /// If the variable being set is not referenced by the question.
    pub fn set(&mut self, variable: &str, value: Value) {
        self.initial_binding.bind(variable, value);
    }

    /// Sets the value of a variable before unification begins.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lumber::{Value, Question};
    /// # use std::convert::TryFrom;
    /// let question = Question::try_from("greeting(A, B)").unwrap()
    ///     .with("A", Value::from("hello"));
    /// ```
    ///
    /// # Panics
    ///
    /// If the variable being set is not referenced by the question.
    pub fn with(mut self, variable: &str, value: Value) -> Self {
        self.set(variable, value);
        self
    }

    /// Uses a binding to extract the answer to this question.
    pub(crate) fn answer(&self, binding: &Binding) -> Answer {
        self.body
            .variables(0)
            .filter(|variable| !variable.is_wildcard())
            .map(|variable| {
                (
                    variable.name().to_owned(),
                    binding.extract(&binding.get(&variable).unwrap()).unwrap(),
                )
            })
            .collect()
    }
}

impl TryFrom<&str> for Question {
    type Error = crate::Error;

    /// A string using Lumber syntax can be converted directly into a question. It is not recommended
    /// to construct questions dynamically in this way, as the error will not be recoverable. There is
    /// not currently another method of constructing questions, but it is a planned feature to have
    /// some sort of question builder, DSL, or derive-based solution for this problem.
    ///
    /// For one-off statically written questions, string conversions should be fine and unwrapped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumber::Question;
    /// use std::convert::TryInto;
    ///
    /// let question: Question = "test(A)".try_into().unwrap();
    /// ```
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
        match ast::Body::new(pair, &mut context) {
            Some(body) => Ok(Self::new(Body::from(body))),
            None => Err(crate::Error::parse(&format!(
                "invalid syntax in question: {}",
                src
            ))),
        }
    }
}

impl Display for Question {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.body.fmt(f)
    }
}

impl AsRef<Body> for Question {
    fn as_ref(&self) -> &Body {
        &self.body
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
    fn question_from_str_parent() {
        assert!(Question::try_from("^::hello(A)").is_err());
    }

    #[test]
    fn question_from_str_punctuated() {
        assert!(Question::try_from("hello(A).").is_err());
    }

    #[test]
    fn question_from_str_multi() {
        Question::try_from(
            "hello(A) -> hello(B), hello(C); hello(C), hello(D) -> hello(E), F =:= 3",
        )
        .unwrap();
    }

    #[test]
    fn question_empty() {
        assert!(Question::try_from("").is_err());
    }
}
