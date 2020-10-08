use super::Question;

/// Provides means to construct any question programmatically. If the question cannot be
/// determined statically or be derived from a struct, this is the recommended way to
/// construct it.
pub struct QuestionBuilder {
    question: Option<Question>,
}

impl QuestionBuilder {
    pub(super) fn new() -> Self {
        Self { question: None }
    }
}

impl From<QuestionBuilder> for Question {
    fn from(builder: QuestionBuilder) -> Self {
        builder.question.unwrap()
    }
}
