use super::expr::Expr;
use std::result;

#[derive(Debug)]
pub enum Error {
    UnexpectedEOF,
    ExpectedAsciiDigit(u8),
    ExpectedOperator(u8),
    ExpectedOpenParen(u8),
    ExpectedCloseParen(u8),
    ExpectedBytesOrState(u8),
    ExpectedEof(Vec<u8>),
}

pub type Result<T> = result::Result<T, Error>;

pub fn parse(text: &str) -> Result<Expr> {
    let mut bytes = Vec::new();
    for byte in text.as_bytes() {
        if !byte.is_ascii_whitespace() {
            bytes.push(*byte);
        }
    }
    let (text, expr) = parse_expr(&bytes)?;

    if text.is_empty() {
        Ok(expr)
    } else {
        Err(Error::ExpectedEof(text.to_vec()))
    }
}

fn parse_expr(text: &[u8]) -> Result<(&[u8], Expr)> {
    parse_binary_operator(text)
        .or_else(|_| parse_const(text))
        .or_else(|_| parse_ref(text))
}

fn parse_ref(text: &[u8]) -> Result<(&[u8], Expr)> {
    not_empty(text)?;
    match text {
        [b'b', b'y', b't', b'e', text @ ..] => Ok((text, Expr::Byte)),
        [b's', b't', b'a', b't', b'e', text @ ..] => Ok((text, Expr::HashState)),
        _ => Err(Error::ExpectedBytesOrState(text[0])),
    }
}

fn parse_const(mut text: &[u8]) -> Result<(&[u8], Expr)> {
    not_empty(text)?;

    if !text[0].is_ascii_digit() {
        return Err(Error::ExpectedAsciiDigit(text[0]));
    }

    let mut num = 0;

    while !text.is_empty() && text[0].is_ascii_digit() {
        num = 10 * num + text[0] as u64 - 48;
        text = &text[1..];
    }

    Ok((text, Expr::Const(num)))
}

type Operator = fn(Box<Expr>, Box<Expr>) -> Expr;

fn parse_binary_operator(mut text: &[u8]) -> Result<(&[u8], Expr)> {
    not_empty(text)?;

    if text[0] != b'(' {
        return Err(Error::ExpectedOpenParen(text[0]));
    }

    let (text, left) = parse_expr(&text[1..])?;

    let (text, op) = match text {
        [b'+', text @ ..] => (text, (Expr::Add) as Operator),
        [b'x', b'o', b'r', text @ ..] => (text, (Expr::Xor) as Operator),
        [b'>', b'>', text @ ..] => (text, (Expr::RotRight) as Operator),
        [b'<', b'<', text @ ..] => (text, (Expr::RotLeft) as Operator),
        [c, ..] => return Err(Error::ExpectedOperator(*c)),
        [] => return Err(Error::UnexpectedEOF),
    };

    let (text, right) = parse_expr(text)?;

    not_empty(text)?;

    if text[0] != b')' {
        return Err(Error::ExpectedCloseParen(text[0]));
    }

    Ok((&text[1..], op(Box::new(left), Box::new(right))))
}

fn not_empty(text: &[u8]) -> Result<()> {
    if text.is_empty() {
        Err(Error::UnexpectedEOF)
    } else {
        Ok(())
    }
}
