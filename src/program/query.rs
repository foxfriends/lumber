use super::*;

#[derive(Clone, Debug)]
pub struct Query {
    /// The shape of this query.
    handle: Handle,
    /// The patterns in each field.
    fields: Vec<Pattern>,
}
