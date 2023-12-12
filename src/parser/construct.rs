use anyhow::{anyhow, Result};
use log::debug;

use crate::lexer::{Lexer, Operator, Token};
use crate::parser::ASTNode;

fn make_node(tree_queue: &mut Vec<ASTNode>, operator: Operator) {
    if let Some(right) = tree_queue.pop() {
        // Special case for unary operators
        let node = if let Operator::Not = operator {
            ASTNode {
                token: Token::Operator(operator),
                left: Some(Box::new(right)),
                right: None,
            }
        } else {
            match tree_queue.pop() {
                Some(left) => ASTNode {
                    token: Token::Operator(operator),
                    left: Some(Box::new(left)),
                    right: Some(Box::new(right)),
                },
                None => ASTNode {
                    token: Token::Operator(operator),
                    left: Some(Box::new(right)),
                    right: None,
                },
            }
        };
        tree_queue.push(node);
    }
}

/// Shunting yard algorithm
pub fn construct_ast(lexer: &mut Lexer) -> Result<ASTNode> {
    let mut operators: Vec<Operator> = Vec::new();
    let mut tree_queue: Vec<ASTNode> = Vec::new();

    while let Some(token) = lexer.next() {
        debug!("{}", token);
        debug!("{:#?}", operators);
        match token {
            Token::Value(v) => {
                let node = ASTNode {
                    token: Token::Value(v),
                    left: None,
                    right: None,
                };
                tree_queue.push(node);
            }
            Token::Operator(operator) => match operator {
                Operator::ParenthisOpen => operators.push(Operator::ParenthisOpen),
                Operator::ParenthisClosed => {
                    while let Some(inner_op) = operators.pop() {
                        match inner_op {
                            Operator::ParenthisOpen => break,
                            Operator::ParenthisClosed => break,
                            op => make_node(&mut tree_queue, op),
                        }
                    }
                }
                current_op => {
                    let mut v = vec![];
                    while let Some(op) = operators.pop() {
                        match op {
                            // Left parenthesis is treated separately, because it has
                            // precedence property (highest) in this implementation.
                            Operator::ParenthisOpen => {
                                v.push(op);
                                break;
                            }
                            _ => {
                                if op >= current_op {
                                    make_node(&mut tree_queue, op);
                                } else {
                                    v.push(op);
                                }
                            }
                        }
                    }
                    operators.extend(v);
                    operators.push(current_op);
                }
            },
        }
    }
    for op in operators.into_iter().rev() {
        make_node(&mut tree_queue, op);
    }

    tree_queue.pop().ok_or(anyhow!(
        "Invalid syntax, expected at least one AST node left"
    ))
}

#[cfg(test)]
mod tests {
    use super::construct_ast;
    use crate::{
        lexer::{Lexer, Operator, Token, Value},
        parser::ASTNode,
    };

    #[test]
    fn test_construct_rpn_and() {
        let mut lexer = Lexer::new("1 ^ 0");
        let results = construct_ast(&mut lexer).unwrap();

        let left = ASTNode::new(Token::Value(Value::Bool(true)));
        let right = ASTNode::new(Token::Value(Value::Bool(false)));
        let expected = ASTNode {
            token: Token::Operator(Operator::And),
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        };
        assert_eq!(results, expected);
    }

    #[test]
    fn test_construct_ast_and_before_or() {
        let mut lexer = Lexer::new("1 ^ 0 v 1");
        let results = construct_ast(&mut lexer).unwrap();

        let left = ASTNode::new(Token::Value(Value::Bool(true)));
        let middle = ASTNode::new(Token::Value(Value::Bool(false)));
        let right = ASTNode::new(Token::Value(Value::Bool(true)));

        let mut and = ASTNode::new(Token::Operator(Operator::And));
        and.left = Some(Box::new(left));
        and.right = Some(Box::new(middle));

        let expected = ASTNode {
            token: Token::Operator(Operator::Or),
            left: Some(Box::new(and)),
            right: Some(Box::new(right)),
        };
        assert_eq!(results, expected);
    }

    #[test]
    fn test_construct_ast_or_before_and() {
        let mut lexer = Lexer::new("1 v 0 ^ 1");
        let results = construct_ast(&mut lexer).unwrap();

        let left = ASTNode::new(Token::Value(Value::Bool(true)));
        let middle = ASTNode::new(Token::Value(Value::Bool(false)));
        let right = ASTNode::new(Token::Value(Value::Bool(true)));

        let mut and = ASTNode::new(Token::Operator(Operator::And));
        and.left = Some(Box::new(middle));
        and.right = Some(Box::new(right));

        let expected = ASTNode {
            token: Token::Operator(Operator::Or),
            left: Some(Box::new(left)),
            right: Some(Box::new(and)),
        };
        assert_eq!(results, expected);
    }

