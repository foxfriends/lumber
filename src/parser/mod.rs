use pest::Parser as _;
#[cfg(test)]
mod test;

/// A parser for the Lumber language.
#[derive(pest_derive::Parser)]
#[grammar = "./parser/lumber.pest"]
pub struct Parser;

impl Parser {
    pub fn parse_module<'i>(source_code: &'i str) -> crate::Result<crate::Pairs<'i>> {
        Ok(Self::parse(Rule::module, source_code)?)
    }
}
