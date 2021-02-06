use super::evaltree::*;
use super::Binding;
use im_rc::{vector, OrdMap, Vector};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

type Fields = OrdMap<Atom, Pattern>;

fn occurs(variable: &Variable, pattern: Pattern, binding: &Binding) -> bool {
    #[cfg(feature = "test-perf")]
    let _guard = {
        let name = match pattern.kind() {
            PatternKind::Variable(identifier) => format!("var {}", identifier.name()),
            PatternKind::List(..) => "list".to_owned(),
            PatternKind::Record(..) => "record".to_owned(),
            PatternKind::Struct(name, ..) => format!("struct {}", name),
            PatternKind::Literal(..) => "literal".to_owned(),
            PatternKind::All(..) => "all".to_owned(),
            PatternKind::Any(..) => "any".to_owned(),
            _ => format!("{}", pattern),
        };
        flame::start_guard(format!("occurs {} <- {}", variable, name))
    };
    pattern.variables().any(|ref var| {
        #[cfg(feature = "test-perf")]
        flame::start_guard(format!("var {}", var));
        if var == variable {
            return true;
        }
        if var.generation().is_none() {
            return false; // a real wildcard is not bound to anything, so it can't contain anything...
        }
        let pattern = binding.get(var).unwrap();
        match pattern.kind() {
            PatternKind::Variable(v) if v == variable => true,
            PatternKind::Variable(..) => false,
            _ => occurs(variable, pattern, binding),
        }
    })
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
pub(crate) fn unify_patterns(
    lhs: Pattern,
    rhs: Pattern,
    binding: Cow<'_, Binding>,
) -> Option<Cow<'_, Binding>> {
    Some(
        unify_patterns_inner(
            lhs.default_age(Some(binding.generation())),
            rhs.default_age(Some(binding.generation())),
            binding,
        )?
        .1,
    )
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
pub(crate) fn unify_patterns_new_generation(
    lhs: Pattern,
    rhs: Pattern,
    binding: Cow<'_, Binding>,
) -> Option<Cow<'_, Binding>> {
    Some(
        unify_patterns_inner(
            lhs.default_age(Some(binding.prev_generation())),
            rhs.default_age(Some(binding.generation())),
            binding,
        )?
        .1,
    )
}

