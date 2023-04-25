use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::convert_error;
use nom::multi::many1;
use nom::multi::*;
use nom::*;
use nom::{self, error::Error};
use nom_locate::{position, LocatedSpan};

type Span<'a> = LocatedSpan<&'a str>;
#[derive(Debug)]
pub struct Token<'a> {
    position: Span<'a>,
    value: TokenValue,
}
#[derive(Debug)]
pub enum TokenValue {
    Int(i64),
}

fn integer(s: Span) -> IResult<Span, Token> {
    let (s, pos) = position(s)?;

    let (s, sign) = opt(one_of("+-"))(s)?;
    let (s, base) = opt(alt((tag("0b"), tag("0q"), tag("0q"), tag("0d"), tag("0x"))))(s)?;
    let base = base.map_or(10, |b| match *b.fragment() {
        "0b" => 2,
        "0q" => 4,
        "0o" => 8,
        "0d" => 10,
        "0x" => 16,
        _ => unreachable!(),
    });

    let (s, number) = map_opt(
        recognize(separated_list1(
            many1(tag("_")),
            many1(satisfy(|c: char| c.is_digit(base))),
        )),
        |digits: Span| {
            let digits = digits;
            if sign == Some('-') {
                digits
                    .chars()
                    .filter(|c| *c != '_')
                    .try_fold(0i64, move |init, digit| {
                        let digit = digit.to_digit(base)? as i64;
                        Some(init.checked_mul(base as i64)?.checked_sub(digit)?)
                    })
            } else {
                digits
                    .chars()
                    .filter(|c| *c != '_')
                    .try_fold(0i64, move |init, digit| {
                        let digit = digit.to_digit(base)? as i64;
                        Some(init.checked_mul(base as i64)?.checked_add(digit)?)
                    })
            }
        },
    )(s)?;
    Ok((
        s,
        Token {
            position: pos,
            value: TokenValue::Int(number),
        },
    ))
}

fn main() {
    let stdin = std::io::stdin();
    let mut line = String::new();
    while let Ok(_size) = stdin.read_line(&mut line) {
        println!("{:?}", integer(Span::new(&line)));
        line.clear();
    }
}
