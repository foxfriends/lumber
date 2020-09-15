use super::*;
use crate::parser::Rule;

/// An alias to expose a handle in a different scope.
#[derive(Clone, Debug)]
pub struct Alias {
    /// The original handle, in its source scope.
    pub(crate) input: Handle,
    /// The exposed handle, in the new scope.
    pub(crate) output: Handle,
}

impl Alias {
    pub(crate) fn unpack_multiple(
        pair: crate::Pair,
        context: &mut Context,
    ) -> Result<impl Iterator<Item = Alias>, Scope> {
        assert_eq!(pair.as_rule(), Rule::multi_handle);
        let mut pairs = pair.into_inner();
        let scope = match Scope::new(pairs.next().unwrap(), context) {
            Some(scope) => scope,
            None => return Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Alias>>),
        };
        match pairs.next() {
            None => Err(scope),
            Some(pair) => {
                let pair = just!(Rule::handles, pair.into_inner());
                let aliases = pair
                    .into_inner()
                    .map(|pair| match pair.as_rule() {
                        Rule::handle => {
                            let input = Handle::new_in_scope(scope.clone(), pair.clone(), context);
                            let output = Handle::new(pair.clone(), context);
                            Alias { input, output }
                        }
                        Rule::alias => {
                            let mut pairs = pair.into_inner();
                            let input =
                                Handle::new_in_scope(scope.clone(), pairs.next().unwrap(), context);
                            let output =
                                Handle::new_in_scope(scope.clone(), pairs.next().unwrap(), context);
                            Alias { input, output }
                        }
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>();
                Ok(Box::new(aliases.into_iter()))
            }
        }
    }
}
