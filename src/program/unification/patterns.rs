use crate::ast::*;
use crate::Binding;
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

// TODO: unification of sets will end up being a big change, as set unification is not deterministic.
// TODO: This function could be wrapped so it does not return the output pattern, as that is only really
//       used internally.
pub(crate) fn unify_patterns(
    lhs: &Pattern,
    rhs: &Pattern,
    binding: Binding,
    occurs: &[Identifier],
) -> Option<(Pattern, Binding)> {
    match (lhs, rhs) {
        // Any values must be the exact same value. We know nothing else about them.
        (Pattern::Any(lhs), Pattern::Any(rhs)) if Rc::ptr_eq(lhs, rhs) => {
            Some((Pattern::Any(lhs.clone()), binding))
        }
        // The "bound" pattern requires the other value to already be bound, so this is the only way
        // a wildcard unification will fail.
        (Pattern::Wildcard, Pattern::Bound(..)) | (Pattern::Bound(..), Pattern::Wildcard) => None,
        // The "unbound" pattern requires the other value is not bound. That is: it will only unify
        // with a wildcard (or a variable that has resolved to a wildcard).
        (Pattern::Wildcard, Pattern::Unbound(pattern))
        | (Pattern::Unbound(pattern), Pattern::Wildcard) => Some(((**pattern).clone(), binding)),
        // Unifying wildcards provides no additional info. It is at this point that an explicit
        // occurs check must be made (it will be caught recursively in other cases).
        (Pattern::Wildcard, other) | (other, Pattern::Wildcard)
            if !other.identifiers().any(|id| occurs.contains(&id)) =>
        {
            Some((other.clone(), binding))
        }
        // Unifying a x with itself succeeds with no additional info.
        (Pattern::Variable(lhs), Pattern::Variable(rhs)) if lhs == rhs => {
            // We don't need to use occurs check here because `A <- A` is allowed, despite
            // `A` being in the occurs list already.
            Some((Pattern::Variable(lhs.clone()), binding))
        }
        // Unifying a x with a different x, we use the natural order of variables
        // to designate one as the source of truth and the other as a reference.
        (Pattern::Variable(lhs), Pattern::Variable(rhs)) => {
            if occurs.contains(lhs) || occurs.contains(rhs) {
                return None;
            }
            let lhs_pat = binding.get(lhs).unwrap().clone();
            let rhs_pat = binding.get(rhs).unwrap().clone();
            let mut occurs = occurs.to_owned();
            occurs.push(lhs.clone());
            occurs.push(rhs.clone());
            let (pattern, mut binding) = unify_patterns(&lhs_pat, &rhs_pat, binding, &occurs)?;
            let min = Identifier::min(lhs.clone(), rhs.clone());
            let max = Identifier::max(lhs.clone(), rhs.clone());
            binding.set(min.clone(), pattern.clone());
            binding.set(max, Pattern::Variable(min));
            Some((pattern, binding))
        }
        // A x unified with a value should attempt to dereference the x and then
        // unify. If that succeeds, the x is replaced with the binding.
        (Pattern::Variable(var), pattern) | (pattern, Pattern::Variable(var)) => {
            if occurs.contains(var) {
                return None;
            }
            let var_pat = binding.get(var).unwrap().clone();
            let mut occurs = occurs.to_owned();
            occurs.push(var.clone());
            let (pattern, mut binding) = unify_patterns(&var_pat, pattern, binding, &occurs)?;
            match pattern {
                Pattern::Variable(new_var) => unify_patterns(
                    &Pattern::Variable(var.clone()),
                    &Pattern::Variable(new_var),
                    binding,
                    &occurs,
                ),
                _ => {
                    binding.set(var.clone(), pattern.clone());
                    Some((pattern, binding))
                }
            }
        }
        // If not with a variable, a "bound" pattern unifies normally
        (lhs, Pattern::Bound(rhs)) => unify_patterns(lhs, rhs, binding, occurs),
        (Pattern::Bound(lhs), rhs) => unify_patterns(lhs, rhs, binding, occurs),
        // Literals must match exactly.
        (Pattern::Literal(lhs), Pattern::Literal(rhs)) if lhs == rhs => {
            Some((Pattern::Literal(lhs.clone()), binding.clone()))
        }
        (Pattern::Literal(..), Pattern::Literal(..)) => None,
        // Structs must match in name, and then their contents must match
        (Pattern::Struct(lhs), Pattern::Struct(rhs))
            if lhs.name == rhs.name && lhs.contents.is_none() && rhs.contents.is_none() =>
        {
            Some((Pattern::Struct(lhs.clone()), binding))
        }
        (Pattern::Struct(lhs), Pattern::Struct(rhs))
            if lhs.name == rhs.name && lhs.contents.is_some() && rhs.contents.is_some() =>
        {
            let (contents, binding) = unify_patterns(
                lhs.contents.as_ref().unwrap(),
                rhs.contents.as_ref().unwrap(),
                binding,
                &occurs,
            )?;
            Some((
                Pattern::Struct(Struct {
                    name: lhs.name.clone(),
                    contents: Some(Box::new(contents)),
                }),
                binding,
            ))
        }
        (Pattern::Struct(..), Pattern::Struct(..)) => None,
        // If neither list has a tail, the heads must match.
        (Pattern::List(lhs, None), Pattern::List(rhs, None)) => {
            let (fields, binding) = unify_sequence(&lhs, &rhs, binding, occurs)?;
            Some((Pattern::List(fields, None), binding))
        }
        // If only one list has a tail, the tail unifies with whatever the head does
        // not already cover.
        (other @ Pattern::List(full, None), Pattern::List(head, Some(tail)))
        | (Pattern::List(head, Some(tail)), other @ Pattern::List(full, None)) => {
            match tail.as_ref() {
                Pattern::Variable(ident) => {
                    let (output, tail, binding) = unify_full_prefix(head, full, binding, occurs)?;
                    let tail_pat = binding.get(ident).unwrap().clone();
                    let mut occurs = occurs.to_owned();
                    occurs.push(ident.clone());
                    let (tail, mut binding) =
                        unify_patterns(&Pattern::List(tail, None), &tail_pat, binding, &occurs)?;
                    binding.set(ident.clone(), tail.clone());
                    Some((Pattern::List(output, Some(Box::new(tail))), binding))
                }
                Pattern::Wildcard => {
                    let (mut output, mut tail, binding) =
                        unify_full_prefix(head, full, binding, occurs)?;
                    output.append(&mut tail);
                    Some((Pattern::List(output, None), binding))
                }
                Pattern::List(cont, tail) => {
                    let mut combined = head.to_owned();
                    combined.extend_from_slice(&cont);
                    let lhs = Pattern::List(combined, tail.clone());
                    unify_patterns(&lhs, other, binding, occurs)
                }
                // If the tail cannot unify with a list, then there is a problem.
                _ => None,
            }
        }
        // If both lists have tails, unify the prefixes of the heads, then we'll have
        // one list and one pattern, which can be unified.
        (Pattern::List(lhs, Some(lhs_tail)), Pattern::List(rhs, Some(rhs_tail))) => {
            let (unified, remaining, binding) = unify_prefix(lhs, rhs, binding, occurs)?;
            // The shorter one is the one that is now "done", so we match it's tail with
            // the remaining head and tail of the other list.
            let (suffix, binding) = if lhs.len() < rhs.len() {
                unify_patterns(
                    lhs_tail.as_ref(),
                    &Pattern::List(remaining, Some(rhs_tail.clone())),
                    binding,
                    occurs,
                )?
            } else {
                unify_patterns(
                    &Pattern::List(remaining, Some(lhs_tail.clone())),
                    rhs_tail.as_ref(),
                    binding,
                    occurs,
                )?
            };
            Some((Pattern::List(unified, Some(Box::new(suffix))), binding))
        }
        // If neither record has a tail, the heads must match like a struct
        (Pattern::Record(lhs, None), Pattern::Record(rhs, None)) => {
            let (fields, binding) = unify_fields(&lhs, &rhs, binding, occurs)?;
            Some((Pattern::Record(fields, None), binding))
        }
        // If only one record has a tail, the tail unifies with whatever the head does
        // not already cover.
        (other @ Pattern::Record(full, None), Pattern::Record(head, Some(tail)))
        | (Pattern::Record(head, Some(tail)), other @ Pattern::Record(full, None)) => {
            match tail.as_ref() {
                Pattern::Variable(ident) => {
                    let (output, tail, binding) =
                        unify_fields_partial(head, full, binding, occurs)?;
                    let tail_pat = binding.get(ident).unwrap().clone();
                    let mut occurs = occurs.to_owned();
                    occurs.push(ident.clone());
                    let (tail, mut binding) =
                        unify_patterns(&Pattern::Record(tail, None), &tail_pat, binding, &occurs)?;
                    binding.set(ident.clone(), tail.clone());
                    Some((Pattern::Record(output, Some(Box::new(tail))), binding))
                }
                Pattern::Wildcard => {
                    let (mut output, mut tail, binding) =
                        unify_fields_partial(head, full, binding, occurs)?;
                    output.append(&mut tail);
                    Some((Pattern::Record(output, None), binding))
                }
                Pattern::Record(cont, tail) => {
                    let mut combined = head.clone();
                    combined.append(&mut cont.clone());
                    let lhs = Pattern::Record(combined, tail.clone());
                    unify_patterns(&lhs, other, binding, occurs)
                }
                // If the tail cannot unify with a record, then there is a problem.
                _ => None,
            }
        }
        // If both records have tails, unify the heads to remove common elements of both, then
        // a record formed from the remaining elements of the other is unified with each tail in
        // turn.
        (Pattern::Record(lhs, Some(lhs_tail)), Pattern::Record(rhs, Some(rhs_tail))) => {
            let (intersection, mut lhs_rest, mut rhs_rest, mut binding) =
                unify_fields_difference(lhs, rhs, binding, occurs)?;
            let unknown_tail = binding.fresh_variable();
            let new_rhs_tail = Pattern::Record(
                lhs_rest.clone(),
                Some(Box::new(Pattern::Variable(unknown_tail.clone()))),
            );
            let new_lhs_tail = Pattern::Record(
                rhs_rest.clone(),
                Some(Box::new(Pattern::Variable(unknown_tail.clone()))),
            );
            lhs_rest.append(&mut rhs_rest);
            let out_tail =
                Pattern::Record(lhs_rest, Some(Box::new(Pattern::Variable(unknown_tail))));
            let (_, binding) = unify_patterns(lhs_tail, &new_lhs_tail, binding, occurs)?;
            let (_, binding) = unify_patterns(rhs_tail, &new_rhs_tail, binding, occurs)?;
            Some((
                Pattern::Record(intersection, Some(Box::new(out_tail))),
                binding,
            ))
        }
        // Otherwise, it's a failure!
        _ => None,
    }
}

