// #![allow(unused_imports)]
// #![allow(dead_code)]
// use std::{collections::HashMap, ops};

// #[derive(Debug, Clone)]
// pub enum JsonValue {
//     JsonNull,
//     JsonBool(bool),
//     JsonNumber(i64), // TODO(make this parse floats etc later)
//     JsonString(String),
//     JsonArray(Box<Vec<JsonValue>>),
//     JsonObject(Box<HashMap<String, JsonValue>>),
// }

// #[derive(Debug, PartialEq)]
// pub struct ParseResult<T> {
//     value: T,
//     s: String, // TODO: should this be &str instead?
// }

// pub trait Parser<T> {
//     fn parse(&self, s: &str) -> Option<ParseResult<T>>;
// }

// struct StringP(String);

// impl Parser<String> for StringP {
//     fn parse(&self, s: &str) -> Option<ParseResult<String>> {
//         if !s.is_empty() && s.starts_with(&self.0) {
//             Some(ParseResult {
//                 value: self.0.to_owned(),
//                 s: s[1..].to_owned(),
//             })
//         } else {
//             None
//         }
//     }
// }

// struct ConstantP<T>(T);

// impl<T: Clone> Parser<T> for ConstantP<T> {
//     fn parse(&self, s: &str) -> Option<ParseResult<T>> {
//         Some(ParseResult {
//             value: self.0.to_owned(),
//             s: s.to_owned(),
//         })
//     }
// }

// struct FlatMapP<A, B> {
//     p: Box<dyn Parser<A>>,
//     f: fn(A) -> dyn Parser<B>,
// }

// impl<A, B> Parser<B> for FlatMapP<A, B> {
//     fn parse(&self, s: &str) -> Option<ParseResult<B>> {    
//         self.p.parse(s).and_then(|pr| {
//             let ParseResult {value: a, s: s1} = pr;
//             let p2 = (self.f)(a);
//             p2.parse(s)
//         })
//     }
// }

// // impl<A, B, C> Parser<C> for ChainP<A, B, C> {
// //     fn parse(&self, s: &str) -> Option<ParseResult<C>> {
// //         self.p1.parse(s).and_then(|psa| {
// //             self.p2.parse(&psa.s).and_then(|psb| {
// //                 let c = (self.f)(psa.value, psb.value);
// //                 Some(ParseResult { value: c, s: psb.s })
// //             })
// //         })
// //     }
// // }

// impl<T> Parser<T> for &dyn Parser<T> {
//     fn parse(&self, s: &str) -> Option<ParseResult<T>> {
//         self.parse(s)
//     }
// }

// // fn map2<A, B, C>(
// //     p1: &dyn Parser<'static, A>,
// //     p2: &dyn Parser<B>,
// //     f: &dyn Fn(A, B) -> C,
// // ) -> impl Parser<C> {
// //     ChainP {
// //         p1: Box::new(p1),
// //         p2: Box::new(p2),
// //         f: Box::new(f),
// //     }
// // }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn stringp_success() {
//         let p1 = StringP("hel".to_owned());
//         assert_eq!(
//             p1.parse("hello"),
//             Some(ParseResult {
//                 value: "hel".to_owned(),
//                 s: "ello".to_owned(),
//             })
//         );
//     }

//     #[test]
//     fn stringp_fail() {
//         let p = StringP("fre".to_owned());
//         assert!(p.parse("friend").is_none());
//     }

//     fn constantp_int() {
//         let p = ConstantP(1);
//         assert_eq!(
//             p.parse("hello"),
//             Some(ParseResult {
//                 value: 1,
//                 s: "hello".to_owned(),
//             })
//         );
//     }

//     fn constantp_string() {
//         let p = ConstantP("hello".to_owned());
//         assert_eq!(
//             p.parse("hello"),
//             Some(ParseResult {
//                 value: "hello".to_owned(),
//                 s: "hello".to_owned(),
//             })
//         );
//     }

//     fn chainp_success() {
//         let p1 = StringP("he".to_owned());
//         let p2 = StringP("llo".to_owned());
//         let p = ChainP {
//             p1: p1,
//             p2: p2,
//             f: |a, b| format!("{}{}", a, b),
//         };
//         assert_eq!(
//             p.parse("hello"),
//             Some(ParseResult {
//                 value: "hello".to_owned(),
//                 s: "".to_owned(),
//             })
//         );
//     }
// }

// // struct Parser<T>(impl Fn(String) -> Option<ParseResult<T>>);

// // allow for syntax let parser = parser1 | parser2;
// // impl ops::BitOr for JsonParser {
// //     type Output = Self;

// //     fn bitor(self, rhs: Self) -> Self::Output {
// //         let JsonParser(f1) = self;
// //         let JsonParser(f2) = rhs;

// //         // let f = |s| {
// //         //     f1(s).or_else(move || f2(s.to_owned()))
// //         // };
// //         // JsonParser(f)
// //         self
// //         // unimplemented!()
// //     }
// // }

#[derive(Debug, PartialEq)]
pub struct ParseResult<T> {
    value: T,
    s: String, // TODO: should this be &str instead?
}

pub trait Parser<T> {
    fn parse(&self, s: &str) -> Option<ParseResult<T>>;
}

fn fmap<A, B>(p: dyn Parser<A>, f: Fn(A) -> B) -> dyn Parser<B> {
    let x = |s| {
        p.parse(s).map(
            |pr| {
                ParseResult {
                    value: f(pr.value),
                    s: pr.s
                }
            }
        )
    }
}

fn flatmap<A, B>(p: Parser<A>, f: Fn(A) -> Parser<B>) -> Parser<B> {
    |s| {
        p.parse(s).and_then(|pr| {
            let pb = f(pr.value);
            pb.parse(&pr.s)
        })
    }
}

fn or<A>(p1: Parser<A>, p2: Parse<A>) -> Parser<A> {
    |s| {
        p1.parse(s).or_else(|| {
            p2.parse(s)
        })
    }
}

fn many(p: Parser<A>) -> Parser<Vec<A>> {
    |s| {
        match p.parse(s) {
            Some(pr) => {
                let many_p = many(p);
                flatmap(many_p, |&vec| {
                    let final_result = Vec::new();
                    final_result.push(pr.value);
                    final_result.extend(vec);
                    final_result
                })
            }
            None => Some(ParseResult { value: Vec::new(), s: s.to_owned()})
        }
    }
}