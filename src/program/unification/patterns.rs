use super::evaltree::*;
use super::Binding;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

#[cfg_attr(feature = "test-perf", flamer::flame)]
pub(crate) fn unify_patterns<'b>(
    lhs: Pattern,
    rhs: Pattern,
    binding: Cow<'b, Binding>,
) -> Option<Cow<'b, Binding>> {
    Some(
        unify_patterns_inner(
            lhs,
            rhs,
            binding.generation(),
            binding.generation(),
            binding,
        )?
        .1,
    )
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
pub(crate) fn unify_patterns_new_generation<'b>(
    lhs: Pattern,
    rhs: Pattern,
    binding: Cow<'b, Binding>,
) -> Option<Cow<'b, Binding>> {
    Some(
        unify_patterns_inner(
            lhs,
            rhs,
            binding.prev_generation(),
            binding.generation(),
            binding,
        )?
        .1,
    )
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_patterns_inner<'p, 'b>(
    lhs: Pattern,
    rhs: Pattern,
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Pattern, Cow<'b, Binding>)> {
    match (lhs.kind(), rhs.kind()) {
        // The All pattern just... unifies all of them
        (.., PatternKind::All(..)) => unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding),
        (PatternKind::All(patterns), _) => patterns
            .iter()
            .cloned()
            .try_fold((rhs, binding), |(rhs, binding), pattern| {
                unify_patterns_inner(pattern, rhs, lhs_gen, rhs_gen, binding)
            }),
        // Unifying a x with itself succeeds with no additional info.
        (PatternKind::Variable(lhs_var), PatternKind::Variable(rhs_var))
            if lhs_var.set_current(lhs_gen) == rhs_var.set_current(rhs_gen) =>
        {
            // We don't need to use occurs check here because `A =:= A` is allowed, despite
            // `A` being in the occurs list already.
            Some((lhs, binding))
        }
        // Unifying a x with a different x, we use the natural order of variables
        // to designate one as the source of truth and the other as a reference.
        (PatternKind::Variable(lhs_var), PatternKind::Variable(rhs_var)) => {
            let lhs_var = lhs_var.set_current(lhs_gen);
            let rhs_var = rhs_var.set_current(rhs_gen);
            let lhs_pat = binding.get(&lhs_var).unwrap();
            let rhs_pat = binding.get(&rhs_var).unwrap();
            let (pattern, mut binding) = match (lhs_pat.kind(), rhs_pat.kind()) {
                (PatternKind::Variable(lvar), PatternKind::Variable(rvar)) if lvar <= rvar => {
                    (lhs_pat, binding)
                }
                (PatternKind::Variable(..), PatternKind::Variable(..)) => (rhs_pat, binding),
                _ => unify_patterns_inner(lhs_pat, rhs_pat, lhs_gen, rhs_gen, binding)?,
            };
            if lhs_var <= rhs_var {
                binding.to_mut().set(lhs_var, pattern.clone());
                binding.to_mut().set(rhs_var, lhs.clone());
            } else {
                binding.to_mut().set(rhs_var, pattern.clone());
                binding.to_mut().set(lhs_var, rhs.clone());
            }
            Some((pattern, binding))
        }
        // The "bound" pattern requires the other value to already be bound, so this is the only way
        // an unbound variable unification will fail.
        (PatternKind::Bound, PatternKind::Variable(..)) => {
            unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding)
        }
        (PatternKind::Variable(var), PatternKind::Bound) => {
            let var = var.set_current(lhs_gen);
            let val = binding.get(&var).unwrap();
            match val.kind() {
                PatternKind::Variable(..) => None,
                _ => Some((val, binding)),
            }
        }
        // The "unbound" pattern requires the other value is not bound.
        (PatternKind::Unbound, PatternKind::Variable(..)) => {
            unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding)
        }
        (PatternKind::Variable(var), PatternKind::Unbound) => {
            let var = var.set_current(lhs_gen);
            let val = binding.get(&var).unwrap();
            match val.kind() {
                PatternKind::Variable(..) => Some((val, binding)),
                _ => None,
            }
        }
        // A x unified with a value should attempt to dereference the x and then
        // unify. If that succeeds, the x is replaced with the binding.
        (.., PatternKind::Variable(..)) => {
            unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding)
        }
        (PatternKind::Variable(var), ..) => {
            let var = var.set_current(lhs_gen);
            let var_pat = binding.get(&var).unwrap();
            match var_pat.kind() {
                PatternKind::Variable(pat_var) if pat_var == &var => {
                    if rhs.variables(rhs_gen).any(|occurred| var == occurred) {
                        return None;
                    }
                    let mut binding = binding;
                    binding.to_mut().set(var, rhs.clone()); // TODO: this pattern needs to be stored with a generation
                    Some((rhs, binding))
                }
                _ => {
                    let (pattern, binding) =
                        unify_patterns_inner(var_pat, rhs, lhs_gen, rhs_gen, binding)?;
                    Some((pattern, binding))
                }
            }
        }
        // Any values must be the exact same value. We know nothing else about them.
        (PatternKind::Any(lhs_any), PatternKind::Any(rhs_any)) if Rc::ptr_eq(lhs_any, rhs_any) => {
            Some((lhs, binding))
        }
        (PatternKind::Unbound, PatternKind::Unbound) => Some((lhs, binding)),
        // If not with a variable, a "bound" pattern unifies normally
        (_, PatternKind::Bound) => Some((lhs, binding)),
        (PatternKind::Bound, _) => Some((rhs, binding)),
        // Literals must match exactly.
        (PatternKind::Literal(lhs_lit), PatternKind::Literal(rhs_lit)) if lhs_lit == rhs_lit => {
            Some((lhs, binding))
        }
        (PatternKind::Literal(..), PatternKind::Literal(..)) => None,
        // Structs must match in name, and then their contents must match
        (PatternKind::Struct(lhs_str), PatternKind::Struct(rhs_str))
            if lhs_str.name == rhs_str.name
                && lhs_str.contents.is_none()
                && rhs_str.contents.is_none() =>
        {
            Some((lhs, binding))
        }
        (PatternKind::Struct(lhs_str), PatternKind::Struct(rhs_str))
            if lhs_str.name == rhs_str.name
                && lhs_str.contents.is_some()
                && rhs_str.contents.is_some() =>
        {
            let (contents, binding) = unify_patterns_inner(
                lhs_str.contents.clone().unwrap(),
                rhs_str.contents.clone().unwrap(),
                lhs_gen,
                rhs_gen,
                binding,
            )?;
            Some((
                Pattern::new(PatternKind::Struct(Struct {
                    name: lhs_str.name.clone(),
                    contents: Some(contents),
                })),
                binding,
            ))
        }
        (PatternKind::Struct(..), PatternKind::Struct(..)) => None,
        // If neither list has a tail, the heads must match.
        (PatternKind::List(lhs_list, None), PatternKind::List(rhs_list, None)) => {
            let (fields, binding) =
                unify_sequence(&lhs_list, &rhs_list, lhs_gen, rhs_gen, binding)?;
            Some((Pattern::new(PatternKind::List(fields, None)), binding))
        }
        // If only one list has a tail, the tail unifies with whatever the head does
        // not already cover.
        (PatternKind::List(.., None), PatternKind::List(.., Some(..))) => {
            unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding)
        }
        (PatternKind::List(head, Some(tail)), PatternKind::List(full, None)) => {
            match tail.kind() {
                PatternKind::Variable(variable) => {
                    let variable = variable.set_current(lhs_gen);
                    let (output, tail, binding) =
                        unify_full_prefix(head, full, lhs_gen, rhs_gen, binding)?;
                    let tail_pat = binding.get(&variable).unwrap();
                    let (tail, binding) = unify_patterns_inner(
                        Pattern::new(PatternKind::List(tail, None)),
                        tail_pat,
                        lhs_gen,
                        rhs_gen,
                        binding,
                    )?;
                    Some((Pattern::new(PatternKind::List(output, Some(tail))), binding))
                }
                PatternKind::List(cont, tail) => {
                    let mut combined = head.to_owned();
                    combined.extend_from_slice(&cont);
                    let lhs = Pattern::new(PatternKind::List(combined, tail.clone()));
                    unify_patterns_inner(lhs, rhs, lhs_gen, rhs_gen, binding)
                }
                // If the tail cannot unify with a list, then there is a problem.
                _ => None,
            }
        }
        // If both lists have tails, unify the prefixes of the heads, then we'll have
        // one list and one pattern, which can be unified.
        (PatternKind::List(lhs, Some(lhs_tail)), PatternKind::List(rhs, Some(rhs_tail))) => {
            let (unified, remaining, binding) = unify_prefix(lhs, rhs, lhs_gen, rhs_gen, binding)?;
            // The shorter one is the one that is now "done", so we match it's tail with
            // the remaining head and tail of the other list.
            let (suffix, binding) = if lhs.len() < rhs.len() {
                unify_patterns_inner(
                    lhs_tail.clone(),
                    Pattern::new(PatternKind::List(remaining, Some(rhs_tail.clone()))),
                    lhs_gen,
                    rhs_gen,
                    binding,
                )?
            } else {
                unify_patterns_inner(
                    Pattern::new(PatternKind::List(remaining, Some(lhs_tail.clone()))),
                    rhs_tail.clone(),
                    lhs_gen,
                    rhs_gen,
                    binding,
                )?
            };
            Some((
                Pattern::new(PatternKind::List(unified, Some(suffix))),
                binding,
            ))
        }
        // If neither record has a tail, the heads must match like a struct
        (PatternKind::Record(lhs, None), PatternKind::Record(rhs, None)) => {
            let (fields, binding) = unify_fields(&lhs, &rhs, lhs_gen, rhs_gen, binding)?;
            Some((Pattern::new(PatternKind::Record(fields, None)), binding))
        }
        // If only one record has a tail, the tail unifies with whatever the head does
        // not already cover.
        (PatternKind::Record(.., None), PatternKind::Record(.., Some(..))) => {
            unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding)
        }
        (PatternKind::Record(head, Some(tail)), PatternKind::Record(full, None)) => {
            match tail.kind() {
                PatternKind::Variable(ident) => {
                    let ident = ident.set_current(lhs_gen);
                    let (output, tail, binding) =
                        unify_fields_partial(head, full, lhs_gen, rhs_gen, binding)?;
                    let tail_pat = binding.get(&ident).unwrap();
                    let (tail, binding) = unify_patterns_inner(
                        Pattern::new(PatternKind::Record(tail, None)),
                        tail_pat.clone(),
                        lhs_gen,
                        rhs_gen,
                        binding,
                    )?;
                    Some((
                        Pattern::new(PatternKind::Record(output, Some(tail))),
                        binding,
                    ))
                }
                PatternKind::Record(cont, tail) => {
                    let mut combined = head.clone();
                    combined.append(&mut cont.clone());
                    let lhs = Pattern::new(PatternKind::Record(combined, tail.clone()));
                    unify_patterns_inner(lhs, rhs, lhs_gen, rhs_gen, binding)
                }
                // If the tail cannot unify with a record, then there is a problem.
                _ => None,
            }
        }
        // If both records have tails, unify the heads to remove common elements of both, then
        // a record formed from the remaining elements of the other is unified with each tail in
        // turn.
        (PatternKind::Record(lhs, Some(lhs_tail)), PatternKind::Record(rhs, Some(rhs_tail))) => {
            let (intersection, mut lhs_rest, mut rhs_rest, mut binding) =
                unify_fields_difference(lhs, rhs, lhs_gen, rhs_gen, binding)?;
            if intersection.is_empty() {
                let (tail, binding) = unify_patterns_inner(
                    lhs_tail.clone(),
                    rhs_tail.clone(),
                    lhs_gen,
                    rhs_gen,
                    binding,
                )?;
                return Some((
                    Pattern::new(PatternKind::Record(intersection, Some(tail))),
                    binding,
                ));
            }
            let unknown_tail =
                Pattern::new(PatternKind::Variable(binding.to_mut().fresh_variable()));
            let new_rhs_tail = Pattern::new(PatternKind::Record(
                lhs_rest.clone(),
                Some(unknown_tail.clone()),
            ));
            let new_lhs_tail = Pattern::new(PatternKind::Record(
                rhs_rest.clone(),
                Some(unknown_tail.clone()),
            ));
            lhs_rest.append(&mut rhs_rest);
            let out_tail = Pattern::new(PatternKind::Record(lhs_rest, Some(unknown_tail)));
            let (_, binding) =
                unify_patterns_inner(lhs_tail.clone(), new_lhs_tail, lhs_gen, rhs_gen, binding)?;
            let (_, binding) =
                unify_patterns_inner(rhs_tail.clone(), new_rhs_tail, rhs_gen, lhs_gen, binding)?;
            Some((
                Pattern::new(PatternKind::Record(intersection, Some(out_tail))),
                binding,
            ))
        }
        // Otherwise, it's a failure!
        _ => None,
    }
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_sequence<'b>(
    lhs: &[Pattern],
    rhs: &[Pattern],
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Vec<Pattern>, Cow<'b, Binding>)> {
    if lhs.len() != rhs.len() {
        return None;
    }
    lhs.iter()
        .zip(rhs.iter())
        .try_fold((vec![], binding), |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) =
                unify_patterns_inner(lhs.clone(), rhs.clone(), lhs_gen, rhs_gen, binding)?;
            patterns.push(pattern);
            Some((patterns, binding))
        })
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_fields<'b>(
    lhs: &Fields,
    rhs: &Fields,
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Fields, Cow<'b, Binding>)> {
    if lhs.len() != rhs.len() {
        return None;
    }
    let (fields, binding) = lhs.iter().zip(rhs.iter()).try_fold(
        (BTreeMap::default(), binding),
        |(mut fields, binding), (lhs, rhs)| {
            let (pattern, binding) =
                unify_patterns_inner(lhs.1.clone(), rhs.1.clone(), lhs_gen, rhs_gen, binding)?;
            fields.insert(lhs.0.clone(), pattern);
            Some((fields, binding))
        },
    )?;
    Some((Fields::from(fields), binding))
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_fields_partial<'b>(
    part: &Fields,
    full: &Fields,
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Fields, Fields, Cow<'b, Binding>)> {
    let mut full: BTreeMap<_, _> = full.clone().into();
    let (fields, binding) = part.iter().try_fold(
        (BTreeMap::new(), binding),
        |(mut fields, binding), (key, pattern)| {
            let (unified, binding) = unify_patterns_inner(
                pattern.clone(),
                full.remove(&key)?,
                lhs_gen,
                rhs_gen,
                binding,
            )?;
            fields.insert(key.clone(), unified);
            Some((fields, binding))
        },
    )?;
    Some((fields.into(), full.into(), binding))
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_fields_difference<'b>(
    lhs: &Fields,
    rhs: &Fields,
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Fields, Fields, Fields, Cow<'b, Binding>)> {
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
                let (pattern, binding) = unify_patterns_inner(lhs, rhs, lhs_gen, rhs_gen, binding)?;
                intersection.insert(key, pattern);
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

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_prefix<'b>(
    lhs: &[Pattern],
    rhs: &[Pattern],
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Vec<Pattern>, Vec<Pattern>, Cow<'b, Binding>)> {
    let (head, binding) = lhs.iter().zip(rhs.iter()).try_fold(
        (vec![], binding),
        |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) =
                unify_patterns_inner(lhs.clone(), rhs.clone(), lhs_gen, rhs_gen, binding)?;
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

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_full_prefix<'b>(
    lhs: &[Pattern],
    rhs: &[Pattern],
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Vec<Pattern>, Vec<Pattern>, Cow<'b, Binding>)> {
    if lhs.len() > rhs.len() {
        return None;
    }
    let (head, binding) = lhs.iter().zip(rhs.iter()).try_fold(
        (vec![], binding),
        |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) =
                unify_patterns_inner(lhs.clone(), rhs.clone(), lhs_gen, rhs_gen, binding)?;
            patterns.push(pattern.clone());
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
            assert!(
                unify_patterns($lhs.clone(), $rhs.clone(), Cow::Owned(Binding::default()),)
                    .is_some()
            )
        };
        ($lhs:expr, $rhs:expr, $binding:expr $(,)?) => {{
            let output = unify_patterns($lhs.clone(), $rhs.clone(), Cow::Owned($binding.clone()));
            assert!(output.is_some());
        }};
    }

    macro_rules! no {
        ($lhs:expr, $rhs:expr $(,)?) => {
            assert!(
                unify_patterns($lhs.clone(), $rhs.clone(), Cow::Owned(Binding::default()),)
                    .is_none()
            )
        };
        ($lhs:expr, $rhs:expr, $binding:expr $(,)?) => {
            assert!(
                unify_patterns($lhs.clone(), $rhs.clone(), Cow::Owned($binding.clone()),).is_none()
            )
        };
    }

    fn atom(name: &str) -> Pattern {
        Pattern::new(PatternKind::Struct(Struct::from_parts(
            Atom::from(name),
            None,
        )))
    }

    fn var(binding: &mut Binding) -> Pattern {
        Pattern::new(PatternKind::Variable(binding.fresh_variable()))
    }

    fn int(val: impl Into<ramp::int::Int>) -> Pattern {
        Pattern::new(PatternKind::Literal(Literal::Integer(val.into())))
    }

    fn rat(val: impl Into<ramp::rational::Rational>) -> Pattern {
        Pattern::new(PatternKind::Literal(Literal::Rational(val.into())))
    }

    fn string(val: impl Into<String>) -> Pattern {
        Pattern::new(PatternKind::Literal(Literal::String(val.into())))
    }

    fn unbound() -> Pattern {
        Pattern::new(PatternKind::Unbound)
    }
    fn bound() -> Pattern {
        Pattern::new(PatternKind::Bound)
    }

    macro_rules! list {
        () => (Pattern::new(PatternKind::List(vec![], None)));
        ($($item:expr),+) => (Pattern::new(PatternKind::List(vec![$($item.clone()),+], None)));
        ($($item:expr),+ ; $rest:expr) => (Pattern::new(PatternKind::List(vec![$($item.clone()),+], Some($rest.clone()))));
    }

    macro_rules! structure {
        (
            $name:ident ($contents:expr)
        ) => {
            Pattern::new(PatternKind::Struct(Struct::from_parts(
                Atom::from(stringify!($name)),
                Some($contents.clone()),
            )))
        };
    }

    macro_rules! record {
        (
            @ [$fields:ident] (.. $rest:expr)
        ) => {{
            Pattern::new(PatternKind::Record($fields.into(), Some($rest.clone())))
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
            Pattern::new(PatternKind::Record($fields.into(), None))
        }};

        ($($field:tt)+) => {{
            #[allow(unused_mut)]
            let mut fields = BTreeMap::default();
            record!(@[fields] ($($field)+))
        }};

        () => { Pattern::new(PatternKind::Record(Default::default(), None)) }
    }

    macro_rules! all {
        ($($pat:expr),+) => {
            Pattern::new(PatternKind::All(vec![$($pat.clone()),+]))
        };
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
        let mut binding = Binding::default();
        yes!(
            structure!(hello(atom("world"))),
            structure!(hello(atom("world"))),
        );

        yes!(
            structure!(hello(list![rat(1)])),
            structure!(hello(list![rat(1)])),
        );

        yes!(
            structure!(hello(var(&mut binding))),
            structure!(hello(atom("hello"))),
            binding,
        );

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
        let mut binding = Binding::default();
        let x = var(&mut binding);
        let y = var(&mut binding);
        yes!(record! {}, record! {});
        yes!(record! { a: int(1) }, record! { a: int(1) });
        yes!(record! { a: int(1) }, record! { a: int(1), ..x }, binding,);
        yes!(
            record! { a: int(1), b: int(2) },
            record! { a: int(1), ..x },
            binding,
        );
        yes!(
            record! { a: int(1), b: int(2), ..x },
            record! { a: int(1), ..x },
            binding,
        );
        yes!(
            record! { a: int(1), b: int(2), ..x },
            record! { a: int(1), c: int(3), ..y },
            binding,
        );
    }

    #[test]
    fn no_unify_record() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        let y = var(&mut binding);
        no!(record! {}, record! { a: int(1) });
        no!(record! { a: int(1) }, record! { a: int(2) });
        no!(record! { a: int(2) }, record! { a: int(1), ..x }, binding,);
        no!(
            record! { a: int(1), b: int(2), ..x },
            record! { a: int(1), b: int(3), ..y },
            binding,
        );
    }

    #[test]
    fn unify_variable() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        let y = var(&mut binding);
        yes!(x, var(&mut binding), binding);
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
        let x = var(&mut binding);
        let y = var(&mut binding);
        let z = var(&mut binding);
        yes!(structure!(test(list![x, y])), z, binding);
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
            record! { c: int(1), d: int(2), ..y },
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
        let x = var(&mut binding);
        no!(x, list![x], binding);
        no!(x, list![int(3); x], binding);
        no!(x, structure!(hello(x)), binding);
        no!(list![int(1); x], list![int(1), int(2); x], binding);
    }

    #[test]
    fn unify_list() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        let y = var(&mut binding);
        let z = var(&mut binding);
        yes!(list![int(1), int(2)], list![int(1), int(2)]);
        yes!(
            list![int(1), int(2) ; x],
            list![int(1), int(2), int(3)],
            binding,
        );
        yes!(list![int(1), int(2) ; x], list![int(1), int(2)], binding);
        yes!(list![int(1) ; x], list![int(1), int(2), int(3)], binding);
        yes!(list![int(1) ; list![int(2)]], list![int(1), int(2)]);
        yes!(
            list![int(1) ; list![x; list![int(2)]]],
            list![int(1), int(3), int(2)],
            binding,
        );
        yes!(list![int(1) ; x], list![y; z], binding);
        yes!(list![int(1) ; x], list![int(1), int(2); y], binding);
        yes!(list![], list![]);
        yes!(list![int(1)], list![int(1); list![]]);
    }

    #[test]
    fn no_unify_list() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        let y = var(&mut binding);
        no!(list![int(1), int(2)], list![int(3), int(4)]);
        no!(list![int(1), int(2)], list![int(3); x], binding);
        no!(list![int(1), int(2)], list![]);
        no!(list![x], list![], binding);
        no!(list![x; y], list![], binding);
        no!(list![int(1)], list![int(1), int(2)]);
    }

    #[test]
    fn unify_unbound() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        let y = var(&mut binding);
        yes!(unbound(), x, binding);
        yes!(all![unbound(), int(3)], x, binding);
        yes!(all![unbound(), x], y, binding);
        yes!(list![all![unbound(), x], x], list![y, int(3)], binding);
        yes!(list![x, all![unbound(), x]], list![int(3), y], binding);
    }

    #[test]
    fn no_unify_unbound() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        no!(unbound(), int(3));
        no!(all![unbound(), int(3)], int(3));
        no!(all![unbound(), x], int(3), binding);
    }

    #[test]
    fn unify_bound() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        yes!(bound(), int(3));
        yes!(all![bound(), int(3)], int(3));
        yes!(all![bound(), x], int(3), binding);
    }

    #[test]
    fn no_unify_bound() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        let y = var(&mut binding);
        no!(bound(), y, binding);
        no!(all![bound(), int(3)], x, binding);
        no!(all![bound(), x], y, binding);
        no!(list![all![bound(), x], x], list![y, int(3)], binding);
        no!(list![x, all![bound(), x]], list![int(3), y], binding);
    }
}