fn unify_sequence(
    lhs: &[Pattern],
    rhs: &[Pattern],
    binding: Binding,
    occurs: &[Identifier],
) -> Option<(Vec<Pattern>, Binding)> {
    if lhs.len() != rhs.len() {
        return None;
    }
    lhs.iter()
        .zip(rhs.iter())
        .try_fold((vec![], binding), |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns(lhs, rhs, binding, occurs)?;
            patterns.push(pattern);
            Some((patterns, binding))
        })
}

fn unify_fields(
    lhs: &Fields,
    rhs: &Fields,
    binding: Binding,
    occurs: &[Identifier],
) -> Option<(Fields, Binding)> {
    if lhs.len() != rhs.len() {
        return None;
    }
    let (fields, binding) = lhs.iter().zip(rhs.iter()).try_fold(
        (BTreeMap::default(), binding),
        |(mut fields, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns(&lhs.1, &rhs.1, binding, occurs)?;
            fields.insert(lhs.0.clone(), pattern);
            Some((fields, binding))
        },
    )?;
    Some((Fields::from(fields), binding))
}

fn unify_fields_partial(
    part: &Fields,
    full: &Fields,
    binding: Binding,
    occurs: &[Identifier],
) -> Option<(Fields, Fields, Binding)> {
    let mut full: BTreeMap<_, _> = full.clone().into();
    let (fields, binding) = part.iter().try_fold(
        (BTreeMap::new(), binding),
        |(mut fields, binding), (key, pattern)| {
            let (unified, binding) =
                unify_patterns(&pattern, &full.remove(&key)?, binding, occurs)?;
            fields.insert(key.clone(), unified);
            Some((fields, binding))
        },
    )?;
    Some((fields.into(), full.into(), binding))
}

