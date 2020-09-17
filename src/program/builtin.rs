use super::*;

macro_rules! op_2 {
    ($name:ident) => {
        pub(crate) fn $name(
            lhs: Pattern,
            rhs: Pattern,
            output: Pattern,
            context: &mut Context,
        ) -> Unification {
            let scope = Scope::builtin(stringify!($name), context);
            let handle = Handle::binop(scope);
            Unification::Query(Query::new(handle, vec![lhs, rhs, output]))
        }
    };
}

pub(crate) fn unify(lhs: Pattern, rhs: Pattern, context: &mut Context) -> Unification {
    let scope = Scope::builtin("unify", context);
    let handle = Handle::from_parts(scope, vec![Arity::Len(2.into())]);
    Unification::Query(Query::new(handle, vec![lhs, rhs]))
}

op_2!(add);
op_2!(sub);
op_2!(mul);
op_2!(div);
op_2!(rem);
op_2!(exp);
op_2!(eq);
op_2!(neq);
op_2!(lt);
op_2!(gt);
op_2!(leq);
op_2!(geq);
op_2!(or);
op_2!(and);
op_2!(bitor);
op_2!(bitand);
op_2!(bitxor);
