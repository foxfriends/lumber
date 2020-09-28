//! Handles parsing of Lumber source files.
use pest::Parser as _;

/// A PEG parser for the Lumber language.
#[derive(pest_derive::Parser)]
#[grammar = "./parser/lumber.pest"]
pub(crate) struct Parser;

impl Parser {
    pub fn parse_module<'i>(source_code: &'i str) -> crate::Result<crate::Pairs<'i>> {
        Ok(Self::parse(Rule::module, source_code)?)
    }

    pub fn parse_handle<'i>(source_code: &'i str) -> crate::Result<crate::Pairs<'i>> {
        Ok(Self::parse(Rule::external_handle, source_code)?)
    }

    pub fn parse_question<'i>(source_code: &'i str) -> crate::Result<crate::Pairs<'i>> {
        Ok(Self::parse(Rule::question, source_code)?)
    }
}

#[cfg(test)]
mod test;
