use super::{FromBinding, IntoQuestion, Question};
use std::marker::PhantomData;

/// Provides means to construct any question programmatically. If the question cannot be
/// determined statically or be derived from a struct, this is the recommended way to
/// construct it.
pub struct QuestionBuilder<T> {
    question: Option<Question>,
    _pd: PhantomData<T>,
}

impl<T> QuestionBuilder<T> {
    pub(super) fn new() -> Self {
        Self {
            question: None,
            _pd: PhantomData,
        }
    }
}

impl<T> IntoQuestion for QuestionBuilder<T>
where
    T: FromBinding,
{
    type Answer = T;

    fn into_question(self) -> Question {
        self.question.unwrap()
    }
}
