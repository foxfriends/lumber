mod associativity;
mod climbable;
mod op;

pub(crate) use associativity::Associativity;
pub(crate) use climbable::Climbable;
pub(crate) use op::Op;

pub(crate) fn climb<Out, Resolved: Climbable, Infix: Copy + Fn(Out, Resolved, Out) -> Out>(
    mut inputs: impl Iterator<Item = Op<Resolved, Out>>,
    infix: Infix,
) -> Out {
    let lhs = inputs.next().and_then(Op::into_rand).unwrap();
    climb_rec(lhs, 0, &mut inputs.peekable(), infix)
}

fn climb_rec<Out, Resolved: Climbable, P, Infix: Copy + Fn(Out, Resolved, Out) -> Out>(
    mut lhs: Out,
    min_prec: usize,
    inputs: &mut std::iter::Peekable<P>,
    infix: Infix,
) -> Out
where
    P: Iterator<Item = Op<Resolved, Out>>,
{
    while inputs.peek().is_some() {
        let item = inputs.peek().unwrap();
        let prec = match item {
            Op::Rator(rator) => rator.prec(),
            _ => unreachable!(),
        };
        if prec >= min_prec {
            let op = inputs.next().and_then(Op::into_rator).unwrap();
            let mut rhs = inputs.next().and_then(Op::into_rand).unwrap();

            while inputs.peek().is_some() {
                let item = inputs.peek().unwrap();
                let (new_prec, assoc) = match item {
                    Op::Rator(rator) => (rator.prec(), rator.assoc()),
                    _ => unreachable!(),
                };
                if new_prec > prec || assoc == Associativity::Right && new_prec == prec {
                    rhs = climb_rec(rhs, new_prec, inputs, infix);
                } else {
                    break;
                }
            }

            lhs = infix(lhs, op, rhs);
        } else {
            break;
        }
    }

    lhs
}
