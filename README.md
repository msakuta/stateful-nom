# stateful-nom

An experimental crate to try memory arena for allocating syntax tree nodes.

## Overview

In [mascal](https://github.com/msakuta/mascal) and few other repos, I tried to make a parser that produces AST with nom.
This method works well in general, but it has some challenges in managing the tree structure afterwards.
In particular, AST tend to be structure like below:

```rust
enum Expression {
    NumLiteral(f64),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    // ...
}
```

The first thing you should note is the repetitiveness of `Box<Expression>`.
In general, Rust is not a great language to manage tree data structure using boxes.

However, boxing is not the only way to define nodes.
There is another way to allocate nodes, which is memory area, hinted by the famous technique called ECS in game programming.
In this approach, the tree is made of 2 things: the nodes array and the pointer (index) to the root node.

```rust
type Nodes = Vec<Node>;
type NodeId = usize;

enum Node {
    NumLiteral(f64),
    Add(NodeId, NodeId),
    Sub(NodeId, NodeId),
}
```

Notice that the pointer to the child nodes are just `usize`s, but their type is typedefed as `NodeId` for documentation.

This data layout helps when you implement type inference, where you would need to modify the tree nodes at random place.

## Type inference

In type inference, each literal and variable can have omitted type.
We indicate this by having `Option<TypeDecl>` as the type annotation.
If the type annotation is omitted, it will be `None`.

```rust
enum Expression {
    NumLiteral(f64, Option<TypeDecl>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    // ...
}

enum TypeDecl {
    I64,
    F64,
    // Str, ...
}
```

For the demonstration purposes, we assume we want to have the same postfix syntax for numeric literals to declare types.
In that case, we may parse expressions like below.

* `123i64` -> `Expression::NumLiteral(123., Some(TypeDecl::I64))`
* `321f64` -> `Expression::NumLiteral(321. Some(TypeDecl::F64))`
* `42` -> `Expression::NumLiteral(42., None)`

However, this is not very convenient data structure to apply type inference, because
you need to traverse the tree from a mutable reference to the root node to apply any change to any node.
It gets particularly annoying when you try to implement Hindley-Milner type inference, where you define a set of type constraints between nodes and resolve them separately.

```rust
struct TypeConstraint<'a> {
    lhs: &'a Expression,
    rhs: &'a Expression
}

type TypeConstraints<'a> = Vec<TypeConstraint<'a>>;
```

In order to make the type inferer able to modify the AST to reflect the result, we will need to wrap type declarations in a `Cell` or `RefCell`.

```rust
enum Expression {
    NumLiteral(f64, Cell<Option<TypeDecl>>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    // ...
}
```

The data structure becomes even more complicated.

If we used memory arena, we can define type constraints like below, and we don't even need generic lifetimes.

```rust
struct TypeConstraint {
    lhs: NodeId,
    rhs: NodeId,
}

type TypeConstraints = Vec<TypeConstraint>;
```

Also, the type declarations won't need to be in a `Cell`, because we can randomly access nodes in the memory arena.

```rust
enum Node {
    NumLiteral(f64, Option<TypeDecl>),
    Add(NodeId, NodeId),
    Sub(NodeId, NodeId),
    // ...
}
```

This repository contains codes to explore this option.

## How to access nodes array from parser functions

`nom` doesn't provide with a way to have contextual information. It assumes the result of a parsing will be self contained
(like the `Expression` tree implementation). In other words, the parsers are pure functions without side effects.

There are a couple of ways to work around this limitation.
One is the global variable, in particular, thread locals, because only one thread will be parsing the same source code at a time.
Another way is to put the reference to `nodes` in the input.
Thankfully, `nom_locate` crate provides with a `LocatedSpan` type with a generic type parameter to store extra information.
We can use a type `LocatedSpan<&str, &RefCell<Vec<Node>>>` to allow any part of the code to have access to global nodes.
It is also thread safe because it points to local variable up somewhere in the stack.
We need `RefCell` because parser input type should be `Copy` and a mutable array is not `Copy`.

It may feel to have no advantage since it needs `RefCell` anyway, but we don't have to wrap individual node types,
unlike the naive tree implementation.
