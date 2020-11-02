use super::*;
use crate::parser::Rule;

pub(crate) fn fields(pair: crate::Pair, context: &mut Context) -> (Arity, Vec<Pattern>) {
    assert_eq!(pair.as_rule(), Rule::fields);
    let mut pairs = pair.into_inner().peekable();
    let mut arity = Arity::default();
    let mut patterns = vec![];
    eprintln!("{:#?}", pairs);
    if pairs.peek().unwrap().as_rule() == Rule::bare_fields {
        patterns.extend(
            pairs
                .next()
                .unwrap()
                .into_inner()
                .map(|pair| Pattern::new(pair, context)),
        );
        arity.len = patterns.len() as u32;
    }
    match pairs.next() {
        Some(pair) => {
            assert_eq!(pair.as_rule(), Rule::named_fields);
            for pair in pair.into_inner() {
                let mut pairs = pair.into_inner();
                let name = Atom::new(pairs.next().unwrap());
                let values = just!(Rule::bare_fields, pairs)
                    .into_inner()
                    .map(|pair| Pattern::new(pair, context))
                    .collect::<Vec<_>>();
                arity.push(name, values.len() as u32);
                patterns.extend(values);
            }
        }
        None => {}
    }
    (arity, patterns)
}
