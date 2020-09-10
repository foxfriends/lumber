#[derive(pest_derive::Parser)]
#[grammar = "./parser/lumber.pest"]
pub struct Parser;

#[cfg(test)]
mod test {
    use pest::Parser as _;
    use super::*;

    #[test]
    fn fact_bare() {
        Parser::parse(Rule::fact, "hello.").unwrap();
    }

    #[test]
    #[should_panic]
    fn fact_no_args() {
        Parser::parse(Rule::fact, "hello().").unwrap();
    }

    #[test]
    fn fact_one_arg() {
        Parser::parse(Rule::fact, "hello(world).").unwrap();
    }

    #[test]
    fn fact_two_args() {
        Parser::parse(Rule::fact, "hello(world, again).").unwrap();
    }

    #[test]
    fn rule_bare() {
        Parser::parse(Rule::rule, "hello :- true.").unwrap();
    }

    #[test]
    #[should_panic]
    fn rule_no_args() {
        Parser::parse(Rule::rule, "hello() :- true.").unwrap();
    }

    #[test]
    fn rule_one_arg() {
        Parser::parse(Rule::rule, "hello(world) :- true.").unwrap();
    }

    #[test]
    fn rule_two_args() {
        Parser::parse(Rule::rule, "hello(world, again) :- true.").unwrap();
    }
}
