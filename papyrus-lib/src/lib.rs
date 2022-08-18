use std::{
    borrow::BorrowMut,
    fmt::{Debug, Display},
    iter::Peekable,
    num::ParseIntError,
    str::{Chars, FromStr},
};

use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
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
            static ref ARTICLE_RE: Regex = Regex::new(r"article:\s*(?P<id>\d+)").unwrap();
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

        Err(TagParseErr::UnknownTag(format!("[{s}]")))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    line: u16,
}

impl Position {
    pub fn new(line: u16) -> Self {
        Self { line }
    }
}

impl From<u16> for Position {
    fn from(line: u16) -> Self {
        Self::new(line)
    }
}

pub struct TokenIter<'a> {
    iter: Peekable<Chars<'a>>,
    position: Position,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}", self.line) // 1 has to be subtracted here, since the Position points at what's next.
    }
}

impl<'a> TokenIter<'a> {
    pub fn new<S: Into<&'a str>>(s: S) -> Self {
        TokenIter {
            iter: s.into().chars().peekable(),
            position: Position::new(1),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenizeErr {
    TagErr(Position, TagParseErr),
}

impl Display for TokenizeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TagErr(p, e) => write!(f, "{e:?} at {p}"),
        }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Result<Token, TokenizeErr>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.iter.next() {
            if next == '\n' {
                self.position.line += 1;
            }

            return Some(match next {
                '[' => self
                    .iter
                    .borrow_mut()
                    .take_while(|p| *p != ']')
                    .collect::<String>()
                    .parse::<Tag>()
                    .map_err(|e| TokenizeErr::TagErr(self.position, e))
                    .map(|tag| Token::Tag(tag)),

                c => {
                    let mut text = String::from(c);
                    while let Some(peek) = self.iter.peek() {
                        if *peek == '[' {
                            break;
                        }
                        // We can use unrwap here since we checked if there is a next character via iter.peek
                        text.push(self.iter.next().unwrap())
                    }
                    Ok(Token::Text(text))
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
            vec![Ok(Token::Tag(Tag::User(0)))]
        );
    }

    #[test]
    pub fn parse_article() {
        assert_eq!(
            TokenIter::new("[article:0]").collect::<Vec<_>>(),
            vec![Ok(Token::Tag(Tag::Article(0)))]
        );
    }

    #[test]
    pub fn line_err() {
        let tag = "\n[unknown]";
        assert_eq!(
            TokenIter::new(tag).collect::<Vec<_>>(),
            vec![Err(TokenizeErr::TagErr(
                2.into(),
                TagParseErr::UnknownTag(tag[1..].to_string())
            ))]
        )
    }

    #[test]
    pub fn parse_err() {
        let tag = "[unknown:0]";
        assert_eq!(
            TokenIter::new(tag).collect::<Vec<_>>(),
            vec![Err(TokenizeErr::TagErr(
                1.into(),
                TagParseErr::UnknownTag(tag.to_string())
            ))]
        )
    }
}
