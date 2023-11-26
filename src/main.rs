use anyhow::{anyhow, Result};
use core::fmt;
use std::{env, fs::File, io::Read, println, write, unimplemented};

#[derive(Debug)]
enum Operator {
    And,
    Or,
    Not
}

#[derive(Debug)]
enum Value {
    Bool(bool),
    Variable(String),
}

#[derive(Debug)]
enum Token {
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
            Token::Value(v) => write!(f, "Value({:?})", v),
            Token::Operator(v) => write!(f, "Operator({:?})", v),
            Token::Empty => write!(f, "Token(Empty)"),
        }
    }
}

#[derive(Debug)]
struct ASTNode {
    token: Token,
    left: Option<Box<ASTNode>>,
    right: Option<Box<ASTNode>>,
}

impl fmt::Display for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let left: String = if let Some(v) = &self.left {
            v.token.to_string()
        } else {
            "None".to_string()
        };

        let right: String = if let Some(v) = &self.right {
            v.token.to_string()
        } else {
            "None".to_string()
        };

        write!(
            f,
            "Node({:?}, left: {:?}, right: {:?})",
            self.token, left, right
        )
    }
}

impl ASTNode {
    fn make_new_root(self, token: Token) -> ASTNode {
        let new_root = ASTNode {
            token,
            left: Some(Box::new(self)),
            right: None,
        };
        new_root
    }

    fn add_left_child(&mut self, token: Token) {
        self.left = Some(Box::new(ASTNode {
            token,
            left: None,
            right: None,
        }));
    }

    fn add_right_child(&mut self, token: Token) {
        self.right = Some(Box::new(ASTNode {
            token,
            left: None,
            right: None,
        }));
    }
}

fn parse(contents: &str) -> Result<ASTNode> {
    let mut root = ASTNode {
        token: Token::Empty,
        left: None,
        right: None,
    };

    for ch in contents.chars() {
        if ch.is_whitespace() {
            continue;
        };
        if ch.is_digit(10) {
            let token = Token::from_digit(ch);
            match root.token {
                Token::Empty => {
                    root.add_left_child(token)
                },
                Token::Operator(ref operator) => match operator {
                    Operator::And => {
                        root.add_right_child(token);
                    },
                    _ => {},
                },
                _ => return Err(anyhow!("Expected operator after a value")),
            };
        };

        // if ch == '(' {
        //     match root.token {

        //     }
        // }

        if ch == '^' {
            let token = Token::Operator(Operator::And);
            match root.token {
                Token::Empty => {
                    root.token = token;
                }
                Token::Value(_) => {
                    let new_root = ASTNode::make_new_root(root, token);
                    root = new_root;
                }
                Token::Operator(op) => match op {
                    Operator::Not => unimplemented!(),
                    val => return Err(anyhow!("Invalid syntax: unexpected {} after {:?}", token, val))
                }
                _ => {}
            };
        };
        if ch == 'v' {
            let token = Token::Operator(Operator::Or);
            match root.token {
                Token::Empty => {
                    root.token = token;
                }
                Token::Value(_) => {
                    let new_root = ASTNode::make_new_root(root, token);
                    root = new_root;
                }
                Token::Operator(op) => match op {
                    Operator::Not => unimplemented!(),
                    val => return Err(anyhow!("Invalid syntax: unexpected {} after {:?}", token, val))
                }
                _ => {}
            };
        };
    };
    Ok(root)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let mut file = File::open(file_path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let ast_root = parse(&buffer)?;

    println!("Root: {}", ast_root);
    Ok(())
}
