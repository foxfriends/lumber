use super::*;
use crate::parser::Rule;

pub(crate) fn fields(pair: crate::Pair, context: &mut Context) -> (Vec<Arity>, Vec<Pattern>) {
    assert_eq!(pair.as_rule(), Rule::fields);
    pair.into_inner().map(|pair| field(pair, context)).fold(
        (vec![], vec![]),
        |(mut arity, mut patterns), (name, pattern)| {
            match name {
                Some(name) => arity.push(Arity::Name(name)),
                None => {
                    if let Some(Arity::Len(len)) = arity.last_mut() {
                        *len += 1;
                    } else {
                        arity.push(Arity::Len(1.into()));
                    }
                }
            }
            patterns.push(pattern);
            (arity, patterns)
        },
    )
}

fn field(pair: crate::Pair, context: &mut Context) -> (Option<Atom>, Pattern) {
    assert_eq!(pair.as_rule(), Rule::field);
    let pair = just!(pair.into_inner());
    match pair.as_rule() {
        Rule::named_field => {
            let mut pairs = pair.into_inner();
            let atom = context.atomizer.atomize(pairs.next().unwrap());
            let pattern = Pattern::new(pairs.next().unwrap(), context);
            (Some(atom), pattern)
        }
        Rule::bare_field => (None, Pattern::new(just!(pair.into_inner()), context)),
        _ => unreachable!(),
    }
}
