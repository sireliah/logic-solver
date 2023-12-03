use std::{fmt, str::Chars};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operator {
    Or,
    And,
    Not,
    ParenthisClosed,
    ParenthisOpen,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Variable(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::Variable(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Value(Value),
    Operator(Operator),
    Empty,
}

impl Token {
    fn from_digit(ch: char) -> Token {
        let b = if ch == '0' { false } else { true };
        Token::Value(Value::Bool(b))
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Value(v) => write!(f, "{}", v),
            Token::Operator(v) => write!(f, "{:?}", v),
            Token::Empty => write!(f, "Empty"),
        }
    }
}

pub struct Lexer<'a> {
    chars: Chars<'a>,
}

impl Lexer<'_> {
    pub fn new(contents: &str) -> Lexer {
        Lexer {
            chars: contents.chars(),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ch = self.chars.next();
            let token = match ch {
                Some('^') => Token::Operator(Operator::And),
                Some('v') => Token::Operator(Operator::Or),
                Some('~') => Token::Operator(Operator::Not),
                Some('(') => Token::Operator(Operator::ParenthisOpen),
                Some(')') => Token::Operator(Operator::ParenthisClosed),
                Some(other) if other.is_digit(10) => Token::from_digit(other),
                Some(other) if other.is_whitespace() => continue,
                Some(_) => continue,
                None => return None,
            };
            return Some(token);
        }
    }
}
