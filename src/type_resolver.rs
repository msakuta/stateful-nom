use crate::{Node, type_constraint::TypeConstraint};

pub(crate) fn resolve_types(nodes: &mut [Node], constraints: &[TypeConstraint]) {
    for constraint in constraints {
        let (lhs, rhs) = if constraint.lhs < constraint.rhs {
            let (left, right) = nodes.split_at_mut(constraint.rhs);
            (&mut left[constraint.lhs], &mut right[0])
        } else if constraint.rhs < constraint.lhs {
            let (left, right) = nodes.split_at_mut(constraint.lhs);
            (&mut right[0], &mut left[constraint.rhs])
        } else {
            panic!("Constraint on the same node");
        };
        if let Some((lhs, rhs)) = lhs.type_decl().zip(rhs.type_decl()) {
            match (*lhs, *rhs) {
                (Some(lhs), Some(rhs)) => {
                    if lhs != rhs {
                        panic!();
                    }
                }
                (None, Some(rhs)) => *lhs = Some(rhs),
                (Some(lhs), None) => *rhs = Some(lhs),
                _ => panic!("All type annotations should be bound"),
            }
        }
    }
}
