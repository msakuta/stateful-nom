use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, multispace0},
    combinator::{opt, recognize},
    multi::{fold_many0, many0, many1},
    sequence::{delimited, pair, terminated},
    Finish, IResult, InputTake,
};
use super::{Span, Input, NodeId, Node, TypeDecl};

/// An extension trait for writing subslice concisely
trait Subslice {
    fn subslice(&self, start: usize, length: usize) -> Self;
}

impl<'a> Subslice for Span<'a> {
    fn subslice(&self, start: usize, length: usize) -> Self {
        self.take_split(start).0.take(length)
    }
}

// impl<'a> Subslice for Input<'a> {
//     fn subslice(&self, start: usize, length: usize) -> Self {
//         self.take_split(start).0.take(length)
//     }
// }

fn decimal(input: Input) -> IResult<Input, NodeId> {
    let (r, (res, ty)) = delimited(
        multispace0,
        pair(
            recognize(many1(terminated(digit1, many0(char('_'))))),
            opt(alt((tag("i64"), tag("f64")))),
        ),
        multispace0,
    )(input)?;
    let num = res.parse::<f64>().unwrap();
    let mut nodes = input.extra.borrow_mut();
    let ret = nodes.len();
    nodes.push(Node::NumLiteral(
        num,
        ty.map(|ty| match *ty {
            "i64" => TypeDecl::I64,
            "f64" => TypeDecl::F64,
            _ => unreachable!(),
        }),
    ));
    Ok((r, ret))
}

fn add(i: Input) -> IResult<Input, NodeId> {
    let (r, init) = decimal(i)?;

    fold_many0(
        pair(alt((char('+'), char('-'))), decimal),
        move || init.clone(),
        move |acc, (_op, val): (char, NodeId)| {
            // let span = i.subslice(
            //     i.offset(&acc.span),
            //     acc.span.offset(&val.span) + val.span.len(),
            // );
            let mut nodes = i.extra.borrow_mut();
            let ret = nodes.len();
            nodes.push(Node::Add(acc, val));
            ret
            // if op == '+' {
            // Expression::new(ExprEnum::Add(Box::new(acc), Box::new(val)), span)
            // } else {
            //     Expression::new(ExprEnum::Sub(Box::new(acc), Box::new(val)), span)
            // }
        },
    )(r)
}

pub(crate) fn parse(i: Input) -> IResult<Input, NodeId> {
    add(i)
}
