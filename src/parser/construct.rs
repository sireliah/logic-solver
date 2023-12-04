use std::unimplemented;

use anyhow::{anyhow, Result};

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

pub fn construct_ast(lexer: &mut Lexer) -> Result<ASTNode> {
    let mut operators: Vec<Operator> = Vec::new();
    let mut tree_queue: Vec<ASTNode> = Vec::new();

    while let Some(token) = lexer.next() {
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
                    // Check last operator in the stack
                    match operators.pop() {
                        // This is countermeasure for the fact that parentheses have precedence in enum
                        // If possible, this should be fixed.
                        Some(prev_op) if prev_op == Operator::ParenthisOpen => {
                            operators.push(prev_op);
                            operators.push(current_op);
                        }
                        Some(prev_op) => {
                            if current_op > prev_op {
                                operators.push(prev_op);
                            } else {
                                make_node(&mut tree_queue, prev_op);
                            };
                            operators.push(current_op);
                        }
                        None => operators.push(current_op),
                    };
                }
            },
            Token::Empty => unimplemented!(),
        }
    }
    for op in operators.into_iter().rev() {
        make_node(&mut tree_queue, op);
    }

    tree_queue.pop().ok_or(anyhow!("Invalid syntax, expected at least one AST node left"))
}

pub fn construct_ast_custom(mut root: ASTNode, lexer: &mut Lexer) -> Result<ASTNode> {
    while let Some(token) = lexer.next() {
        println!("Token: {}", token);
        match token {
            Token::Empty => unimplemented!(),
            Token::Value(_) => handle_value(&mut root, token)?,
            Token::Operator(op) if op == Operator::ParenthisOpen => {
                // FIXME: bug here - root stays empty
                println!("Parent open, descending");
                let new_root = ASTNode {
                    token: Token::Empty,
                    left: None,
                    right: None,
                };
                let sub_node = construct_ast_custom(new_root, lexer)?;
                println!("Root: {}, new root: {}", root, sub_node);
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
                println!("HERE: {}", root);
                if let Some(_) = root.left {
                    root.add_right_child(token);
                } else {
                    root.add_left_child(token);
                }
            }
            _ => unimplemented!(),
        },
        _ => return Err(anyhow!("Expected operator after a value")),
    };
    Ok(())
}