    #[test]
    fn test_construct_ast_parent_with_lower_prec() {
        let mut lexer = Lexer::new("1 ^ (0 v 1)");
        let results = construct_ast(&mut lexer).unwrap();

        let left = ASTNode::new(Token::Value(Value::Bool(true)));

        let mut or = ASTNode::new(Token::Operator(Operator::Or));
        or.add_left_token(Token::Value(Value::Bool(false)));
        or.add_right_token(Token::Value(Value::Bool(true)));

        let expected = ASTNode {
            token: Token::Operator(Operator::And),
            left: Some(Box::new(left)),
            right: Some(Box::new(or)),
        };
        assert_eq!(results, expected);
    }

    #[test]
    fn test_construct_ast_parent_with_higher_prec() {
        let mut lexer = Lexer::new("(1 ^ 0) v 1");
        let results = construct_ast(&mut lexer).unwrap();

        let right = ASTNode::new(Token::Value(Value::Bool(true)));

        let mut and = ASTNode::new(Token::Operator(Operator::And));
        and.add_left_token(Token::Value(Value::Bool(true)));
        and.add_right_token(Token::Value(Value::Bool(false)));

        let expected = ASTNode {
            token: Token::Operator(Operator::Or),
            left: Some(Box::new(and)),
            right: Some(Box::new(right)),
        };
        assert_eq!(results, expected);
    }

    #[test]
    fn test_construct_rpn_double_parent_should_not_change_anything() {
        let mut lexer = Lexer::new("((1 ^ 0) v 1)");

        let results = construct_ast(&mut lexer).unwrap();

        let right = ASTNode::new(Token::Value(Value::Bool(true)));
        let mut and = ASTNode::new(Token::Operator(Operator::And));
        and.add_left_token(Token::Value(Value::Bool(true)));
        and.add_right_token(Token::Value(Value::Bool(false)));

        let expected = ASTNode {
            token: Token::Operator(Operator::Or),
            left: Some(Box::new(and)),
            right: Some(Box::new(right)),
        };
        assert_eq!(results, expected);
    }

    #[test]
    fn test_construct_ast_negation() {
        let mut lexer = Lexer::new("~1 v 0");
        let results = construct_ast(&mut lexer).unwrap();

        let right = ASTNode::new(Token::Value(Value::Bool(false)));

        let mut not = ASTNode::new(Token::Operator(Operator::Not));
        not.add_left_token(Token::Value(Value::Bool(true)));

        let expected = ASTNode {
            token: Token::Operator(Operator::Or),
            left: Some(Box::new(not)),
            right: Some(Box::new(right)),
        };

        assert_eq!(results, expected);
    }

    #[test]
    fn test_construct_ast_negation_double() {
        let mut lexer = Lexer::new("~1 v ~0");
        let results = construct_ast(&mut lexer).unwrap();

        let mut not = ASTNode::new(Token::Operator(Operator::Not));
        not.add_left_token(Token::Value(Value::Bool(true)));
        let mut not2 = ASTNode::new(Token::Operator(Operator::Not));
        not2.add_left_token(Token::Value(Value::Bool(false)));

        let expected = ASTNode {
            token: Token::Operator(Operator::Or),
            left: Some(Box::new(not)),
            right: Some(Box::new(not2)),
        };

        assert_eq!(results, expected);
    }

    #[test]
    fn test_construct_ast_longer_statement() {
        let mut lexer = Lexer::new("0 ^ 1 v 0 ^ 1");
        let results = construct_ast(&mut lexer).unwrap();

        let mut left_and = ASTNode::new(Token::Operator(Operator::And));
        left_and.add_left_token(Token::Value(Value::Bool(false)));
        left_and.add_right_token(Token::Value(Value::Bool(true)));

        let mut right_and = ASTNode::new(Token::Operator(Operator::And));
        right_and.add_left_token(Token::Value(Value::Bool(false)));
        right_and.add_right_token(Token::Value(Value::Bool(true)));

        let expected = ASTNode {
            token: Token::Operator(Operator::Or),
            left: Some(Box::new(left_and)),
            right: Some(Box::new(right_and)),
        };

        assert_eq!(results, expected);
    }

