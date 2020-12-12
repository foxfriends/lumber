use super::*;
use crate::parser::Rule;

/// An alias to expose a handle in a different scope.
#[derive(Clone, Debug)]
pub(crate) enum Alias {
    Predicate {
        /// The original handle, in its source scope.
        input: Handle,
        /// The exposed handle, in the new scope.
        output: Handle,
    },
    Operator {
        /// The name of the imported operator.
        name: Atom,
        /// The scope from which the operator is imported.
        scope: Scope,
    },
}

impl Alias {
    #[allow(clippy::needless_collect)]
    pub fn unpack_multiple(
        pair: crate::Pair,
        context: &mut Context,
    ) -> Result<impl Iterator<Item = Alias>, Scope> {
        assert_eq!(pair.as_rule(), Rule::multi_handle);
        let mut pairs = pair.into_inner();
        let scope = match Scope::new_module_path(pairs.next().unwrap(), context) {
            Some(scope) => scope,
            None => return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Alias>>),
        };
        match pairs.next() {
            None => Err(scope),
            Some(pair) => {
                let aliases = pair
                    .into_inner()
                    .map(|pair| match pair.as_rule() {
                        Rule::handle => {
                            let input = Handle::new_in_scope(scope.clone(), pair.clone());
                            let output = Handle::new(pair, context);
                            Alias::Predicate { input, output }
                        }
                        Rule::alias => {
                            let mut pairs = pair.into_inner();
                            let input = Handle::new_in_scope(scope.clone(), pairs.next().unwrap());
                            let output = Handle::new(pairs.next().unwrap(), context);
                            if !output.can_alias(&input) {
                                context.error_invalid_alias_arity(&input, &output);
                            }
                            Alias::Predicate { input, output }
                        }
                        Rule::operator => Alias::Operator {
                            name: Atom::from(pair.as_str()),
                            scope: scope.clone(),
                        },
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>();
                Ok(Box::new(aliases.into_iter()))
            }
        }
    }
}
