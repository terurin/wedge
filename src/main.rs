use combine::error::StreamError;
use combine::parser::range::recognize_with_value;
use combine::parser::{
    char::digit, char::string, combinator::recognize, repeat::many1, repeat::sep_by1,
    repeat::skip_many, repeat::skip_many1, token::token,
};
use combine::stream::StreamErrorFor;
use combine::{
    attempt, choice, one_of, optional, parser, satisfy, unexpected, value, ParseError, Parser,
    Stream,
};
use std::collections::BTreeSet;
use std::option;

fn integer<Input>() -> impl Parser<Input, Output = i64>
where
    Input: Stream<Token = char>,
    // Necessary due to rust-lang/rust#24159
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let sign = || optional(one_of("+-".chars())).map(|s| s == Some('-'));

    let base = || {
        choice!(
            attempt(string("0b").with(value(2))),
            attempt(string("0q").with(value(4))),
            attempt(string("0o").with(value(8))),
            attempt(string("0d").with(value(10))),
            attempt(string("0x").with(value(16))),
            value(10)
        )
    };

    (sign(), base()).then(|(sign, base)| {
        let digit_n = |n: u32| satisfy(move |c: char| c.is_digit(n));

        recognize((
            skip_many1(digit_n(base)),
            skip_many((skip_many1(token('_')), skip_many1(digit_n(base)))),
        ))
        .and_then(move |digits: String| {
            
            let number = digits
                .chars()
                .into_iter()
                .try_fold(0i64, |value, digit: char| {
                    if digit == '_' {
                        return Some(value);
                    };

                    value
                        .checked_mul(base as i64)
                        .map(move |x| match sign {
                            false => x.checked_add(digit.to_digit(base).unwrap() as i64),
                            true => x.checked_sub(digit.to_digit(base).unwrap() as i64),
                        })
                        .flatten()
                });
            match number {
                Some(x) => Ok(x),
                None => Err(StreamErrorFor::<Input>::unexpected_static_message(
                    "overflow",
                )),
            }
        })
    })
}

fn main() {
    println!("{:?}", integer().parse("-0x11111111_1A111111"));

    assert!(integer().parse("+0d1__9__1").is_ok());
    assert!(integer().parse("+0b1__0__1").is_ok());
}
