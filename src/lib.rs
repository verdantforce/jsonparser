#![allow(unused_imports)]
#![allow(dead_code)]
use std::{collections::HashMap, ops, string, vec};
// use std::ops::{Shl, Shr};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    JsonNull,
    JsonBool(bool),
    JsonNumber(u64), // TODO(make this parse floats etc later)
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
    // leave the string unparsed and inject value
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
        let rest: String = input.chars().skip_while(|c| f(c)).collect();
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

// fn many<A: Clone>(pa: impl Parser<A>) -> impl Parser<Vec<A>>
// {
//     move |input: String| match pa.parse(input.clone()) {
//         Some(ParseResult { value, s }) => map2(unit_p(value), many(pa), |value, vs| {
//             let mut v = vec![value];
//             v.extend(vs);
//             v
//         })
//         .parse(s),
//         None => Some(ParseResult {
//             value: Vec::new(),
//             s: input,
//         }),
//     }
// }

fn many1<A: Clone>(pa: impl Parser<A>) -> impl Parser<Vec<A>> {
    move |input: String| {
        let mut v = Vec::new();
        let mut next_input = input.clone();

        while let Some(ParseResult { value, s }) = pa.parse(next_input.clone()) {
            next_input = s;
            v.push(value);
        }
        Some(ParseResult {
            value: v,
            s: next_input,
        })
    }
}

fn left<A>(pa: impl Parser<A>, pb: impl Parser<A>) -> impl Parser<A> {
    // run both parsers but keep the result from the left parser
    map2(pa, pb, |a, _b| a)
}

fn right<A>(pa: impl Parser<A>, pb: impl Parser<A>) -> impl Parser<A> {
    // run both parsers but keep the result from the right parser
    map2(pa, pb, |_a, b| b)
}

fn parse_null() -> impl Parser<JsonValue> {
    fmap(string_p("null".to_owned()), |_s: String| {
        JsonValue::JsonNull
    })
}

fn parse_bool() -> impl Parser<JsonValue> {
    fmap(
        or(string_p("true".to_owned()), string_p("false".to_owned())),
        |s: String| match &s[..] {
            "true" => JsonValue::JsonBool(true),
            _ => JsonValue::JsonBool(false),
        },
    )
}

fn parse_quote() -> impl Parser<String> {
    string_p("\"".to_string())
}

fn parse_string() -> impl Parser<JsonValue> {
    // TODO: handle escaped quotes
    // TODO: make it so that this can be written as parse_quote() >> while_p(|&c| c != '"') << parse_quote()
    fmap(
        left(right(parse_quote(), while_p(|&c| c != '"')), parse_quote()),
        |s| JsonValue::JsonString(s),
    )
}

fn parse_number() -> impl Parser<JsonValue> {
    // TODO: make this better
    // TODO: should there be and_then on Parser itself?
    move |input: String| {
        while_p(|&c| c.is_digit(10))
            .parse(input)
            .and_then(|ParseResult { value, s }| {
                if !value.is_empty() {
                    Some(ParseResult {
                        value: JsonValue::JsonNumber(value.parse().unwrap()),
                        s: s,
                    })
                } else {
                    None
                }
            })
    }
}

fn parse_json1() -> impl Parser<JsonValue> {
    // TODO: make it so that it is parse_null() | parse_bool() | parse_number() ...
    or(
        or(or(parse_null(), parse_bool()), parse_number()),
        parse_string(),
    )
}

pub fn parse_json(input: String) -> Option<JsonValue> {
    parse_json1().parse(input).and_then(
        |ParseResult { value, s }| {
            if s.is_empty() {
                Some(value)
            } else {
                None
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use JsonValue::*;

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

    #[test]
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

    #[test]
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

    // #[test]
    // fn test_many() {
    //     let pa = string_p("a".to_owned());
    //     let pb = many(pa);
    //     let expected_value:Vec<String> = vec!["a".to_string(); 5];
    //     if let Some(ParseResult{value, s}) = pb.parse("aaaaabbbb".to_owned()) {
    //         assert_eq!(value.join(""), "aaaaa");
    //         assert_eq!(s, "bbbb".to_owned());
    //     } else {
    //         panic!("parse failed!");
    //     }
    // }

    #[test]
    fn test_many1() {
        let pa = string_p("a".to_owned());
        let pb = many1(pa);
        if let Some(ParseResult { value, s }) = pb.parse("aaaaabbbb".to_owned()) {
            assert_eq!(value.join(""), "aaaaa");
            assert_eq!(s, "bbbb".to_owned());
        } else {
            panic!("parse failed!");
        }
    }

    #[test]
    fn test_parse_null() {
        assert_eq!(
            parse_null().parse("null_hello".to_string()),
            Some(ParseResult {
                value: JsonNull,
                s: "_hello".to_owned(),
            })
        );
        assert_eq!(parse_null().parse("hello".to_string()), None,);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(
            parse_bool().parse("true_hello".to_string()),
            Some(ParseResult {
                value: JsonBool(true),
                s: "_hello".to_owned(),
            })
        );
        assert_eq!(
            parse_bool().parse("false_hello".to_string()),
            Some(ParseResult {
                value: JsonBool(false),
                s: "_hello".to_owned(),
            })
        );
        assert_eq!(parse_bool().parse("foo_hello".to_string()), None,);
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_string().parse("\"hello\"friend".to_string()),
            Some(ParseResult {
                value: JsonString("hello".to_string()),
                s: "friend".to_owned(),
            })
        );
        assert_eq!(
            // fails due to lack of end quote
            parse_bool().parse("\"hello".to_string()),
            None,
        );
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(
            parse_number().parse("1234hello".to_string()),
            Some(ParseResult {
                value: JsonNumber(1234),
                s: "hello".to_owned(),
            })
        );
    }

    #[test]
    fn test_parse_json() {
        assert_eq!(parse_json("null".to_string()), Some(JsonNull),);
        assert_eq!(parse_json("true".to_string()), Some(JsonBool(true)),);
        assert_eq!(parse_json("1234".to_string()), Some(JsonNumber(1234)),);
        assert_eq!(
            parse_json("\"foo\"".to_string()),
            Some(JsonString("foo".to_string())),
        );
    }
}
