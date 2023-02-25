#![allow(unused_imports)]
#![allow(dead_code)]
use std::{collections::HashMap, ops, vec};

#[derive(Debug, Clone)]
pub enum JsonValue {
    JsonNull,
    JsonBool(bool),
    JsonNumber(i64), // TODO(make this parse floats etc later)
    JsonString(String),
    JsonArray(Vec<Box<JsonValue>>),
    JsonObject(HashMap<String, Box<JsonValue>>),
}

#[derive(Debug, PartialEq)]
pub struct ParseResult<T> {
    value: T,
    s: String, // stores the rest of unparsed string
}

// Parser is a Monad
pub trait Parser<T> {
    fn parse(&self, s: String) -> Option<ParseResult<T>>;
}

impl<T, F> Parser<T> for F
where
    F: Fn(String) -> Option<ParseResult<T>>,
{
    fn parse(&self, s: String) -> Option<ParseResult<T>> {
        self(s)
    }
}

fn unit_p<T: Clone>(value: T) -> impl Parser<T> {
    move |input: String| {
        Some(ParseResult {
            value: value.clone(),
            s: input.clone(),
        })
    }
}

// fn flatmap<A, B, F>(pa: impl Parser<A>, f: F) -> impl Parser<B>
// where
//     F: Fn(A) -> dyn Parser<B>,
// {
//     move |input: String| {
//         pa.parse(input)
//             .and_then(|ParseResult { value, s }| f(value).parse(s))
//     }
// }

fn string_p(s: String) -> impl Parser<String> {
    move |input: String| {
        if input.starts_with(&s) {
            Some(ParseResult {
                value: s.clone(),
                s: input[s.len()..].to_owned(),
            })
        } else {
            None
        }
    }
}

fn while_p<F>(f: F) -> impl Parser<String>
where
    F: Fn(&char) -> bool,
{
    move |input: String| {
        let parsed: String = input.chars().take_while(|c| f(c)).collect();
        let rest: String = input.chars().skip_while(|c| !f(c)).collect();
        Some(ParseResult {
            value: parsed,
            s: rest,
        })
    }
}

fn map2<A, B, C, F>(pa: impl Parser<A>, pb: impl Parser<B>, f: F) -> impl Parser<C>
where
    F: Fn(A, B) -> C,
{
    move |input: String| {
        pa.parse(input).and_then(
            |ParseResult {
                 value: a,
                 s: input_a,
             }| {
                pb.parse(input_a).and_then(
                    |ParseResult {
                         value: b,
                         s: input_b,
                     }| {
                        Some(ParseResult {
                            value: f(a, b),
                            s: input_b,
                        })
                    },
                )
            },
        )
    }
}

fn fmap<A, B, F>(pa: impl Parser<A>, f: F) -> impl Parser<B>
where
    F: Fn(A) -> B,
{
    move |input: String| {
        pa.parse(input)
            .map(|ParseResult { value, s }| ParseResult { value: f(value), s })
    }
}

fn or<A>(pa1: impl Parser<A>, pa2: impl Parser<A>) -> impl Parser<A> {
    move |input: String| pa1.parse(input.clone()).or(pa2.parse(input))
}

fn many<A: Clone, F>(pa: F) -> impl Parser<Vec<A>>
where F: Fn(String) -> Option<ParseResult<A>>
{
    move |input: String| match pa.parse(input.clone()) {
        Some(ParseResult { value, s }) => map2(unit_p(value), many(pa), |value, vs| {
            let mut v = vec![value];
            v.extend(vs);
            v
        })
        .parse(s),
        None => Some(ParseResult {
            value: Vec::new(),
            s: input,
        }),
    }
}

// fn many1<A: Clone>(pa: impl Parser<A>) -> impl Parser<Vec<A>> {
//     map2(
//         pa,
//         or(many(pa), unit_p(Vec::new())),
//         |x, mut xs| {
//             xs.insert(0, x);
//             xs
//         }
//     )
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_p_success() {
        let p1 = string_p("hel".to_owned());
        assert_eq!(
            p1.parse("hello".to_owned()),
            Some(ParseResult {
                value: "hel".to_owned(),
                s: "lo".to_owned(),
            })
        );
    }

    #[test]
    fn test_string_p_fail() {
        let p = string_p("fre".to_owned());
        assert!(p.parse("friend".to_owned()).is_none());
    }

    #[test]
    fn test_constant_p_int() {
        let p = unit_p(1);
        assert_eq!(
            p.parse("hello".to_owned()),
            Some(ParseResult {
                value: 1,
                s: "hello".to_owned(),
            })
        );
    }

    #[test]
    fn test_constant_p_string() {
        let p = unit_p("world".to_owned());
        assert_eq!(
            p.parse("hello".to_owned()),
            Some(ParseResult {
                value: "world".to_owned(),
                s: "hello".to_owned(),
            })
        );
    }

    #[test]
    fn test_map2() {
        // test chaining 2 parsers together using map2
        let pa = string_p("hello".to_owned());
        let pb = string_p("world".to_owned());
        let f = |a: String, b: String| a.clone() + &b;
        let pc = map2(pa, pb, f);
        assert_eq!(
            pc.parse("helloworldfriend".to_owned()),
            Some(ParseResult {
                value: "helloworld".to_owned(),
                s: "friend".to_owned(),
            })
        );
    }

    #[test]
    fn test_or() {
        // test chaining 2 parsers together using map2
        let pa = string_p("hello".to_owned());
        let pb = string_p("world".to_owned());
        let pc = or(pa, pb);
        assert_eq!(
            pc.parse("hello".to_owned()),
            Some(ParseResult {
                value: "hello".to_owned(),
                s: "".to_owned(),
            })
        );
        assert_eq!(
            pc.parse("world".to_owned()),
            Some(ParseResult {
                value: "world".to_owned(),
                s: "".to_owned(),
            })
        );
    }

    fn test_fmap() {
        let pa = string_p("hello".to_owned());
        let pb = fmap(pa, |s| s.len());
        assert_eq!(
            pb.parse("hello".to_owned()),
            Some(ParseResult {
                value: 5,
                s: "".to_owned(),
            })
        );
    }

    fn test_while_p() {
        let pa = while_p(|&c| c == 'h');
        assert_eq!(
            pa.parse("hhhello".to_owned()),
            Some(ParseResult {
                value: "hhh".to_owned(),
                s: "ello".to_owned(),
            })
        );
    }

    fn test_many() {
        let pa = string_p("a".to_owned());
        let pb = many(pa);
        let expected_value = vec!['a', 'a', 'a', 'a', 'a'];
        if let Some(ParseResult{value, s}) = pb.parse("aaaaabbbb".to_owned()) {
            assert_eq!(value.iter().zip(&expected_value).filter(|&(a, b)| a.chars().nth(0).unwrap() == *b).count(), 5);
            assert_eq!(s, "bbbb".to_owned());
        } else {
            panic!("parse failed!");
        }
    }
}