fn handle_operator(mut root: ASTNode, token: Token) -> Result<ASTNode> {
    println!("Root token: {}", root);
    match root.token {
        Token::Empty => {
            println!("Root token was empty, setting token to: {}", token);
            root.token = token;
        }
        Token::Value(_) => return Err(anyhow!("Value can never be a root")),
        Token::Operator(ref op) => match op {
            Operator::Not => {
                println!("Root: {}, Parsing: {}", root.token, token);
                let new_root = root.make_new_root_left(token);
                return Ok(new_root);
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
                // FIXME: This probably doesn't cover all cases
                match token {
                    Token::Value(_) => {
                        root.add_right_child(token);
                    }
                    Token::Operator(_) => {
                        let new_root = root.make_new_root_left(token);
                        return Ok(new_root);
                    }
                    _ => unimplemented!(),
                }
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
        let results = construct_ast(&mut lexer);

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
        let results = construct_ast(&mut lexer);

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
        let results = construct_ast(&mut lexer);

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
        let results = construct_ast(&mut lexer);

        let left = ASTNode::new(Token::Value(Value::Bool(true)));

        let mut or = ASTNode::new(Token::Operator(Operator::Or));
        or.add_left_child(Token::Value(Value::Bool(false)));
        or.add_right_child(Token::Value(Value::Bool(true)));

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
        let results = construct_ast(&mut lexer);

        let right = ASTNode::new(Token::Value(Value::Bool(true)));

        let mut and = ASTNode::new(Token::Operator(Operator::And));
        and.add_left_child(Token::Value(Value::Bool(true)));
        and.add_right_child(Token::Value(Value::Bool(false)));

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
        let results = construct_ast(&mut lexer);

        let right = ASTNode::new(Token::Value(Value::Bool(false)));

        let mut not = ASTNode::new(Token::Operator(Operator::Not));
        not.add_left_child(Token::Value(Value::Bool(true)));

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
        let results = construct_ast(&mut lexer);

        let mut not = ASTNode::new(Token::Operator(Operator::Not));
        not.add_left_child(Token::Value(Value::Bool(true)));
        let mut not2 = ASTNode::new(Token::Operator(Operator::Not));
        not2.add_left_child(Token::Value(Value::Bool(false)));

        let expected = ASTNode {
            token: Token::Operator(Operator::Or),
            left: Some(Box::new(not)),
            right: Some(Box::new(not2)),
        };

        assert_eq!(results, expected);
    }

    // #[test]
    // fn test_construct_rpn_or_before_and() {
    //     let mut lexer = Lexer::new("1 v 0 ^ 1");
    //     let results = construct_rpn(&mut lexer);

    //     let expected = VecDeque::from(vec![
    //         Token::Value(Value::Bool(true)),
    //         Token::Value(Value::Bool(false)),
    //         Token::Value(Value::Bool(true)),
    //         Token::Operator(Operator::And),
    //         Token::Operator(Operator::Or),
    //     ]);
    //     assert_eq!(results, expected);
    // }

    // #[test]
    // fn test_construct_rpn_parent_with_lower_prec() {
    //     let mut lexer = Lexer::new("1 ^ (0 v 1)");
    //     let results = construct_rpn(&mut lexer);

    //     let expected = VecDeque::from(vec![
    //         Token::Value(Value::Bool(true)),
    //         Token::Value(Value::Bool(false)),
    //         Token::Value(Value::Bool(true)),
    //         Token::Operator(Operator::Or),
    //         Token::Operator(Operator::And),
    //     ]);
    //     assert_eq!(results, expected);
    // }

    // #[test]
    // fn test_construct_rpn_parent_should_not_change_anything() {
    //     let mut lexer = Lexer::new("(1 ^ 0) v 1");
    //     let results = construct_rpn(&mut lexer);

    //     let expected = VecDeque::from(vec![
    //         Token::Value(Value::Bool(true)),
    //         Token::Value(Value::Bool(false)),
    //         Token::Operator(Operator::And),
    //         Token::Value(Value::Bool(true)),
    //         Token::Operator(Operator::Or),
    //     ]);
    //     assert_eq!(results, expected);
    // }

    // #[test]
    // fn test_construct_rpn_double_parent_should_not_change_anything() {
    //     let mut lexer = Lexer::new("((1 ^ 0) v 1)");
    //     let results = construct_rpn(&mut lexer);

    //     let expected = VecDeque::from(vec![
    //         Token::Value(Value::Bool(true)),
    //         Token::Value(Value::Bool(false)),
    //         Token::Operator(Operator::And),
    //         Token::Value(Value::Bool(true)),
    //         Token::Operator(Operator::Or),
    //     ]);
    //     assert_eq!(results, expected);
    // }

    // #[test]
    // fn test_construct_rpn_double_parent_should_not_change_anything2() {
    //     let mut lexer = Lexer::new("(1 ^ (0 v 1))");
    //     let results = construct_rpn(&mut lexer);

    //     let expected = VecDeque::from(vec![
    //         Token::Value(Value::Bool(true)),
    //         Token::Value(Value::Bool(false)),
    //         Token::Value(Value::Bool(true)),
    //         Token::Operator(Operator::Or),
    //         Token::Operator(Operator::And),
    //     ]);
    //     assert_eq!(results, expected);
    // }

    // #[test]
    // fn test_construct_rpn_repeated_and() {
    //     let mut lexer = Lexer::new("1 ^ 0 ^ 1 ^ 0");
    //     let results = construct_rpn(&mut lexer);

    //     let expected = VecDeque::from(vec![
    //         Token::Value(Value::Bool(true)),
    //         Token::Value(Value::Bool(false)),
    //         Token::Operator(Operator::And),
    //         Token::Value(Value::Bool(true)),
    //         Token::Operator(Operator::And),
    //         Token::Value(Value::Bool(false)),
    //         Token::Operator(Operator::And),
    //     ]);
    //     assert_eq!(results, expected);
    // }

    // #[test]
    // fn test_construct_rpn_negation() {
    //     let mut lexer = Lexer::new("~1 v ~0");
    //     let results = construct_rpn(&mut lexer);

    //     let expected = VecDeque::from(vec![
    //         Token::Value(Value::Bool(true)),
    //         Token::Operator(Operator::Not),
    //         Token::Value(Value::Bool(false)),
    //         Token::Operator(Operator::Not),
    //         Token::Operator(Operator::Or),
    //     ]);
    //     assert_eq!(results, expected);
    // }

    // #[test]
    // fn test_construct_rpn_negation_with_parentheses() {
    //     let mut lexer = Lexer::new("(1 v 0) ^ ~1");
    //     let results = construct_rpn(&mut lexer);

    //     let expected = VecDeque::from(vec![
    //         Token::Value(Value::Bool(true)),
    //         Token::Value(Value::Bool(false)),
    //         Token::Operator(Operator::Or),
    //         Token::Value(Value::Bool(true)),
    //         Token::Operator(Operator::Not),
    //         Token::Operator(Operator::And),
    //     ]);
    //     assert_eq!(results, expected);
    // }
}
