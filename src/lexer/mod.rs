use std::{fmt, iter::Peekable, str::Chars};

use anyhow::{anyhow, Result};

// Order of variants in this enum encodes operator precedence
// where top one is the least significant
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operator {
    Equivalence,
    Implication,
    Or,
    And,
    Not,
    ParenthisClosed,
    ParenthisOpen,
    Assign,
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
}

impl Token {
    fn from_digit(ch: char) -> Token {
        Token::Value(Value::Bool(Token::eval_bool(ch)))
    }

    fn eval_bool(ch: char) -> bool {
        if ch == '0' {
            false
        } else {
            true
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Value(v) => write!(f, "{}", v),
            Token::Operator(v) => write!(f, "{:?}", v),
        }
    }
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl Lexer<'_> {
    pub fn new(contents: &str) -> Lexer {
        Lexer {
            chars: contents.chars().peekable(),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ch = self.chars.next();
            let token = match ch {
                Some('^') => Token::Operator(Operator::And),
                Some('v') => Token::Operator(Operator::Or),
                Some('~') => Token::Operator(Operator::Not),
                Some('(') => Token::Operator(Operator::ParenthisOpen),
                Some(')') => Token::Operator(Operator::ParenthisClosed),
                Some('<') => {
                    // "<=>" equivalence
                    let next = self.chars.next();
                    let next_after = self.chars.next();
                    if let (Some('='), Some('>')) = (next, next_after) {
                        Token::Operator(Operator::Equivalence)
                    } else {
                        return Some(Err(anyhow!(
                            "Unexpected '{}{}' after <. Did you mean '<=>'?",
                            next.unwrap_or(' '),
                            next_after.unwrap_or(' ')
                        )));
                    }
                }
                Some(':') => {
                    let next = self.chars.peek();
                    if let Some('=') = next {
                        self.chars.next();
                        Token::Operator(Operator::Assign)
                    } else {
                        continue;
                    }
                }
                Some('=') => {
                    // "=>" implication is differentiated from equivalence only because
                    // the iterator advanced twice on previous step
                    let next = self.chars.peek();
                    if let Some('>') = next {
                        self.chars.next();
                        Token::Operator(Operator::Implication)
                    } else {
                        continue;
                    }
                }
                Some(other) if other.is_digit(10) => Token::from_digit(other),
                Some(other) if other.is_whitespace() => continue,
                Some(other) if other.is_ascii_alphabetic() => {
                    Token::Value(Value::Variable(other.to_string()))
                }
                Some(other) => return Some(Err(anyhow!("Unexpected character '{}'", other))),
                None => return None,
            };
            return Some(Ok(token));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Operator, Token, Value};

    #[test]
    fn test_lexer_simple() {
        let lexer = Lexer::new("1 ^ 0 v ~1 => 0 <=> 1");
        let result: Vec<Token> = lexer.into_iter().map(|r| r.unwrap()).collect();

        let expected = vec![
            Token::Value(Value::Bool(true)),
            Token::Operator(Operator::And),
            Token::Value(Value::Bool(false)),
            Token::Operator(Operator::Or),
            Token::Operator(Operator::Not),
            Token::Value(Value::Bool(true)),
            Token::Operator(Operator::Implication),
            Token::Value(Value::Bool(false)),
            Token::Operator(Operator::Equivalence),
            Token::Value(Value::Bool(true)),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_lexer_parents() {
        let lexer = Lexer::new("(1 ^ 0) ^ 1");
        let result: Vec<Token> = lexer.into_iter().map(|r| r.unwrap()).collect();

        let expected = vec![
            Token::Operator(Operator::ParenthisOpen),
            Token::Value(Value::Bool(true)),
            Token::Operator(Operator::And),
            Token::Value(Value::Bool(false)),
            Token::Operator(Operator::ParenthisClosed),
            Token::Operator(Operator::And),
            Token::Value(Value::Bool(true)),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_lexer_variables() {
        let lexer = Lexer::new("p := 1 q := 0 p ^ q");
        let result: Vec<Token> = lexer.into_iter().map(|r| r.unwrap()).collect();

        let expected = vec![
            Token::Value(Value::Variable("p".to_string())),
            Token::Operator(Operator::Assign),
            Token::Value(Value::Bool(true)),
            Token::Value(Value::Variable("q".to_string())),
            Token::Operator(Operator::Assign),
            Token::Value(Value::Bool(false)),
            Token::Value(Value::Variable("p".to_string())),
            Token::Operator(Operator::And),
            Token::Value(Value::Variable("q".to_string())),
        ];
        assert_eq!(result, expected);
    }
}
