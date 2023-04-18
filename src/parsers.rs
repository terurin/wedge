use combine::error::StreamError;

use combine::parser::{
    char::string, combinator::recognize, repeat::skip_many, repeat::skip_many1, token::token,
};
use combine::stream::StreamErrorFor;
use combine::{attempt, choice, one_of, optional, satisfy, value, ParseError, Parser, Stream};

pub fn integer<Input>() -> impl Parser<Input, Output = i64>
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
#[cfg(test)]
mod tests {
    use combine::Parser;

    use crate::parsers::integer;

    #[test]
    fn integer_base_test() {
        assert_eq!(integer().parse("0b10").unwrap().0, 2);
        assert_eq!(integer().parse("0q10").unwrap().0, 4);
        assert_eq!(integer().parse("0o10").unwrap().0, 8);
        assert_eq!(integer().parse("0d10").unwrap().0, 10);
        assert_eq!(integer().parse("0x10").unwrap().0, 16);
        assert_eq!(integer().parse("10").unwrap().0, 10);
    }
    
    #[test]
    fn integer_sign_test() {
        assert_eq!(integer().parse("99").unwrap().0, 99);
        assert_eq!(integer().parse("+99").unwrap().0, 99);
        assert_eq!(integer().parse("-99").unwrap().0, -99);
    }

    #[test]
    fn integer_skip_test() {
        assert_eq!(integer().parse("1_1").unwrap().0, 11);
        assert_eq!(integer().parse("1__1").unwrap().0, 11);
        assert!(integer().parse("__1").is_err());
    }

    #[test]
    fn integer_overflow_test() {
        assert!(integer().parse("0x1_1111_1111_1111_1111").is_err());   
    }

}
