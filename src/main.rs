use nom::{
    branch::alt,
    character::complete::{char, digit1, multispace0},
    combinator::recognize,
    multi::{fold_many0, many0, many1},
    sequence::{delimited, pair, terminated},
    Finish, IResult, InputTake, Offset,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

fn main() {
    let source = Span::new("123 + 456");
    let ast = add(source).finish().unwrap();
    println!("parsed: {ast:?}");
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expression<'a> {
    pub(crate) expr: ExprEnum<'a>,
    pub(crate) span: Span<'a>,
}

impl<'a> Expression<'a> {
    fn new(expr: ExprEnum<'a>, span: Span<'a>) -> Self {
        Self { expr, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ExprEnum<'a> {
    NumLiteral(f64),
    Add(Box<Expression<'a>>, Box<Expression<'a>>),
}

/// An extension trait for writing subslice concisely
trait Subslice {
    fn subslice(&self, start: usize, length: usize) -> Self;
}

impl<'a> Subslice for Span<'a> {
    fn subslice(&self, start: usize, length: usize) -> Self {
        self.take_split(start).0.take(length)
    }
}

fn decimal(input: Span) -> IResult<Span, Expression> {
    let (r, res) = delimited(
        multispace0,
        recognize(many1(terminated(digit1, many0(char('_'))))),
        multispace0,
    )(input)?;
    let num = res.parse::<f64>().unwrap();
    Ok((
        r,
        Expression {
            expr: ExprEnum::NumLiteral(num),
            span: input,
        },
    ))
}

fn add(i: Span) -> IResult<Span, Expression> {
    let (r, init) = decimal(i)?;

    fold_many0(
        pair(alt((char('+'), char('-'))), decimal),
        move || init.clone(),
        move |acc, (_op, val): (char, Expression)| {
            let span = i.subslice(
                i.offset(&acc.span),
                acc.span.offset(&val.span) + val.span.len(),
            );
            // if op == '+' {
            Expression::new(ExprEnum::Add(Box::new(acc), Box::new(val)), span)
            // } else {
            //     Expression::new(ExprEnum::Sub(Box::new(acc), Box::new(val)), span)
            // }
        },
    )(r)
}