fn unify_fields_difference(
    lhs: &Fields,
    rhs: &Fields,
    binding: Binding,
    occurs: &[Identifier],
) -> Option<(Fields, Fields, Fields, Binding)> {
    let mut lhs: BTreeMap<_, _> = lhs.clone().into();
    let mut rhs: BTreeMap<_, _> = rhs.clone().into();
    let all_keys: HashSet<_> = lhs.keys().chain(rhs.keys()).cloned().collect();

    let (intersection, lhs_rest, rhs_rest, binding) = all_keys.into_iter().try_fold(
        (BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), binding),
        |(mut intersection, mut lhs_rest, mut rhs_rest, binding), key| match (
            lhs.remove(&key),
            rhs.remove(&key),
        ) {
            (Some(lhs), Some(rhs)) => {
                let (patterns, binding) = unify_patterns(&lhs, &rhs, binding, occurs)?;
                intersection.insert(key, patterns);
                Some((intersection, lhs_rest, rhs_rest, binding))
            }
            (Some(dif), None) => {
                lhs_rest.insert(key, dif);
                Some((intersection, lhs_rest, rhs_rest, binding))
            }
            (None, Some(dif)) => {
                rhs_rest.insert(key, dif);
                Some((intersection, lhs_rest, rhs_rest, binding))
            }
            _ => unreachable!(),
        },
    )?;
    Some((
        intersection.into(),
        lhs_rest.into(),
        rhs_rest.into(),
        binding,
    ))
}

