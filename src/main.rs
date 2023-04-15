use nom;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::many1;
use nom::multi::*;
use nom::*;
use std::io::*;

fn integer(s: &str) -> IResult<&str, i64> {
    //let mut result=0;
    let (s, sign) = opt(one_of("+-"))(s)?;
    let (s, base) = map(
        opt(alt((tag("0b"), tag("0q"), tag("0o"), tag("0d"), tag("0x")))),
        |base| match base {
            Some("0b") => 2,
            Some("0q") => 4,
            Some("0o") => 8,
            Some("0d") | None => 10,
            Some("0x") => 16,
            Some(_) => unreachable!(),
        },
    )(s)?;

    let (s, number) = map_opt(
        separated_list1(many1(char('_')), many1(satisfy(|c: char| c.is_digit(base)))),
        |digits| {
            let mut result = 0i64;
            if sign != Some('-') {
                for digit in digits.into_iter().flatten() {
                    result = result.checked_mul(base as i64)?;
                    result = result.checked_add(digit.to_digit(16).unwrap() as i64)?;
                }
            } else {
                for digit in digits.into_iter().flatten() {
                    result = result.checked_mul(base as i64)?;
                    result = result.checked_sub(digit.to_digit(16).unwrap() as i64)?;
                }
            }
            Some(result)
        },
    )(s)?;

    Ok((s, number))
}

fn main() {
    let stdin = stdin();
    let mut line = String::new();
    while let Ok(_size) = stdin.read_line(&mut line) {
        println!("{:?}", integer(&line));
        line.clear();
    }
}
