use anyhow::{anyhow, Result};
use core::fmt;
use std::{
    collections::VecDeque,
    env,
    fs::File,
    io::{Read, Write},
    println, unimplemented, write,
};

#[derive(Debug)]
enum Operator {
    And,
    Or,
    Not,
}

#[derive(Debug)]
enum Value {
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
            Token::Value(v) => write!(f, "{}", v),
            Token::Operator(v) => write!(f, "{:?}", v),
            Token::Empty => write!(f, "Empty"),
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
    fn make_new_root_left(self, token: Token) -> ASTNode {
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

    #[allow(dead_code)]
    fn print_graph(self) {
        let mut queue = VecDeque::new();
        queue.push_back(Box::new(self));
        loop {
            match queue.pop_front() {
                Some(node) => {
                    println!("{}", node);
                    if let Some(left) = node.left {
                        match left.token {
                            Token::Operator(_) => queue.push_back(left),
                            _ => {}
                        };
                    };
                    if let Some(right) = node.right {
                        match right.token {
                            Token::Operator(_) => queue.push_back(right),
                            _ => {}
                        };
                    };
                }
                None => break,
            }
        }
    }

    /// Outputs graph in graphviz format
    /// Check https://graphviz.org/pdf/dotguide.pdf
    fn visualize_graph(self) -> Result<()> {

        fn write_definition(counter: u32, token: &Token) -> String {
            match token {
                Token::Value(_) => format!("    {} [label=\"{}\"]\n", counter, token),
                Token::Operator(_) => {
                    format!("    {} [label=\"{}\" shape=\"box\"]\n", counter, token)
                }
                Token::Empty => format!("    {} [label=\"{}\"]\n", counter, token),
            }
        }

        let mut queue = VecDeque::new();
        let mut graph_relations = vec![];
        let mut graph = vec!["digraph G {\n".to_string()];
        let mut counter: u32 = 0;
        graph.push(write_definition(counter, &self.token));

        queue.push_back((counter, Box::new(self)));

        loop {
            match queue.pop_front() {
                Some((num, node)) => {
                    if counter > 0 {
                        println!("{}, {}", counter, node);
                        graph.push(write_definition(counter, &node.token));
                        graph_relations.push(format!("    {} -> {}\n", num, counter));
                    }
                    if let Some(left) = node.left {
                        match left.token {
                            Token::Operator(_) => queue.push_back((counter, left)),
                            Token::Value(_) => queue.push_back((counter, left)),
                            _ => {}
                        };
                    };
                    if let Some(right) = node.right {
                        match right.token {
                            Token::Operator(_) => queue.push_back((counter, right)),
                            Token::Value(_) => queue.push_back((counter, right)),
                            _ => {}
                        };
                    };
                }
                None => break,
            }
            counter += 1;
        }
        let mut file = File::create("graph.dot")?;
        for definition in graph {
            file.write(definition.as_bytes())?;
        }
        for relation in graph_relations {
            file.write(relation.as_bytes())?;
        }
        file.write("}".as_bytes())?;
        Ok(())
    }
}

fn descend_right(node: &mut Box<ASTNode>, token: Token) {
    match node.right {
        Some(ref mut right) => descend_right(right, token),
        None => node.add_right_child(token),
    };
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
                Token::Empty => root.add_left_child(token),
                Token::Operator(ref operator) => match operator {
                    Operator::And => {
                        match root.right {
                            Some(ref mut right) => {
                                descend_right(right, token);
                            }
                            None => {
                                root.add_right_child(token);
                            }
                        };
                    }
                    Operator::Or => {
                        match root.right {
                            Some(ref mut right) => {
                                descend_right(right, token);
                            }
                            None => {
                                root.add_right_child(token);
                            }
                        };
                    }
                    _ => unimplemented!(),
                },
                _ => return Err(anyhow!("Expected operator after a value")),
            };
        };

        if ch == '(' {};

        if ch == ')' {};

        if ch == '^' {
            let token = Token::Operator(Operator::And);
            match root.token {
                Token::Empty => {
                    root.token = token;
                }
                Token::Value(_) => return Err(anyhow!("Value can never be a root")),
                Token::Operator(ref op) => match op {
                    Operator::Not => unimplemented!(),
                    Operator::Or => {
                        // take right child of root and place node there
                        // make this child left of this node
                        let right = root.right.take();
                        let node = ASTNode {
                            token,
                            left: right,
                            right: None,
                        };
                        root.right = Some(Box::new(node));
                    }
                    Operator::And => {
                        root = root.make_new_root_left(token);
                    }
                },
            };
        };

        if ch == 'v' {
            let token = Token::Operator(Operator::Or);
            match root.token {
                Token::Empty => {
                    root.token = token;
                }
                Token::Value(_) => {
                    root = root.make_new_root_left(token);
                }
                Token::Operator(ref op) => match op {
                    Operator::Not => unimplemented!(),
                    Operator::Or => {
                        root = root.make_new_root_left(token);
                    }
                    Operator::And => {
                        root = root.make_new_root_left(token);
                    }
                },
            };
        };
    }
    Ok(root)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let mut file = File::open(file_path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let ast_root = parse(&buffer)?;

    ast_root.visualize_graph()?;
    Ok(())
}