    #[test]
    fn test_construct_ast_equivalence_precedence() {
        let mut lexer = Lexer::new("~1 v ~0 <=> 0");
        let results = construct_ast(&mut lexer).unwrap();

        let mut or = ASTNode::new(Token::Operator(Operator::Or));
        let mut not_left = ASTNode::new(Token::Operator(Operator::Not));
        let mut not_right = ASTNode::new(Token::Operator(Operator::Not));

        not_left.add_left_token(Token::Value(Value::Bool(true)));
        not_right.add_left_token(Token::Value(Value::Bool(false)));

        or.add_left_child(not_left);
        or.add_right_child(not_right);

        let right = ASTNode::new(Token::Value(Value::Bool(false)));

        let expected = ASTNode {
            token: Token::Operator(Operator::Equivalence),
            left: Some(Box::new(or)),
            right: Some(Box::new(right)),
        };

        assert_eq!(results, expected);
    }
}

// This is my own alternative implementation of parser that built the AST
// recursively rather than with stacks.
// It was not used in the end, because of worse ability to handle operator
// precedence.
//
// pub fn construct_ast_alt(mut root: ASTNode, lexer: &mut Lexer) -> Result<ASTNode> {
//     while let Some(token) = lexer.next() {
//         println!("Token: {}", token);
//         match token {
//             Token::Empty => unimplemented!(),
//             Token::Value(_) => handle_value(&mut root, token)?,
//             Token::Operator(op) if op == Operator::ParenthisOpen => {
//                 // FIXME: bug here - root stays empty
//                 println!("Parent open, descending");
//                 let new_root = ASTNode {
//                     token: Token::Empty,
//                     left: None,
//                     right: None,
//                 };
//                 let sub_node = construct_ast_custom(new_root, lexer)?;
//                 println!("Root: {}, new root: {}", root, sub_node);
//                 if let Token::Empty = root.token {
//                     root.left = Some(Box::new(sub_node));
//                 } else {
//                     root.right = Some(Box::new(sub_node));
//                 }
//             }
//             Token::Operator(op) if op == Operator::ParenthisClosed => return Ok(root),
//             Token::Operator(_) => {
//                 root = handle_operator(root, token)?;
//             }
//         }
//     }
//     Ok(root)
// }

// fn handle_value(root: &mut ASTNode, token: Token) -> Result<()> {
//     match root.token {
//         Token::Empty => root.add_left_token(token),
//         Token::Operator(ref operator) => match operator {
//             Operator::And => {
//                 match root.right {
//                     Some(ref mut right) => {
//                         descend_right(right, token);
//                     }
//                     None => {
//                         root.add_right_token(token);
//                     }
//                 };
//             }
//             Operator::Or => {
//                 match root.right {
//                     Some(ref mut right) => {
//                         descend_right(right, token);
//                     }
//                     None => {
//                         root.add_right_token(token);
//                     }
//                 };
//             }
//             Operator::Not => {
//                 // FIXME: Check me
//                 println!("HERE: {}", root);
//                 if let Some(_) = root.left {
//                     root.add_right_token(token);
//                 } else {
//                     root.add_left_token(token);
//                 }
//             }
//             _ => unimplemented!(),
//         },
//         _ => return Err(anyhow!("Expected operator after a value")),
//     };
//     Ok(())
// }

// fn handle_operator(mut root: ASTNode, token: Token) -> Result<ASTNode> {
//     println!("Root token: {}", root);
//     match root.token {
//         Token::Empty => {
//             println!("Root token was empty, setting token to: {}", token);
//             root.token = token;
//         }
//         Token::Value(_) => return Err(anyhow!("Value can never be a root")),
//         Token::Operator(ref op) => match op {
//             Operator::Not => {
//                 println!("Root: {}, Parsing: {}", root.token, token);
//                 let new_root = root.make_new_root_left(token);
//                 return Ok(new_root);
//             }
//             Operator::Or => {
//                 // take right child of root and place node there
//                 // make this child left of this node
//                 let right = root.right.take();
//                 let node = ASTNode {
//                     token,
//                     left: right,
//                     right: None,
//                 };
//                 root.right = Some(Box::new(node));
//             }
//             Operator::And => {
//                 // FIXME: This probably doesn't cover all cases
//                 match token {
//                     Token::Value(_) => {
//                         root.add_right_token(token);
//                     }
//                     Token::Operator(_) => {
//                         let new_root = root.make_new_root_left(token);
//                         return Ok(new_root);
//                     }
//                     _ => unimplemented!(),
//                 }
//             }
//             _ => unimplemented!(),
//         },
//         // FIXME: Cover missing
//         _ => unimplemented!(),
//     };
//     Ok(root)
// }

// fn descend_right(node: &mut Box<ASTNode>, token: Token) {
//     match node.right {
//         Some(ref mut right) => descend_right(right, token),
//         None => node.add_right_token(token),
//     };
// }
