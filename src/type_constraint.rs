use crate::{Node, NodeId, TypeDecl};

#[derive(Debug)]
pub(crate) struct TypeConstraint {
    pub lhs: NodeId,
    pub rhs: NodeId,
}

pub(crate) struct TypeConstraintBuilder<'a> {
    nodes: &'a Vec<Node>,
    root: NodeId,
    constraints: Vec<TypeConstraint>,
}

impl<'a> TypeConstraintBuilder<'a> {
    pub(crate) fn new(nodes: &'a Vec<Node>, root: NodeId) -> Self {
        Self {
            nodes,
            root,
            constraints: vec![],
        }
    }

    pub(crate) fn forward(&mut self, root: NodeId) -> Result<Option<(NodeId, TypeDecl)>, String> {
        Ok(match self.nodes[root] {
            Node::NumLiteral(_, Some(ty)) => Some((root, ty)),
            Node::NumLiteral(_, _) => None,
            Node::Add(lhs, rhs) => {
                let lhs_res = self.forward(lhs)?;
                let rhs_res = self.forward(rhs)?;
                match (lhs_res, rhs_res) {
                    (Some((lhs, lhty)), Some((rhs, rhty))) => {
                        if lhty != rhty {
                            return Err(format!("Type conflict between node {lhs} and {rhs}"));
                        } else {
                            None
                        }
                    }
                    (None, Some(rhs)) => {
                        self.constraints.push(TypeConstraint { lhs, rhs: rhs.0 });
                        Some(rhs)
                    }
                    (Some(lhs), None) => {
                        self.constraints.push(TypeConstraint { lhs: lhs.0, rhs });
                        Some(lhs)
                    }
                    _ => None,
                }
            }
        })
    }

    pub(crate) fn build(mut self) -> Result<Vec<TypeConstraint>, String> {
        self.forward(self.root)?;
        Ok(self.constraints)
    }
}
