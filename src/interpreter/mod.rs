use crate::lexer::{Operator, Token, Value};
use crate::parser::ASTNode;

pub fn evaluate(root: ASTNode) -> Option<bool> {
    let l_value = root.left.and_then(|left| evaluate(*left));
    let r_value = root.right.and_then(|right| evaluate(*right));

    match root.token {
        Token::Empty => None,
        Token::Value(value) => match value {
            Value::Bool(val) => Some(val),
            Value::Variable(_var) => todo!(),
        },
        Token::Operator(op) => match op {
            Operator::Equivalence => eval_binary(l_value, r_value, |a, b| a == b),
            Operator::Implication => eval_binary(l_value, r_value, implication),
            Operator::Or => eval_binary(l_value, r_value, |a, b| a || b),
            Operator::And => eval_binary(l_value, r_value, |a, b| a && b),
            Operator::Not => l_value.and_then(|v| Some(!v)),
            _ => None,
        },
    }
}

fn eval_binary(
    l_value: Option<bool>,
    r_value: Option<bool>,
    func: fn(bool, bool) -> bool,
) -> Option<bool> {
    if let (Some(l_bool), Some(r_bool)) = (l_value, r_value) {
        Some(func(l_bool, r_bool))
    } else {
        None
    }
}

fn implication(l_value: bool, r_value: bool) -> bool {
    if l_value & !r_value {
        false
    } else {
        true
    }
}
