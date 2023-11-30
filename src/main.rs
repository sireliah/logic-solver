use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Read;
use std::{
    env,
    unimplemented,
};

use logic_solver::ast::ASTNode;
use logic_solver::lexer::{Lexer, Operator, Token};

fn descend_right(node: &mut Box<ASTNode>, token: Token) {
    match node.right {
        Some(ref mut right) => descend_right(right, token),
        None => node.add_right_child(token),
    };
}

fn construct_ast(mut root: ASTNode, lexer: &mut Lexer) -> Result<ASTNode> {
    loop {
        if let Some(token) = lexer.next() {
            match token {
                Token::Value(_) => {
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
                }
                Token::Operator(_) => {
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
                        // FIXME: Cover missing
                        _ => {}
                    };
                }
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
                Token::Empty => todo!(),
            }
        } else {
            break;
        }
    }
    Ok(root)
}

fn parse(contents: &str) -> Result<ASTNode> {
    let mut root = ASTNode {
        token: Token::Empty,
        left: None,
        right: None,
    };
    let mut lexer = Lexer::new(contents);
    root = construct_ast(root, &mut lexer)?;
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
