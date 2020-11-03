use crate::ast::*;
use crate::Binding;
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
            binding.set(var.clone(), pattern.clone());
            Some((pattern, binding))
        }
        // Literals must match exactly.
        (Pattern::Literal(lhs), Pattern::Literal(rhs)) if lhs == rhs => {
            Some((Pattern::Literal(lhs.clone()), binding.clone()))
        }
        (Pattern::Literal(..), Pattern::Literal(..)) => None,
        // Structs must match in name and arity, and then all their fields must match as well.
        (Pattern::Struct(lhs), Pattern::Struct(rhs))
            if lhs.name == rhs.name
                && lhs.patterns.len() == rhs.patterns.len()
                && lhs.fields.similar(&rhs.fields) =>
        {
            let (patterns, binding) =
                unify_sequence(&lhs.patterns, &rhs.patterns, binding, occurs)?;
            let (fields, binding) = unify_fields(&lhs.fields, &rhs.fields, binding, occurs)?;
            Some((
                Pattern::Struct(Struct {
                    name: lhs.name.clone(),
                    patterns,
                    fields,
                }),
                binding,
            ))
        }
        (Pattern::Struct(..), Pattern::Struct(..)) => None,
        // If neither list has a tail, the heads must match, similar to struct fields.
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
                    let (output, tail, binding) = unify_prefix(head, full, binding, occurs)?;
                    let tail_pat = binding.get(ident).unwrap().clone();
                    let mut occurs = occurs.to_owned();
                    occurs.push(ident.clone());
                    let (tail, binding) =
                        unify_patterns(&Pattern::List(tail, None), &tail_pat, binding, &occurs)?;
                    Some((Pattern::List(output, Some(Box::new(tail))), binding))
                }
                Pattern::Wildcard => {
                    let (mut output, mut tail, binding) =
                        unify_prefix(head, full, binding, occurs)?;
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
    if !lhs.similar(rhs) {
        return None;
    }
    let (fields, binding) = lhs.iter().zip(rhs.iter()).try_fold(
        (vec![], binding),
        |(mut fields, binding), (lhs, rhs)| {
            let (patterns, binding) = unify_sequence(&lhs.1, &rhs.1, binding, occurs)?;
            fields.push((lhs.0.clone(), patterns));
            Some((fields, binding))
        },
    )?;
    Some((fields.into_iter().collect(), binding))
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

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;

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
        Pattern::Struct(Struct {
            name: Atom::from(name),
            patterns: vec![],
            fields: Fields::default(),
        })
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
        ($($item:expr),+) => (Pattern::List(vec![$($item.clone()),+], None));
        ($($item:expr),+ ; $rest:expr) => (Pattern::List(vec![$($item.clone()),+], Some(Box::new($rest.clone()))));
    }

    macro_rules! structure {
        (
            $name:ident ( $($field:tt)+ )
        ) => {{
            #[allow(unused_mut)]
            let mut patterns = vec![];
            #[allow(unused_mut)]
            let mut fields = BTreeMap::default();
            structure!(@[patterns, fields] $name ($($field)+))
        }};

        (
            @ [$patterns:ident, $fields:ident] $name:ident ( $fieldname:ident: $pat:expr $(, $($field:tt)+)? )
        ) => {{
            #[allow(unused_mut)]
            let mut field_values = vec![$pat.clone()];
            structure!(@ $fieldname [$patterns, $fields, field_values] $name ($($($field)+)?))
        }};

        (
            @ [$patterns:ident, $fields:ident] $name:ident ( $pat:expr $(, $($field:tt)+)? )
        ) => {{
            $patterns.push($pat.clone());
            structure!(@[$patterns, $fields] $name ($($($field)+)?))
        }};

        (
            @ $fieldname:ident [$patterns:ident, $fields:ident, $field_values:ident] $name:ident ( $nextfield:ident: $pat:expr $(, $($field:tt)+)? )
        ) => {{
            $fields.insert(Atom::from(stringify!($fieldname)), $field_values);
            #[allow(unused_mut)]
            let mut field_values = vec![$pat.clone()];
            structure!(@ $nextfield [$patterns, $fields, field_values] $name ($($($field)+)?))
        }};

        (
            @ $fieldname:ident [$patterns:ident, $fields:ident, $field_values:ident] $name:ident ( $pat:expr $(, $($field:tt)+)? )
        ) => {{
            $field_values.push($pat.clone());
            structure!(@ $fieldname [$patterns, $fields, $field_values] $name ($($($field)+)?))
        }};

        (
            @[$patterns:ident, $fields:ident] $name:ident ()
        ) => {
            Pattern::Struct(Struct::from_parts(
                Atom::from(stringify!($name)),
                $patterns,
                $fields.into_iter().collect(),
            ))
        };

        (
            @ $fieldname:ident [$patterns:ident, $fields:ident, $field_values:ident] $name:ident ()
        ) => {{
            $fields.insert(Atom::from(stringify!($fieldname)), $field_values);
            Pattern::Struct(Struct::from_parts(
                Atom::from(stringify!($name)),
                $patterns,
                $fields.into_iter().collect(),
            ))
        }};
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
            structure!(hello(atom("hello"), list![rat(1)])),
            structure!(hello(atom("hello"), list![rat(1)])),
        );

        yes!(
            structure!(hello(WILD, list![rat(1)])),
            structure!(hello(atom("hello"), WILD)),
        );

        yes!(
            structure!(hello(int(3), a: int(1), b: int(2))),
            structure!(hello(int(3), a: int(1), b: int(2))),
        );

        yes!(
            structure!(hello(int(3), a: int(1), b: int(2))),
            structure!(hello(int(3), b: int(2), a: int(1))),
        );
    }

    #[test]
    fn no_unify_struct() {
        no!(
            structure!(hello(atom("world"))),
            structure!(world(atom("world"))),
        );

        no!(
            structure!(hello(atom("hello"), list![rat(1)])),
            structure!(hello(atom("world"), list![rat(1)])),
        );

        no!(
            structure!(hello(int(3), a: int(1), b: int(2))),
            structure!(hello(int(3), b: int(1), a: int(2))),
        );

        no!(
            structure!(hello(int(3), a: int(1))),
            structure!(hello(int(3), b: int(1), a: int(1))),
        );

        no!(
            structure!(hello(int(3), b: int(1), a: int(1))),
            structure!(hello(int(3), a: int(1))),
        );

        no!(
            structure!(hello(int(3), b: int(1))),
            structure!(hello(int(3), a: int(1))),
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
        yes!(x, structure!(hello(int(3), rat(4))), binding);
        yes!(x, list![int(3), int(4)], binding);
        yes!(x, list![y], binding);
    }

    #[test]
    fn unify_multiple_variables() {
        let mut binding = Binding::default();
        let x = id("x", &mut binding);
        let y = id("y", &mut binding);
        yes!(structure!(test(x, y)), WILD, binding);
        yes!(structure!(test(x, y)), structure!(test(int(3), x)), binding);
        yes!(
            structure!(test(x, y)),
            structure!(test(int(3), int(4))),
            binding
        );
        no!(
            structure!(test(x, x)),
            structure!(test(int(3), int(4))),
            binding
        );
        yes!(
            structure!(test(int(3), y)),
            structure!(test(x, int(4))),
            binding
        );
        no!(
            structure!(test(int(3), x)),
            structure!(test(x, int(4))),
            binding
        );
        yes!(list![x, y], list![int(1), int(2)], binding);
        no!(list![x, x], list![int(1), int(2)], binding);
        yes!(list![x; x], list![list![int(1)], int(1)], binding);
        no!(list![x; x], list![int(1), int(1)], binding);
        yes!(list![int(1); x], list![int(1), int(2); y], binding);
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
    }

    #[test]
    fn no_unify_list() {
        no!(list![int(1), int(2)], list![int(3), int(4)]);
        no!(list![int(1), int(2)], list![int(3); WILD]);
        no!(list![int(1)], list![int(1), int(2)]);
    }
}
