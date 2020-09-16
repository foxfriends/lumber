use pest::Parser as _;

/// A parser for the Lumber language.
#[derive(pest_derive::Parser)]
#[grammar = "./parser/lumber.pest"]
pub struct Parser;

impl Parser {
    pub fn parse_module<'i>(source_code: &'i str) -> crate::Result<crate::Pairs<'i>> {
        Ok(Self::parse(Rule::module, source_code)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! full_parse {
        ($rule:expr, $src:literal) => {
            assert_eq!(Parser::parse($rule, $src).unwrap().as_str(), $src);
        };
    }

    macro_rules! no_parse {
        ($rule:expr, $src:literal) => {
            let parsed = Parser::parse($rule, $src);
            assert!(parsed.is_err() || parsed.unwrap().as_str() != $src);
        };
    }

    #[test]
    fn parse_fact() {
        full_parse!(Rule::fact, "hello.");
        no_parse!(Rule::fact, "hello().");
        full_parse!(Rule::fact, "hello(world).");
        full_parse!(Rule::fact, "hello(world, again).");
    }

    #[test]
    fn parse_rule() {
        full_parse!(Rule::rule, "hello :- true.");
        no_parse!(Rule::rule, "hello() :- true.");
        full_parse!(Rule::rule, "hello(world) :- true.");
        full_parse!(Rule::rule, "hello(world, again) :- true.");
    }

    #[test]
    fn parse_scope() {
        full_parse!(Rule::scope, "hello");
        full_parse!(Rule::scope, "hello::world");
        full_parse!(Rule::scope, "@lib::hello::world");
        full_parse!(Rule::scope, "~::hello::world");
        full_parse!(Rule::scope, "^::^::hello::world");
        no_parse!(Rule::scope, "^");
        no_parse!(Rule::scope, "^::^");
        no_parse!(Rule::scope, "@lib");
        no_parse!(Rule::scope, "~");
    }

    #[test]
    fn parse_atom() {
        full_parse!(Rule::atom, "hello");
        full_parse!(Rule::atom, "hello_world");
        full_parse!(Rule::atom, "helloWorld");

        no_parse!(Rule::atom, "hello-world");
        no_parse!(Rule::atom, "hello.world");
        no_parse!(Rule::atom, "_hello");
        no_parse!(Rule::atom, "-hello");
        no_parse!(Rule::atom, ".hello");
    }

    #[test]
    fn parse_handle() {
        full_parse!(Rule::handle, "hello/0");
        full_parse!(Rule::handle, "hello/2");

        full_parse!(Rule::handle, "hello:to");
        full_parse!(Rule::handle, "hello:to:from");

        full_parse!(Rule::handle, "hello:to:from/2");
        full_parse!(Rule::handle, "hello:to/2:from");
        full_parse!(Rule::handle, "hello:to/2:from/2");

        no_parse!(Rule::handle, "hello::world/0");
        no_parse!(Rule::handle, "hello::world/2");
        no_parse!(Rule::handle, "hello::world:to");
        no_parse!(Rule::handle, "hello::world:to:from");
        no_parse!(Rule::handle, "hello::world:to:from/2");
        no_parse!(Rule::handle, "hello::world:to/2:from");
        no_parse!(Rule::handle, "hello::world:to/2:from/2");
    }
}
