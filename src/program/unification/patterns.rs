use super::evaltree::*;
use super::Binding;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

#[cfg_attr(feature = "test-perf", flamer::flame)]
pub(crate) fn unify_patterns<'p, 'b>(
    lhs: Cow<'p, Pattern>,
    rhs: Cow<'p, Pattern>,
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
pub(crate) fn unify_patterns_new_generation<'p, 'b>(
    lhs: Cow<'p, Pattern>,
    rhs: Cow<'p, Pattern>,
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
    lhs: Cow<'p, Pattern>,
    rhs: Cow<'p, Pattern>,
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Cow<'p, Pattern>, Cow<'b, Binding>)> {
    eprintln!("{:?} {} =:= {:?} {}", lhs, lhs_gen, rhs, rhs_gen);
    eprintln!("{}", binding);
    match (lhs.as_ref(), rhs.as_ref()) {
        // The All pattern just... unifies all of them
        (.., Pattern::All(..)) => unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding),
        (Pattern::All(patterns), other) => patterns
            .iter()
            .try_fold(
                (Cow::Borrowed(other), binding),
                |(other, binding), pattern| {
                    unify_patterns_inner(Cow::Borrowed(pattern), other, lhs_gen, rhs_gen, binding)
                },
            )
            .map(|(cow, binding)| (Cow::Owned(cow.into_owned()), binding)),
        // Unifying a x with itself succeeds with no additional info.
        (Pattern::Variable(lhs_var), Pattern::Variable(rhs_var))
            if lhs_var.set_current(lhs_gen) == rhs_var.set_current(rhs_gen) =>
        {
            // We don't need to use occurs check here because `A =:= A` is allowed, despite
            // `A` being in the occurs list already.
            Some((lhs, binding))
        }
        // Unifying a x with a different x, we use the natural order of variables
        // to designate one as the source of truth and the other as a reference.
        (Pattern::Variable(lhs_var), Pattern::Variable(rhs_var)) => {
            let lhs_var = lhs_var.set_current(lhs_gen);
            let rhs_var = rhs_var.set_current(rhs_gen);
            let lhs_pat = binding.get(&lhs_var).unwrap();
            let rhs_pat = binding.get(&rhs_var).unwrap();
            let (pattern, mut binding) = match (lhs_pat.as_ref(), rhs_pat.as_ref()) {
                (Pattern::Variable(lvar), Pattern::Variable(rvar)) => {
                    let pattern = Pattern::Variable(Variable::min(lvar.clone(), rvar.clone()));
                    (Cow::Owned(pattern), binding)
                }
                _ => unify_patterns_inner(
                    Cow::Borrowed(lhs_pat.as_ref()),
                    Cow::Borrowed(rhs_pat.as_ref()),
                    lhs_gen,
                    rhs_gen,
                    binding,
                )?,
            };
            let min = Variable::min(lhs_var.clone(), rhs_var.clone());
            let max = Variable::max(lhs_var, rhs_var);
            let reference = Pattern::Variable(min.clone());
            binding.to_mut().set(min, pattern.clone().into_owned());
            binding.to_mut().set(max, reference);
            Some((Cow::Owned(pattern.into_owned()), binding))
        }
        // The "bound" pattern requires the other value to already be bound, so this is the only way
        // an unbound variable unification will fail.
        (Pattern::Bound, Pattern::Variable(..)) => {
            unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding)
        }
        (Pattern::Variable(var), Pattern::Bound) => {
            let var = var.set_current(lhs_gen);
            match binding.get(&var).unwrap().as_ref() {
                Pattern::Variable(..) => None,
                val => Some((Cow::Owned(val.clone()), binding)),
            }
        }
        // The "unbound" pattern requires the other value is not bound.
        (Pattern::Unbound, Pattern::Variable(..)) => {
            unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding)
        }
        (Pattern::Variable(var), Pattern::Unbound) => {
            let var = var.set_current(lhs_gen);
            match binding.get(&var).unwrap().as_ref() {
                val @ Pattern::Variable(..) => Some((Cow::Owned(val.clone()), binding)),
                _ => None,
            }
        }
        // A x unified with a value should attempt to dereference the x and then
        // unify. If that succeeds, the x is replaced with the binding.
        (.., Pattern::Variable(..)) => unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding),
        (Pattern::Variable(var), pattern) => {
            let var = var.set_current(lhs_gen);
            let var_pat = binding.get(&var).unwrap();
            match var_pat.as_ref() {
                Pattern::Variable(pat_var) if pat_var == &var => {
                    if pattern.variables(rhs_gen).any(|occurred| var == occurred) {
                        return None;
                    }
                    let mut binding = binding;
                    binding.to_mut().set(var, pattern.clone()); // TODO: this pattern needs to be stored with a generation
                    Some((Cow::Owned(pattern.clone()), binding))
                }
                _ => {
                    let (pattern, binding) = unify_patterns_inner(
                        Cow::Borrowed(var_pat.as_ref()),
                        Cow::Borrowed(pattern),
                        lhs_gen,
                        rhs_gen,
                        binding,
                    )?;
                    Some((Cow::Owned(pattern.into_owned()), binding))
                }
            }
        }
        // Any values must be the exact same value. We know nothing else about them.
        (Pattern::Any(lhs_any), Pattern::Any(rhs_any)) if Rc::ptr_eq(lhs_any, rhs_any) => {
            Some((lhs, binding))
        }
        (Pattern::Unbound, Pattern::Unbound) => Some((lhs, binding)),
        // If not with a variable, a "bound" pattern unifies normally
        (_, Pattern::Bound) => Some((lhs, binding)),
        (Pattern::Bound, _) => Some((rhs, binding)),
        // Literals must match exactly.
        (Pattern::Literal(lhs_lit), Pattern::Literal(rhs_lit)) if lhs_lit == rhs_lit => {
            Some((lhs, binding))
        }
        (Pattern::Literal(..), Pattern::Literal(..)) => None,
        // Structs must match in name, and then their contents must match
        (Pattern::Struct(lhs_str), Pattern::Struct(rhs_str))
            if lhs_str.name == rhs_str.name
                && lhs_str.contents.is_none()
                && rhs_str.contents.is_none() =>
        {
            Some((lhs, binding))
        }
        (Pattern::Struct(lhs_str), Pattern::Struct(rhs_str))
            if lhs_str.name == rhs_str.name
                && lhs_str.contents.is_some()
                && rhs_str.contents.is_some() =>
        {
            let (contents, binding) = unify_patterns_inner(
                Cow::Borrowed(lhs_str.contents.as_ref().unwrap()),
                Cow::Borrowed(rhs_str.contents.as_ref().unwrap()),
                lhs_gen,
                rhs_gen,
                binding,
            )?;
            Some((
                Cow::Owned(Pattern::Struct(Struct {
                    name: lhs_str.name.clone(),
                    contents: Some(Box::new(contents.into_owned())),
                })),
                binding,
            ))
        }
        (Pattern::Struct(..), Pattern::Struct(..)) => None,
        // If neither list has a tail, the heads must match.
        (Pattern::List(lhs_list, None), Pattern::List(rhs_list, None)) => {
            let (fields, binding) =
                unify_sequence(&lhs_list, &rhs_list, lhs_gen, rhs_gen, binding)?;
            Some((
                Cow::Owned(Pattern::List(
                    fields.into_iter().map(Cow::into_owned).collect(),
                    None,
                )),
                binding,
            ))
        }
        // If only one list has a tail, the tail unifies with whatever the head does
        // not already cover.
        (Pattern::List(.., None), Pattern::List(.., Some(..))) => {
            unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding)
        }
        (Pattern::List(head, Some(tail)), Pattern::List(full, None)) => {
            match tail.as_ref() {
                Pattern::Variable(variable) => {
                    let variable = variable.set_current(lhs_gen);
                    let (output, tail, binding) =
                        unify_full_prefix(head, full, lhs_gen, rhs_gen, binding)?;
                    let tail_pat = binding.get(&variable).unwrap();
                    let (tail, binding) = unify_patterns_inner(
                        Cow::Owned(Pattern::List(tail, None)),
                        Cow::Borrowed(tail_pat.as_ref()),
                        lhs_gen,
                        rhs_gen,
                        binding,
                    )?;
                    Some((
                        Cow::Owned(Pattern::List(output, Some(Box::new(tail.into_owned())))),
                        binding,
                    ))
                }
                Pattern::List(cont, tail) => {
                    let mut combined = head.to_owned();
                    combined.extend_from_slice(&cont);
                    let lhs = Pattern::List(combined, tail.clone());
                    unify_patterns_inner(Cow::Owned(lhs), rhs, lhs_gen, rhs_gen, binding)
                }
                // If the tail cannot unify with a list, then there is a problem.
                _ => None,
            }
        }
        // If both lists have tails, unify the prefixes of the heads, then we'll have
        // one list and one pattern, which can be unified.
        (Pattern::List(lhs, Some(lhs_tail)), Pattern::List(rhs, Some(rhs_tail))) => {
            let (unified, remaining, binding) = unify_prefix(lhs, rhs, lhs_gen, rhs_gen, binding)?;
            // The shorter one is the one that is now "done", so we match it's tail with
            // the remaining head and tail of the other list.
            let (suffix, binding) = if lhs.len() < rhs.len() {
                unify_patterns_inner(
                    Cow::Borrowed(lhs_tail.as_ref()),
                    Cow::Owned(Pattern::List(remaining, Some(rhs_tail.clone()))),
                    lhs_gen,
                    rhs_gen,
                    binding,
                )?
            } else {
                unify_patterns_inner(
                    Cow::Owned(Pattern::List(remaining, Some(lhs_tail.clone()))),
                    Cow::Borrowed(rhs_tail.as_ref()),
                    lhs_gen,
                    rhs_gen,
                    binding,
                )?
            };
            Some((
                Cow::Owned(Pattern::List(unified, Some(Box::new(suffix.into_owned())))),
                binding,
            ))
        }
        // If neither record has a tail, the heads must match like a struct
        (Pattern::Record(lhs, None), Pattern::Record(rhs, None)) => {
            let (fields, binding) = unify_fields(&lhs, &rhs, lhs_gen, rhs_gen, binding)?;
            Some((Cow::Owned(Pattern::Record(fields, None)), binding))
        }
        // If only one record has a tail, the tail unifies with whatever the head does
        // not already cover.
        (Pattern::Record(.., None), Pattern::Record(.., Some(..))) => {
            unify_patterns_inner(rhs, lhs, rhs_gen, lhs_gen, binding)
        }
        (Pattern::Record(head, Some(tail)), Pattern::Record(full, None)) => {
            match tail.as_ref() {
                Pattern::Variable(ident) => {
                    let ident = ident.set_current(lhs_gen);
                    let (output, tail, binding) =
                        unify_fields_partial(head, full, lhs_gen, rhs_gen, binding)?;
                    let tail_pat = binding.get(&ident).unwrap();
                    let (tail, binding) = unify_patterns_inner(
                        Cow::Owned(Pattern::Record(tail, None)),
                        Cow::Borrowed(tail_pat.as_ref()),
                        lhs_gen,
                        rhs_gen,
                        binding,
                    )?;
                    Some((
                        Cow::Owned(Pattern::Record(output, Some(Box::new(tail.into_owned())))),
                        binding,
                    ))
                }
                Pattern::Record(cont, tail) => {
                    let mut combined = head.clone();
                    combined.append(&mut cont.clone());
                    let lhs = Pattern::Record(combined, tail.clone());
                    unify_patterns_inner(Cow::Owned(lhs), rhs, lhs_gen, rhs_gen, binding)
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
                unify_fields_difference(lhs, rhs, lhs_gen, rhs_gen, binding)?;
            if intersection.is_empty() {
                let (tail, binding) = unify_patterns_inner(
                    Cow::Borrowed(lhs_tail.as_ref()),
                    Cow::Borrowed(rhs_tail.as_ref()),
                    lhs_gen,
                    rhs_gen,
                    binding,
                )?;
                return Some((
                    Cow::Owned(Pattern::Record(
                        intersection,
                        Some(Box::new(tail.into_owned())),
                    )),
                    binding,
                ));
            }
            let unknown_tail = binding.to_mut().fresh_variable();
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
            let (_, binding) = unify_patterns_inner(
                Cow::Borrowed(lhs_tail.as_ref()),
                Cow::Owned(new_lhs_tail),
                lhs_gen,
                rhs_gen,
                binding,
            )?;
            let (_, binding) = unify_patterns_inner(
                Cow::Borrowed(rhs_tail.as_ref()),
                Cow::Owned(new_rhs_tail),
                rhs_gen,
                lhs_gen,
                binding,
            )?;
            Some((
                Cow::Owned(Pattern::Record(intersection, Some(Box::new(out_tail)))),
                binding,
            ))
        }
        // Otherwise, it's a failure!
        _ => None,
    }
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_sequence<'p, 'b>(
    lhs: &'p [Pattern],
    rhs: &'p [Pattern],
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Vec<Cow<'p, Pattern>>, Cow<'b, Binding>)> {
    if lhs.len() != rhs.len() {
        return None;
    }
    lhs.iter()
        .zip(rhs.iter())
        .try_fold((vec![], binding), |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns_inner(
                Cow::Borrowed(lhs),
                Cow::Borrowed(rhs),
                lhs_gen,
                rhs_gen,
                binding,
            )?;
            patterns.push(pattern);
            Some((patterns, binding))
        })
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_fields<'p, 'b>(
    lhs: &'p Fields,
    rhs: &'p Fields,
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
            let (pattern, binding) = unify_patterns_inner(
                Cow::Borrowed(&lhs.1),
                Cow::Borrowed(&rhs.1),
                lhs_gen,
                rhs_gen,
                binding,
            )?;
            fields.insert(lhs.0.clone(), pattern.into_owned());
            Some((fields, binding))
        },
    )?;
    Some((Fields::from(fields), binding))
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_fields_partial<'p, 'b>(
    part: &'p Fields,
    full: &'p Fields,
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Fields, Fields, Cow<'b, Binding>)> {
    let mut full: BTreeMap<_, _> = full.clone().into();
    let (fields, binding) = part.iter().try_fold(
        (BTreeMap::new(), binding),
        |(mut fields, binding), (key, pattern)| {
            let (unified, binding) = unify_patterns_inner(
                Cow::Borrowed(pattern),
                Cow::Owned(full.remove(&key)?),
                lhs_gen,
                rhs_gen,
                binding,
            )?;
            fields.insert(key.clone(), unified.into_owned());
            Some((fields, binding))
        },
    )?;
    Some((fields.into(), full.into(), binding))
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_fields_difference<'p, 'b>(
    lhs: &'p Fields,
    rhs: &'p Fields,
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
                let (patterns, binding) = unify_patterns_inner(
                    Cow::Owned(lhs),
                    Cow::Owned(rhs),
                    lhs_gen,
                    rhs_gen,
                    binding,
                )?;
                intersection.insert(key, patterns.into_owned());
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
fn unify_prefix<'p, 'b>(
    lhs: &'p [Pattern],
    rhs: &'p [Pattern],
    lhs_gen: usize,
    rhs_gen: usize,
    binding: Cow<'b, Binding>,
) -> Option<(Vec<Pattern>, Vec<Pattern>, Cow<'b, Binding>)> {
    let (head, binding) = lhs.iter().zip(rhs.iter()).try_fold(
        (vec![], binding),
        |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns_inner(
                Cow::Borrowed(lhs),
                Cow::Borrowed(rhs),
                lhs_gen,
                rhs_gen,
                binding,
            )?;
            patterns.push(pattern.into_owned());
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
fn unify_full_prefix<'p, 'b>(
    lhs: &'p [Pattern],
    rhs: &'p [Pattern],
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
            let (pattern, binding) = unify_patterns_inner(
                Cow::Borrowed(lhs),
                Cow::Borrowed(rhs),
                lhs_gen,
                rhs_gen,
                binding,
            )?;
            patterns.push(pattern.into_owned());
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
            assert!(unify_patterns(
                Cow::Owned($lhs.clone()),
                Cow::Owned($rhs.clone()),
                Cow::Owned(Binding::default()),
            )
            .is_some())
        };
        ($lhs:expr, $rhs:expr, $binding:expr $(,)?) => {{
            let output = unify_patterns(
                Cow::Owned($lhs.clone()),
                Cow::Owned($rhs.clone()),
                Cow::Owned($binding.clone()),
            );
            assert!(output.is_some());
        }};
    }

    macro_rules! no {
        ($lhs:expr, $rhs:expr $(,)?) => {
            assert!(unify_patterns(
                Cow::Owned($lhs.clone()),
                Cow::Owned($rhs.clone()),
                Cow::Owned(Binding::default()),
            )
            .is_none())
        };
        ($lhs:expr, $rhs:expr, $binding:expr $(,)?) => {
            assert!(unify_patterns(
                Cow::Borrowed(&$lhs),
                Cow::Borrowed(&$rhs),
                Cow::Owned($binding.clone()),
            )
            .is_none())
        };
    }

    fn atom(name: &str) -> Pattern {
        Pattern::Struct(Struct::from_parts(Atom::from(name), None))
    }

    fn var(binding: &mut Binding) -> Pattern {
        Pattern::Variable(binding.fresh_variable())
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

    const UNBOUND: Pattern = Pattern::Unbound;
    const BOUND: Pattern = Pattern::Bound;

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

    macro_rules! all {
        ($($pat:expr),+) => {
            Pattern::All(vec![$($pat.clone()),+])
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
        yes!(UNBOUND, x, binding);
        yes!(all![UNBOUND, int(3)], x, binding);
        yes!(all![UNBOUND, x], y, binding);
        yes!(list![all![UNBOUND, x], x], list![y, int(3)], binding);
        yes!(list![x, all![UNBOUND, x]], list![int(3), y], binding);
    }

    #[test]
    fn no_unify_unbound() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        no!(UNBOUND, int(3));
        no!(all![UNBOUND, int(3)], int(3));
        no!(all![UNBOUND, x], int(3), binding);
    }

    #[test]
    fn unify_bound() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        yes!(BOUND, int(3));
        yes!(all![BOUND, int(3)], int(3));
        yes!(all![BOUND, x], int(3), binding);
    }

    #[test]
    fn no_unify_bound() {
        let mut binding = Binding::default();
        let x = var(&mut binding);
        let y = var(&mut binding);
        no!(BOUND, y, binding);
        no!(all![BOUND, int(3)], x, binding);
        no!(all![BOUND, x], y, binding);
        no!(list![all![BOUND, x], x], list![y, int(3)], binding);
        no!(list![x, all![BOUND, x]], list![int(3), y], binding);
    }
}
