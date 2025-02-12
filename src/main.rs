mod parser;
mod type_constraint;
mod type_resolver;

use std::cell::RefCell;

use crate::{
    parser::parse,
    type_constraint::{TypeConstraint, TypeConstraintBuilder},
    type_resolver::resolve_types,
};
use nom::Finish;
use nom_locate::LocatedSpan;

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