fn unify_patterns_inner(
    lhs: Pattern,
    rhs: Pattern,
    binding: Cow<'_, Binding>,
) -> Option<(Pattern, Cow<'_, Binding>)> {
    let lhs_age = lhs.age();
    let rhs_age = rhs.age();

    #[cfg(feature = "test-perf")]
    let _guard = {
        let lname = match lhs.kind() {
            PatternKind::Variable(identifier) => format!("var {}", identifier.name()),
            PatternKind::List(..) => "list".to_owned(),
            PatternKind::Record(..) => "record".to_owned(),
            PatternKind::Struct(name, ..) => format!("struct {}", name),
            PatternKind::Literal(..) => "literal".to_owned(),
            PatternKind::All(..) => "all".to_owned(),
            PatternKind::Any(..) => "any".to_owned(),
            _ => format!("{}", lhs),
        };

        let rname = match rhs.kind() {
            PatternKind::Variable(identifier) => format!("var {}", identifier.name()),
            PatternKind::List(..) => "list".to_owned(),
            PatternKind::Record(..) => "record".to_owned(),
            PatternKind::Struct(name, ..) => format!("struct {}", name),
            PatternKind::Literal(..) => "literal".to_owned(),
            PatternKind::All(..) => "all".to_owned(),
            PatternKind::Any(..) => "any".to_owned(),
            _ => format!("{}", rhs),
        };
        flame::start_guard(format!("{} =:= {}", lname, rname))
    };

    match (lhs.kind(), rhs.kind()) {
        // The All pattern just... unifies all of them
        (.., PatternKind::All(..)) => unify_patterns_inner(rhs, lhs, binding),
        (PatternKind::All(patterns), _) => {
            patterns
                .iter()
                .cloned()
                .try_fold((rhs, binding), |(rhs, binding), pattern| {
                    unify_patterns_inner(pattern.default_age(lhs_age), rhs, binding)
                })
        }
        // Unifying a x with itself succeeds with no additional info.
        (PatternKind::Variable(lhs_var), PatternKind::Variable(rhs_var)) if lhs_var == rhs_var => {
            // We don't need to use occurs check here because `A =:= A` is allowed, despite
            // `A` being in the occurs list already.
            Some((lhs, binding))
        }
        // Unifying a x with a different x, we use the natural order of variables
        // to designate one as the source of truth and the other as a reference.
        (PatternKind::Variable(lhs_var), PatternKind::Variable(rhs_var)) => {
            let lhs_pat = binding.get(&lhs_var).unwrap();
            let rhs_pat = binding.get(&rhs_var).unwrap();
            let (pattern, mut binding) = match (lhs_pat.kind(), rhs_pat.kind()) {
                (PatternKind::Variable(lvar), PatternKind::Variable(rvar)) => {
                    if lvar < rvar {
                        (lhs_pat, binding)
                    } else {
                        (rhs_pat, binding)
                    }
                }
                _ => unify_patterns_inner(lhs_pat, rhs_pat, binding)?,
            };
            binding.to_mut().set(lhs_var.clone(), pattern.clone());
            binding.to_mut().set(rhs_var.clone(), pattern.clone());
            Some((pattern, binding))
        }
        // The "bound" pattern requires the other value to already be bound, so this is the only way
        // an unbound variable unification will fail.
        (PatternKind::Bound, PatternKind::Variable(..)) => unify_patterns_inner(rhs, lhs, binding),
        (PatternKind::Variable(var), PatternKind::Bound) => {
            let val = binding.get(var).unwrap();
            match val.kind() {
                PatternKind::Variable(..) => None,
                _ => Some((val, binding)),
            }
        }
        // The "unbound" pattern requires the other value is not bound.
        (PatternKind::Unbound, PatternKind::Variable(..)) => {
            unify_patterns_inner(rhs, lhs, binding)
        }
        (PatternKind::Variable(var), PatternKind::Unbound) => {
            let val = binding.get(var).unwrap();
            match val.kind() {
                PatternKind::Variable(..) => Some((val, binding)),
                _ => None,
            }
        }
        // A x unified with a value should attempt to dereference the x and then
        // unify. If that succeeds, the x is replaced with the binding.
        (.., PatternKind::Variable(..)) => unify_patterns_inner(rhs, lhs, binding),
        (PatternKind::Variable(var), ..) => {
            let var_pat = binding.get(&var).unwrap();
            match var_pat.kind() {
                PatternKind::Variable(pat_var) => {
                    if occurs(pat_var, rhs.clone(), binding.as_ref()) {
                        return None;
                    }
                    let mut binding = binding;
                    binding.to_mut().set(pat_var.clone(), rhs.clone());
                    Some((rhs, binding))
                }
                _ => {
                    let (pattern, binding) = unify_patterns_inner(var_pat, rhs, binding)?;
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
        (PatternKind::Struct(lname, None), PatternKind::Struct(rname, None)) if lname == rname => {
            Some((lhs, binding))
        }
        (
            PatternKind::Struct(lname, Some(lcontents)),
            PatternKind::Struct(rname, Some(rcontents)),
        ) if lname == rname => {
            let (contents, binding) = unify_patterns_inner(
                lcontents.default_age(lhs_age),
                rcontents.default_age(rhs_age),
                binding,
            )?;
            Some((
                Pattern::from(PatternKind::Struct(lname.clone(), Some(contents))),
                binding,
            ))
        }
        (PatternKind::Struct(..), PatternKind::Struct(..)) => None,
        // If neither list has a tail, the heads must match.
        (PatternKind::List(lhs_list, None), PatternKind::List(rhs_list, None)) => {
            let (fields, binding) = unify_sequence(
                lhs_list
                    .iter()
                    .map(|pat| pat.default_age(lhs_age))
                    .collect(),
                rhs_list
                    .iter()
                    .map(|pat| pat.default_age(rhs_age))
                    .collect(),
                binding,
            )?;
            Some((Pattern::list(fields, None), binding))
        }
        // If only one list has a tail, the tail unifies with whatever the head does
        // not already cover.
        (PatternKind::List(.., None), PatternKind::List(.., Some(..))) => {
            unify_patterns_inner(rhs, lhs, binding)
        }
        (PatternKind::List(head, Some(tail)), PatternKind::List(full, None)) => {
            match tail.kind() {
                PatternKind::Variable(variable) => {
                    let (output, tail, binding) = unify_full_prefix(
                        head.iter().map(|pat| pat.default_age(lhs_age)).collect(),
                        full.iter().map(|pat| pat.default_age(rhs_age)).collect(),
                        binding,
                    )?;
                    let tail_pat = binding.get(&variable.set_current(lhs_age)).unwrap();
                    let (tail, binding) =
                        unify_patterns_inner(tail_pat, Pattern::list(tail, None), binding)?;
                    Some((Pattern::list(output, Some(tail)), binding))
                }
                PatternKind::List(..) => {
                    panic!("should not reach here... the tails should always be variables")
                }
                // If the tail cannot unify with a list, then there is a problem.
                _ => None,
            }
        }
        // If both lists have tails, unify the prefixes of the heads, then we'll have
        // one list and one pattern, which can be unified.
        (PatternKind::List(lhs, Some(lhs_tail)), PatternKind::List(rhs, Some(rhs_tail))) => {
            let (unified, remaining, binding) = unify_prefix(
                lhs.iter().map(|pat| pat.default_age(lhs_age)).collect(),
                rhs.iter().map(|pat| pat.default_age(rhs_age)).collect(),
                binding,
            )?;
            // The shorter one is the one that is now "done", so we match it's tail with
            // the remaining head and tail of the other list.
            let (suffix, binding) = if lhs.len() < rhs.len() {
                unify_patterns_inner(
                    lhs_tail.default_age(lhs_age),
                    Pattern::list(remaining, Some(rhs_tail.default_age(rhs_age))),
                    binding,
                )?
            } else {
                unify_patterns_inner(
                    Pattern::list(remaining, Some(lhs_tail.default_age(lhs_age))),
                    rhs_tail.default_age(rhs_age),
                    binding,
                )?
            };
            Some((Pattern::list(unified, Some(suffix)), binding))
        }
        // If neither record has a tail, the heads must match.
        (PatternKind::Record(lhs, None), PatternKind::Record(rhs, None)) => {
            let (fields, binding) = unify_fields(
                lhs.iter()
                    .map(|(k, v)| (k.clone(), v.default_age(lhs_age)))
                    .collect(),
                rhs.iter()
                    .map(|(k, v)| (k.clone(), v.default_age(rhs_age)))
                    .collect(),
                binding,
            )?;
            Some((Pattern::record(fields, None), binding))
        }
        // If only one record has a tail, the tail unifies with whatever the head does
        // not already cover.
        (PatternKind::Record(.., None), PatternKind::Record(.., Some(..))) => {
            unify_patterns_inner(rhs, lhs, binding)
        }
        (PatternKind::Record(head, Some(tail)), PatternKind::Record(full, None)) => {
            match tail.kind() {
                PatternKind::Variable(ident) => {
                    let (output, tail, binding) = unify_fields_partial(
                        head.iter()
                            .map(|(k, v)| (k.clone(), v.default_age(lhs_age)))
                            .collect(),
                        full.iter()
                            .map(|(k, v)| (k.clone(), v.default_age(rhs_age)))
                            .collect(),
                        binding,
                    )?;
                    let tail_pat = binding.get(&ident.set_current(lhs_age)).unwrap();
                    let (tail, binding) =
                        unify_patterns_inner(tail_pat, Pattern::record(tail, None), binding)?;
                    Some((Pattern::record(output, Some(tail)), binding))
                }
                PatternKind::Record(..) => {
                    panic!("should not reach here... the tails should always be variables")
                }
                // If the tail cannot unify with a record, then there is a problem.
                _ => None,
            }
        }
        // If both records have tails, unify the heads to remove common elements of both, then
        // a record formed from the remaining elements of the other is unified with each tail in
        // turn.
        (
            PatternKind::Record(lhead, Some(lhs_tail)),
            PatternKind::Record(rhead, Some(rhs_tail)),
        ) => {
            let (intersection, lhs_rest, rhs_rest, mut binding) = unify_fields_difference(
                lhead
                    .iter()
                    .map(|(k, v)| (k.clone(), v.default_age(lhs_age)))
                    .collect(),
                rhead
                    .iter()
                    .map(|(k, v)| (k.clone(), v.default_age(rhs_age)))
                    .collect(),
                binding,
            )?;
            let shared_tail =
                Pattern::from(PatternKind::Variable(binding.to_mut().fresh_variable()));
            let mut not_intersection = lhs_rest.clone();
            not_intersection.extend(rhs_rest.clone());
            let complete_tail = Pattern::record(not_intersection, Some(shared_tail));
            let new_lhs_tail = Pattern::record(lhs_rest, Some(lhs_tail.default_age(lhs_age)));
            let new_rhs_tail = Pattern::record(rhs_rest, Some(rhs_tail.default_age(rhs_age)));
            let (_, binding) = unify_patterns_inner(new_lhs_tail, complete_tail.clone(), binding)?;
            let (_, binding) = unify_patterns_inner(new_rhs_tail, complete_tail.clone(), binding)?;
            Some((Pattern::record(intersection, Some(complete_tail)), binding))
        }
        // Otherwise, it's a failure!
        _ => None,
    }
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_sequence(
    lhs: Vec<Pattern>,
    rhs: Vec<Pattern>,
    binding: Cow<'_, Binding>,
) -> Option<(Vector<Pattern>, Cow<'_, Binding>)> {
    if lhs.len() != rhs.len() {
        return None;
    }
    lhs.into_iter().zip(rhs.into_iter()).try_fold(
        (vector![], binding),
        |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns_inner(lhs, rhs, binding)?;
            patterns.push_back(pattern);
            Some((patterns, binding))
        },
    )
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_fields(
    lhs: BTreeMap<Atom, Pattern>,
    rhs: BTreeMap<Atom, Pattern>,
    binding: Cow<'_, Binding>,
) -> Option<(Fields, Cow<'_, Binding>)> {
    if lhs.len() != rhs.len() {
        return None;
    }
    let (fields, binding) = lhs.into_iter().zip(rhs.into_iter()).try_fold(
        (BTreeMap::default(), binding),
        |(mut fields, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns_inner(lhs.1, rhs.1, binding)?;
            fields.insert(lhs.0, pattern);
            Some((fields, binding))
        },
    )?;
    Some((Fields::from(fields), binding))
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_fields_partial(
    part: BTreeMap<Atom, Pattern>,
    mut full: BTreeMap<Atom, Pattern>,
    binding: Cow<'_, Binding>,
) -> Option<(Fields, Fields, Cow<'_, Binding>)> {
    let (fields, binding) = part.into_iter().try_fold(
        (BTreeMap::new(), binding),
        |(mut fields, binding), (key, pattern)| {
            let (unified, binding) = unify_patterns_inner(pattern, full.remove(&key)?, binding)?;
            fields.insert(key, unified);
            Some((fields, binding))
        },
    )?;
    Some((fields.into(), full.into(), binding))
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_fields_difference(
    mut lhs: BTreeMap<Atom, Pattern>,
    mut rhs: BTreeMap<Atom, Pattern>,
    binding: Cow<'_, Binding>,
) -> Option<(Fields, Fields, Fields, Cow<'_, Binding>)> {
    let all_keys: HashSet<_> = lhs.keys().chain(rhs.keys()).cloned().collect();

    let (intersection, lhs_rest, rhs_rest, binding) = all_keys.into_iter().try_fold(
        (BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), binding),
        |(mut intersection, mut lhs_rest, mut rhs_rest, binding), key| match (
            lhs.remove(&key),
            rhs.remove(&key),
        ) {
            (Some(lhs), Some(rhs)) => {
                let (pattern, binding) = unify_patterns_inner(lhs, rhs, binding)?;
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
fn unify_prefix(
    mut lhs: Vec<Pattern>,
    mut rhs: Vec<Pattern>,
    binding: Cow<'_, Binding>,
) -> Option<(Vector<Pattern>, Vector<Pattern>, Cow<'_, Binding>)> {
    let min_len = usize::min(lhs.len(), rhs.len());
    let (head, binding) = lhs.drain(..min_len).zip(rhs.drain(..min_len)).try_fold(
        (vector![], binding),
        |(mut patterns, binding), (lhs, rhs)| {
            let (pattern, binding) = unify_patterns_inner(lhs, rhs, binding)?;
            patterns.push_back(pattern);
            Some((patterns, binding))
        },
    )?;
    if lhs.is_empty() {
        Some((head, rhs.into(), binding))
    } else {
        Some((head, lhs.into(), binding))
    }
}

#[cfg_attr(feature = "test-perf", flamer::flame)]
fn unify_full_prefix(
    lhs: Vec<Pattern>,
    mut rhs: Vec<Pattern>,
    binding: Cow<'_, Binding>,
) -> Option<(Vector<Pattern>, Vector<Pattern>, Cow<'_, Binding>)> {
    if lhs.len() > rhs.len() {
        return None;
    }
    let (head, binding) = rhs.drain(..lhs.len()).zip(lhs.into_iter()).try_fold(
        (vector![], binding),
        |(mut patterns, binding), (rhs, lhs)| {
            let (pattern, binding) = unify_patterns_inner(lhs, rhs, binding)?;
            patterns.push_back(pattern);
            Some((patterns, binding))
        },
    )?;
    Some((head, rhs.into(), binding))
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
        Pattern::from(PatternKind::Struct(Atom::from(name), None))
    }

    fn var(binding: &mut Binding) -> Pattern {
        Pattern::from(PatternKind::Variable(binding.fresh_variable()))
    }

    fn int(val: impl Into<ramp::int::Int>) -> Pattern {
        Pattern::from(PatternKind::Literal(Literal::Integer(val.into())))
    }

    fn rat(val: impl Into<ramp::rational::Rational>) -> Pattern {
        Pattern::from(PatternKind::Literal(Literal::Rational(val.into())))
    }

    fn string(val: impl Into<String>) -> Pattern {
        Pattern::from(PatternKind::Literal(Literal::String(val.into())))
    }

    fn unbound() -> Pattern {
        Pattern::from(PatternKind::Unbound)
    }
    fn bound() -> Pattern {
        Pattern::from(PatternKind::Bound)
    }

    macro_rules! list {
        () => (Pattern::list(vector![], None));
        ($($item:expr),+) => (Pattern::list(vector![$($item.clone()),+], None));
        ($($item:expr),+ ; $rest:expr) => (Pattern::list(vector![$($item.clone()),+], Some($rest.clone())));
    }

    macro_rules! structure {
        (
            $name:ident ($contents:expr)
        ) => {
            Pattern::from(PatternKind::Struct(
                Atom::from(stringify!($name)),
                Some($contents.clone()),
            ))
        };
    }

    macro_rules! record {
        (
            @ [$fields:ident] (.. $rest:expr)
        ) => {{
            Pattern::record($fields.into(), Some($rest.clone()))
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
            Pattern::record($fields.into(), None)
        }};

        ($($field:tt)+) => {{
            #[allow(unused_mut)]
            let mut fields = BTreeMap::default();
            record!(@[fields] ($($field)+))
        }};

        () => { Pattern::record(Default::default(), None) }
    }

    macro_rules! all {
        ($($pat:expr),+) => {
            Pattern::from(PatternKind::All(vec![$($pat.clone()),+]))
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
            record! { a: int(1), ..y },
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
