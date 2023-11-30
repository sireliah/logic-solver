use anyhow::{anyhow, Result};
use std::{collections::VecDeque, fmt, fs::File, io::Write};

use crate::lexer::{Lexer, Operator, Token};

#[derive(Debug)]
pub struct ASTNode {
    pub token: Token,
    pub left: Option<Box<ASTNode>>,
    pub right: Option<Box<ASTNode>>,
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
    pub fn make_new_root_left(self, token: Token) -> ASTNode {
        let new_root = ASTNode {
            token,
            left: Some(Box::new(self)),
            right: None,
        };
        new_root
    }

    pub fn add_left_child(&mut self, token: Token) {
        self.left = Some(Box::new(ASTNode {
            token,
            left: None,
            right: None,
        }));
    }

    pub fn add_right_child(&mut self, token: Token) {
        self.right = Some(Box::new(ASTNode {
            token,
            left: None,
            right: None,
        }));
    }

    /// Outputs graph in graphviz format
    /// Check https://graphviz.org/pdf/dotguide.pdf
    pub fn visualize_graph(self) -> Result<()> {
        fn write_definition(counter: u32, token: &Token) -> String {
            match token {
                Token::Value(_) => format!("    {} [label=\"{}\"]\n", counter, token),
                Token::Operator(_) => {
                    format!("    {} [label=\"{}\" shape=\"box\"]\n", counter, token)
                }
                Token::Empty => format!("    {} [label=\"{}\"]\n", counter, token),
                _ => "\n".to_string(),
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

pub fn construct_ast(mut root: ASTNode, lexer: &mut Lexer) -> Result<ASTNode> {
    loop {
        if let Some(token) = lexer.next() {
            match token {
                Token::Value(_) => handle_value(&mut root, token)?,
                Token::Operator(_) => root = handle_operator(root, token)?,
                Token::ParenthisOpen => {
                    let new_root = ASTNode {
                        token: Token::Empty,
                        left: None,
                        right: None,
                    };
                    let sub_node = construct_ast(new_root, lexer)?;

                    if let Token::Empty = root.token {
                        root.left = Some(Box::new(sub_node));
                    } else {
                        root.right = Some(Box::new(sub_node));
                    }
                }
                Token::ParenthisClosed => return Ok(root),
                Token::Empty => unimplemented!(),
            }
        } else {
            break;
        }
    }
    Ok(root)
}

fn descend_right(node: &mut Box<ASTNode>, token: Token) {
    match node.right {
        Some(ref mut right) => descend_right(right, token),
        None => node.add_right_child(token),
    };
}

fn handle_value(root: &mut ASTNode, token: Token) -> Result<()> {
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
    Ok(())
}

fn handle_operator(mut root: ASTNode, token: Token) -> Result<ASTNode> {
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
                let new_root = root.make_new_root_left(token);
                return Ok(new_root);
            }
        },
        // FIXME: Cover missing
        _ => unimplemented!(),
    };
    Ok(root)
}
