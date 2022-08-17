use std::{
    borrow::BorrowMut,
    fmt::Debug,
    iter::Peekable,
    num::ParseIntError,
    str::{Chars, FromStr},
};

use regex::Regex;

#[derive(Debug)]
pub enum TagParseErr {
    NoCaptures,
    CaptureNotFound,
    CaptureParseErr(ParseIntError),
    UnknownTag(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Text(String),
    Tag(Tag),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Tag {
    User(usize),
    Article(usize),
}

impl FromStr for Tag {
    type Err = TagParseErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static::lazy_static! {
            static ref USER_RE: Regex = Regex::new(r"user:\s*(?P<id>\d+)").unwrap();
            static ref ARTICLE_RE: Regex = Regex::new(r"article:\s*(?<id>\d+)").unwrap();
        }

        if let Some(cap) = USER_RE.captures(s) {
            return Ok(Tag::User(
                cap.name("id")
                    .ok_or(TagParseErr::CaptureNotFound)?
                    .as_str()
                    .parse()
                    .map_err(|e| TagParseErr::CaptureParseErr(e))?,
            ));
        }

        if let Some(cap) = ARTICLE_RE.captures(s) {
            return Ok(Tag::Article(
                cap.name("id")
                    .ok_or(TagParseErr::CaptureNotFound)?
                    .as_str()
                    .parse()
                    .map_err(|e| TagParseErr::CaptureParseErr(e))?,
            ));
        }

        Err(TagParseErr::UnknownTag(s.to_string()))
    }
}

pub struct TokenIter<'a> {
    iter: Peekable<Chars<'a>>,
}

impl<'a> TokenIter<'a> {
    pub fn new<S: Into<&'a str>>(s: S) -> Self {
        TokenIter {
            iter: s.into().chars().peekable(),
        }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.iter.next() {
            return Some(match next {
                '[' => Token::Tag(
                    self.iter
                        .borrow_mut()
                        .take_while(|p| *p != ']')
                        .collect::<String>()
                        .parse::<Tag>()
                        .unwrap(),
                ),

                c => {
                    let mut text = String::from(c);
                    while let Some(peek) = self.iter.peek() {
                        if *peek == '[' {
                            break;
                        }
                        text.push(self.iter.next().unwrap())
                    }
                    Token::Text(text)
                }
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    pub fn parse_user() {
        assert_eq!(
            TokenIter::new("[user:0]").collect::<Vec<_>>(),
            vec![Token::Tag(Tag::User(0))]
        );
    }

    #[test]
    pub fn parse_article() {
        assert_eq!(
            TokenIter::new("[article:0]").collect::<Vec<_>>(),
            vec![Token::Tag(Tag::Article(0))]
        );
    }
}
