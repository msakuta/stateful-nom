mod parser;

use std::cell::RefCell;

use crate::parser::parse;
use nom_locate::LocatedSpan;
use nom::Finish;

type Input<'a> = LocatedSpan<&'a str, &'a RefCell<Vec<Node>>>;
type Span<'a> = LocatedSpan<&'a str, &'a RefCell<Vec<Node>>>;

fn main() {
    let nodes = RefCell::new(vec![]);
    let source = Input::new_extra("123i64 + 456 - 789", &nodes);
    let ast = parse(source).finish().unwrap();
    println!("nodes: {nodes:?}");
    print_tree(&nodes.borrow(), ast.1);

    let mut nodes = nodes.borrow_mut();
    let builder = TypeConstraintBuilder::new(&nodes, ast.1);
    let constraints = builder.build().unwrap();
    println!("Constraints: {constraints:?}");
    resolve_types(&mut nodes, &constraints);

    println!("after resolved types:");
    print_tree(&nodes, ast.1);
}

#[derive(Debug, Clone, Copy)]
enum Node {
    NumLiteral(f64, Option<TypeDecl>),
    Add(usize, usize),
}

impl Node {
    fn type_decl(&mut self) -> Option<&mut Option<TypeDecl>> {
        match self {
            Self::NumLiteral(_, ty) => Some(ty),
            _ => None,
        }
    }
}

/// Index into Vec<Node>
type NodeId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeDecl {
    F64,
    I64,
}

fn print_tree(nodes: &[Node], root: NodeId) {
    fn print_tree_int(nodes: &[Node], root: NodeId, indent: usize) {
        let spaces = "  ".repeat(indent) + " ";
        let node = &nodes[root];
        match node {
            Node::NumLiteral(_, _) => {
                println!("[{root:2}]{spaces}{node:?}");
            }
            Node::Add(lhs, rhs) => {
                println!("[{root:2}]{spaces}{:?}", node);
                print_tree_int(nodes, *lhs, indent + 1);
                print_tree_int(nodes, *rhs, indent + 1);
            }
        }
    }
    print_tree_int(nodes, root, 0);
}

fn resolve_types(nodes: &mut [Node], constraints: &[TypeConstraint]) {
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

#[derive(Debug)]
struct TypeConstraint {
    lhs: NodeId,
    rhs: NodeId,
}

struct TypeConstraintBuilder<'a> {
    nodes: &'a Vec<Node>,
    root: NodeId,
    constraints: Vec<TypeConstraint>,
}

impl<'a> TypeConstraintBuilder<'a> {
    fn new(nodes: &'a Vec<Node>, root: NodeId) -> Self {
        Self {
            nodes,
            root,
            constraints: vec![],
        }
    }

    fn forward(&mut self, root: NodeId) -> Result<Option<(NodeId, TypeDecl)>, String> {
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

    fn build(mut self) -> Result<Vec<TypeConstraint>, String> {
        self.forward(self.root)?;
        Ok(self.constraints)
    }
}
