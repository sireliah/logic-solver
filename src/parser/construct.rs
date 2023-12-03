use std::collections::{BinaryHeap, VecDeque};
use std::unimplemented;

use anyhow::{anyhow, Result};

use crate::lexer::{Lexer, Operator, Token};
use crate::parser::ASTNode;

// TODO:
// - build the ast
// - parentheses
// - negation

// pub fn construct(mut node: ASTNode, lexer: &mut Lexer) -> Result<ASTNode> {
//     let mut output: VecDeque<Token> = VecDeque::new();
//     let mut operators: BinaryHeap<Operator> = BinaryHeap::new();

//     while let Some(token) = lexer.next() {
//         match token {
//             Token::Value(v) => output.push_back(Token::Value(v)),
//             Token::Operator(operator) => match operator {
//                 Operator::ParenthisOpen => {},
//                 Operator::ParenthisClosed => {
//                     // BUG: this pops items by priority
//                     // Pop in the normal order instead
//                     while let Some(inner_op) = operators.pop() {
//                         println!("Inner op {:?}", inner_op);
//                         match inner_op {
//                             Operator::ParenthisOpen => {}
//                             Operator::ParenthisClosed => break,
//                             op => output.push_back(Token::Operator(op)),
//                         }
//                     }
//                 }
//                 other_op => {
//                     // Check last operator in the stack
//                     match operators.peek() {
//                         Some(prev_op) => {
//                             if prev_op > &other_op {
//                                 // TOOD: reduce this
//                                 if let Some(last_op) = operators.pop() {
//                                     output.push_back(Token::Operator(last_op));
//                                     operators.push(other_op);
//                                 }
//                             } else if prev_op == &other_op {
//                                 output.push_back(Token::Operator(other_op));
//                             } else {
//                                 operators.push(other_op);
//                             };
//                         }
//                         None => operators.push(other_op),
//                     };
//                 }
//             },
//             Token::Empty => unimplemented!(),
//         }
//     }
//     for op in operators.into_iter() {
//         output.push_back(Token::Operator(op));
//     }

//     construct_ast_from_queue(node, output)
// }

// pub fn construct_ast_from_queue(mut node: ASTNode, output: VecDeque<Token>) -> Result<ASTNode> {
//     for out in output.into_iter() {
//         println!("{}", out);

//         match out {
//             Token::Value(value) => {
//                 if let Some(_) = node.left {
//                     node.add_right_child(Token::Value(value));
//                 } else {
//                     node.add_left_child(Token::Value(value));
//                 }
//             },
//             Token::Operator(operator) => {
//                 match node.token {
//                     Token::Empty => node.token = Token::Operator(operator),
//                 }
//             },
//             other => panic!("Not expected to get here! {:?}", other),
//         }

//     };
//     Ok(node)
// }

pub fn construct_ast(mut root: ASTNode, lexer: &mut Lexer) -> Result<ASTNode> {
    loop {
        if let Some(token) = lexer.next() {
            match token {
                Token::Value(_) => handle_value(&mut root, token)?,
                Token::Operator(op) if op == Operator::ParenthisOpen => {
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
                Token::Operator(op) if op == Operator::ParenthisClosed => return Ok(root),
                Token::Operator(_) => {
                    root = handle_operator(root, token)?;
                }
                Token::Empty => unimplemented!(),
            }
        } else {
            break;
        }
    }
    Ok(root)
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
            Operator::Not => {
                // FIXME: Check me
                root.add_left_child(token);
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
            Operator::Not => {
                println!("Root: {}, Parsing: {}", root.token, token);
                panic!("aa");
            }
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
            _ => unimplemented!(),
        },
        // FIXME: Cover missing
        _ => unimplemented!(),
    };
    Ok(root)
}

fn descend_right(node: &mut Box<ASTNode>, token: Token) {
    match node.right {
        Some(ref mut right) => descend_right(right, token),
        None => node.add_right_child(token),
    };
}
