use anyhow::Result;
use std::{collections::VecDeque, fmt, fs::File, io::Write, path::Path};

use crate::lexer::Token;
mod construct;
pub use construct::construct_ast;

#[derive(Debug, PartialEq)]
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
    pub fn new(token: Token) -> ASTNode {
        ASTNode {
            token,
            left: None,
            right: None,
        }
    }

    pub fn make_new_root_left(self, token: Token) -> ASTNode {
        let new_root = ASTNode {
            token,
            left: Some(Box::new(self)),
            right: None,
        };
        new_root
    }

    pub fn make_new_root_right(self, token: Token) -> ASTNode {
        let new_root = ASTNode {
            token,
            left: None,
            right: Some(Box::new(self)),
        };
        new_root
    }

    pub fn add_left_child(&mut self, node: ASTNode) {
        self.left = Some(Box::new(node))
    }

    pub fn add_right_child(&mut self, node: ASTNode) {
        self.right = Some(Box::new(node))
    }

    pub fn add_left_token(&mut self, token: Token) {
        self.left = Some(Box::new(ASTNode {
            token,
            left: None,
            right: None,
        }));
    }

    pub fn add_right_token(&mut self, token: Token) {
        self.right = Some(Box::new(ASTNode {
            token,
            left: None,
            right: None,
        }));
    }

    /// Outputs graph in graphviz format
    /// Check https://graphviz.org/pdf/dotguide.pdf
    pub fn visualize_graph(&self, out_path: &Path) -> Result<()> {
        fn write_definition(counter: u32, token: &Token) -> String {
            match token {
                Token::Value(_) => format!("    {} [label=\"{}\"]\n", counter, token),
                Token::Operator(_) => {
                    format!("    {} [label=\"{}\" shape=\"box\"]\n", counter, token)
                }
            }
        }

        let mut queue = VecDeque::new();
        let mut graph_relations = vec![];
        let mut graph = vec!["graph G {\n".to_string()];
        let mut counter: u32 = 0;
        graph.push(write_definition(counter, &self.token));

        queue.push_back((counter, Box::new(self)));

        loop {
            match queue.pop_front() {
                Some((num, node)) => {
                    if counter > 0 {
                        graph.push(write_definition(counter, &node.token));
                        graph_relations.push(format!("    {} -- {}\n", num, counter));
                    }
                    if let Some(left) = &node.left {
                        match left.token {
                            Token::Operator(_) => queue.push_back((counter, Box::new(&left))),
                            Token::Value(_) => queue.push_back((counter, Box::new(&left))),
                        };
                    };
                    if let Some(right) = &node.right {
                        match right.token {
                            Token::Operator(_) => queue.push_back((counter, Box::new(&right))),
                            Token::Value(_) => queue.push_back((counter, Box::new(&right))),
                        };
                    };
                }
                None => break,
            }
            counter += 1;
        }
        let mut file = File::create(out_path)?;
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