fn unify_prefix(
    lhs: &[Pattern],
    rhs: &[Pattern],
    binding: Binding,
    occurs: &[Identifier],
) -> Option<(Vec<Pattern>, Vec<Pattern>, Binding)> {
    let (head, binding) = lhs.iter().zip(rhs.iter()).try_fold(
        (vec![], binding),
        |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns(lhs, rhs, binding, occurs)?;
            patterns.push(pattern);
            Some((patterns, binding))
        },
    )?;
    if lhs.len() < rhs.len() {
        Some((head, rhs[lhs.len()..].to_owned(), binding))
    } else {
        Some((head, lhs[rhs.len()..].to_owned(), binding))
    }
}

fn unify_full_prefix(
    lhs: &[Pattern],
    rhs: &[Pattern],
    binding: Binding,
    occurs: &[Identifier],
) -> Option<(Vec<Pattern>, Vec<Pattern>, Binding)> {
    if lhs.len() > rhs.len() {
        return None;
    }
    let (head, binding) = lhs.iter().zip(rhs.iter()).try_fold(
        (vec![], binding),
        |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns(lhs, rhs, binding, occurs)?;
            patterns.push(pattern);
            Some((patterns, binding))
        },
    )?;
    Some((head, rhs[lhs.len()..].to_owned(), binding))
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! yes {
        ($lhs:expr, $rhs:expr $(,)?) => {
            assert!(unify_patterns(&$lhs, &$rhs, Binding::default(), &[]).is_some())
        };
        ($lhs:expr, $rhs:expr, $binding:expr $(,)?) => {{
            let output = unify_patterns(&$lhs, &$rhs, $binding.clone(), &[]);
            assert!(output.is_some());
            output.unwrap()
        }};
    }

    macro_rules! no {
        ($lhs:expr, $rhs:expr $(,)?) => {
            assert!(unify_patterns(&$lhs, &$rhs, Binding::default(), &[]).is_none())
        };
        ($lhs:expr, $rhs:expr, $binding:expr $(,)?) => {
            assert!(unify_patterns(&$lhs, &$rhs, $binding.clone(), &[]).is_none())
        };
    }

    fn atom(name: &str) -> Pattern {
        Pattern::Struct(Struct::from_parts(Atom::from(name), None))
    }

    fn id(name: &str, binding: &mut Binding) -> Pattern {
        let identifier = Identifier::new(name.to_owned());
        binding.0.insert(identifier.clone(), Pattern::Wildcard);
        Pattern::Variable(identifier)
    }

    fn int(val: impl Into<ramp::int::Int>) -> Pattern {
        Pattern::Literal(Literal::Integer(val.into()))
    }

    fn rat(val: impl Into<ramp::rational::Rational>) -> Pattern {
        Pattern::Literal(Literal::Rational(val.into()))
    }

    fn string(val: impl Into<String>) -> Pattern {
        Pattern::Literal(Literal::String(val.into()))
    }

    const WILD: Pattern = Pattern::Wildcard;

    macro_rules! list {
        () => (Pattern::List(vec![], None));
        ($($item:expr),+) => (Pattern::List(vec![$($item.clone()),+], None));
        ($($item:expr),+ ; $rest:expr) => (Pattern::List(vec![$($item.clone()),+], Some(Box::new($rest.clone()))));
    }

    macro_rules! structure {
        (
            $name:ident ($contents:expr)
        ) => {
            Pattern::Struct(Struct::from_parts(
                Atom::from(stringify!($name)),
                Some(Box::new($contents.clone())),
            ))
        };
    }

    macro_rules! record {
        (
            @ [$fields:ident] (.. $rest:expr)
        ) => {{
            Pattern::Record($fields.into(), Some(Box::new($rest.clone())))
        }};

        (
            @ [$fields:ident] ( $fieldname:ident: $pat:expr $(, $($field:tt)+)? )
        ) => {{
            $fields.insert(Atom::from(stringify!($fieldname)), $pat.clone());
            record!(@ [$fields] ($($($field)+)?))
        }};

        (
            @ [$fields:ident] ()
        ) => {{
            Pattern::Record($fields.into(), None)
        }};

        ($($field:tt)+) => {{
            #[allow(unused_mut)]
            let mut fields = BTreeMap::default();
            record!(@[fields] ($($field)+))
        }};

        () => { Pattern::Record(Default::default(), None) }
    }

    fn unbound(pattern: Pattern) -> Pattern {
        Pattern::Unbound(Box::new(pattern))
    }

    fn bound(pattern: Pattern) -> Pattern {
        Pattern::Bound(Box::new(pattern))
    }

    #[test]
    fn unify_literal_integer() {
        yes!(int(3), int(3));
    }

    #[test]
    fn no_unify_literal_integer() {
        no!(int(3), int(4));
    }

    #[test]
    fn unify_literal_rational() {
        yes!(rat(3), rat(3));
    }

    #[test]
    fn no_unify_literal_rational() {
        no!(rat(3), rat(4));
    }

    #[test]
    fn unify_literal_string() {
        yes!(string("hello"), string("hello"));
    }

    #[test]
    fn no_unify_literal_string() {
        no!(string("hello"), string("world"));
    }

    #[test]
    fn unify_atom() {
        yes!(atom("true"), atom("true"));
    }

    #[test]
    fn no_unify_atom() {
        no!(atom("true"), atom("false"));
    }

    #[test]
    fn unify_struct() {
        yes!(
            structure!(hello(atom("world"))),
            structure!(hello(atom("world"))),
        );

        yes!(
            structure!(hello(list![rat(1)])),
            structure!(hello(list![rat(1)])),
        );

        yes!(structure!(hello(WILD)), structure!(hello(atom("hello"))),);

        yes!(
            structure!(hello(record! { a: int(1), b: int(2) })),
            structure!(hello(record! { a: int(1), b: int(2) })),
        );
    }

    #[test]
    fn no_unify_struct() {
        no!(
            structure!(hello(atom("world"))),
            structure!(world(atom("world"))),
        );

        no!(atom("world"), structure!(world(atom("world"))),);
    }

    #[test]
    fn unify_record() {
        yes!(record! {}, record! {});
        yes!(record! { a: int(1) }, record! { a: int(1) });
        yes!(record! { a: int(1) }, record! { a: int(1), ..WILD });
        yes!(
            record! { a: int(1), b: int(2) },
            record! { a: int(1), ..WILD }
        );
        yes!(
            record! { a: int(1), b: int(2), ..WILD },
            record! { a: int(1), ..WILD }
        );
        yes!(
            record! { a: int(1), b: int(2), ..WILD },
            record! { a: int(1), c: int(3), ..WILD }
        );
    }

    #[test]
    fn no_unify_record() {
        no!(record! {}, record! { a: int(1) });
        no!(record! { a: int(1) }, record! { a: int(2) });
        no!(record! { a: int(2) }, record! { a: int(1), ..WILD });
        no!(
            record! { a: int(1), b: int(2), ..WILD },
            record! { a: int(1), b: int(3), ..WILD }
        );
    }

    #[test]
    fn unify_wildcard() {
        yes!(WILD, WILD);
        yes!(WILD, atom("anything"));
        yes!(WILD, int(3));
        yes!(WILD, list![WILD, atom("anything"); WILD]);
    }

    #[test]
    fn unify_variable() {
        let mut binding = Binding::default();
        let x = id("x", &mut binding);
        let y = id("y", &mut binding);
        yes!(x, WILD, binding);
        yes!(x, x, binding);
        yes!(x, y, binding);
        yes!(x, int(3), binding);
        yes!(x, atom("hello"), binding);
        yes!(x, structure!(hello(int(3))), binding);
        yes!(x, list![int(3), int(4)], binding);
        yes!(x, list![y], binding);
    }

    #[test]
    fn unify_multiple_variables() {
        let mut binding = Binding::default();
        let x = id("x", &mut binding);
        let y = id("y", &mut binding);
        yes!(structure!(test(list![x, y])), WILD, binding);
        yes!(
            structure!(test(list![x, y])),
            structure!(test(list![int(3), x])),
            binding
        );
        yes!(
            structure!(test(list![x, y])),
            structure!(test(list![int(3), int(4)])),
            binding
        );
        no!(
            structure!(test(list![x, x])),
            structure!(test(list![int(3), int(4)])),
            binding
        );
        yes!(
            structure!(test(list![int(3), y])),
            structure!(test(list![x, int(4)])),
            binding
        );
        no!(
            structure!(test(list![int(3), x])),
            structure!(test(list![x, int(4)])),
            binding
        );
        yes!(list![x, y], list![int(1), int(2)], binding);
        no!(list![x, x], list![int(1), int(2)], binding);
        yes!(list![x; x], list![list![int(1)], int(1)], binding);
        no!(list![x; x], list![int(1), int(1)], binding);
        yes!(list![int(1); x], list![int(1), int(2); y], binding);

        yes!(
            record! { a: x, b: int(0), ..x },
            record! { c: int(1), d: int(2), ..WILD },
            binding,
        );
        yes!(
            record! { a: x, b: int(0), ..x },
            record! { c: int(1), d: int(2), a: record! { c: int(1), d: int(2) }, ..y },
            binding,
        );
    }

    #[test]
    fn no_unify_variable_occurs() {
        let mut binding = Binding::default();
        let x = id("x", &mut binding);
        no!(x, list![x], binding);
        no!(x, list![int(3); x], binding);
        no!(x, structure!(hello(x)), binding);
        no!(list![int(1); x], list![int(1), int(2); x], binding);
    }

    #[test]
    fn unify_list() {
        yes!(list![int(1), int(2)], list![int(1), int(2)]);
        yes!(list![int(1), int(2) ; WILD], list![int(1), int(2), int(3)]);
        yes!(list![int(1), int(2) ; WILD], list![int(1), int(2)]);
        yes!(list![int(1) ; WILD], list![int(1), int(2), int(3)]);
        yes!(list![int(1) ; list![int(2)]], list![int(1), int(2)]);
        yes!(
            list![int(1) ; list![WILD; list![int(2)]]],
            list![int(1), int(3), int(2)]
        );
        yes!(list![int(1) ; WILD], list![WILD; WILD]);
        yes!(list![int(1) ; WILD], list![int(1), int(2); WILD]);
        yes!(list![], list![]);
        yes!(list![int(1)], list![int(1); list![]]);
    }

    #[test]
    fn no_unify_list() {
        no!(list![int(1), int(2)], list![int(3), int(4)]);
        no!(list![int(1), int(2)], list![int(3); WILD]);
        no!(list![int(1), int(2)], list![]);
        no!(list![WILD], list![]);
        no!(list![WILD; WILD], list![]);
        no!(list![int(1)], list![int(1), int(2)]);
    }

    #[test]
    fn unify_unbound() {
        let mut binding = Binding::default();
        let x = id("x", &mut binding);
        yes!(unbound(WILD), WILD);
        yes!(unbound(int(3)), x, binding);
        yes!(unbound(x.clone()), WILD, binding);
        yes!(list![unbound(x.clone()), x], list![WILD, int(3)], binding);
        yes!(list![x, unbound(x.clone())], list![int(3), WILD], binding);
    }

    #[test]
    fn no_unify_unbound() {
        let mut binding = Binding::default();
        let x = id("x", &mut binding);
        no!(unbound(WILD), int(3));
        no!(unbound(int(3)), int(3));
        no!(unbound(x), int(3), binding);
    }

    #[test]
    fn unify_bound() {
        let mut binding = Binding::default();
        let x = id("x", &mut binding);
        yes!(bound(WILD), int(3));
        yes!(bound(int(3)), int(3));
        yes!(bound(x), int(3), binding);
    }

    #[test]
    fn no_unify_bound() {
        let mut binding = Binding::default();
        let x = id("x", &mut binding);
        no!(bound(WILD), WILD);
        no!(bound(int(3)), x, binding);
        no!(bound(x.clone()), WILD, binding);
        no!(list![bound(x.clone()), x], list![WILD, int(3)], binding);
        no!(list![x, bound(x.clone())], list![int(3), WILD], binding);
    }
}
