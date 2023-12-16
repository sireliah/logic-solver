use anyhow::{anyhow, Result};

use crate::lexer::{Operator, Token, Value};
use crate::parser::{ASTNode, StoredVariables};

pub fn evaluate(node: ASTNode, vars: &StoredVariables) -> Result<bool> {
    match node.token {
        Token::Value(value) => match value {
            Value::Bool(val) => Ok(val),
            Value::Variable(var) => match vars.get(&var) {
                Some(var_value) => Ok(*var_value),
                None => Err(anyhow!("Undefined variable {}", var)),
            },
        },
        Token::Operator(op) => match op {
            Operator::Equivalence => eval_binary(node.left, node.right, vars, |a, b| a == b),
            Operator::Implication => eval_binary(node.left, node.right, vars, implication),
            Operator::Or => eval_binary(node.left, node.right, vars, |a, b| a || b),
            Operator::And => eval_binary(node.left, node.right, vars, |a, b| a && b),
            Operator::Not => match node.left {
                Some(left) => Ok(!evaluate(*left, vars)?),
                None => Err(anyhow!("Cannot evaluate negation without value")),
            },
            other => Err(anyhow!("Unexpected operator {:?}", other)),
        },
    }
}

fn eval_binary(
    l_node: Option<Box<ASTNode>>,
    r_node: Option<Box<ASTNode>>,
    vars: &StoredVariables,
    func: fn(bool, bool) -> bool,
) -> Result<bool> {
    match (l_node, r_node) {
        (Some(left), Some(right)) => {
            let l_result = evaluate(*left, vars)?;
            let r_result = evaluate(*right, vars)?;
            Ok(func(l_result, r_result))
        }
        (Some(left), None) => Err(anyhow!(
            "Expected two values for infix function, got only left: {}",
            left
        )),
        (None, Some(right)) => Err(anyhow!(
            "Expected two values for infix function, got only right: {}",
            right
        )),
        _ => Err(anyhow!("Expected two values for infix function, got none")),
    }
}

fn implication(l_value: bool, r_value: bool) -> bool {
    if l_value & !r_value {
        false
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::collections::HashMap;

    use crate::lexer::{Token, Value};
    use crate::parser::construct_ast;
    use crate::{lexer::Lexer, parser::ASTNode};

    use super::{eval_binary, evaluate};

    #[test]
    fn test_eval_binary() {
        let left = Box::new(ASTNode::new(Token::Value(Value::Bool(true))));
        let right = Box::new(ASTNode::new(Token::Value(Value::Bool(false))));

        let result = eval_binary(Some(left), Some(right), &HashMap::new(), |a, b| a && b).unwrap();

        assert_eq!(result, false);
    }

    #[test]
    fn test_eval_binary_should_handle_missing_value() {
        let left = Box::new(ASTNode::new(Token::Value(Value::Bool(true))));
        let result = eval_binary(Some(left), None, &HashMap::new(), |a, b| a && b);

        assert!(result.is_err());
    }

    // Truth tables tests
    //
    // Assumption warning!
    // It's assumed here that the lexer and the parser are working correctly
    // And can be used for testing this module.

    #[rstest]
    #[case("1", true)]
    #[case("0", false)]
    fn test_evaluate_base_bool_evaluation(#[case] expr: &str, #[case] expected: bool) {
        let mut lexer = Lexer::new(expr);
        let (root, vars) = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root, &vars).unwrap();

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("~1", false)]
    #[case("~0", true)]
    fn test_evaluate_negation(#[case] expr: &str, #[case] expected: bool) {
        let mut lexer = Lexer::new(expr);
        let (root, vars) = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root, &vars).unwrap();

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1 ^ 1", true)]
    #[case("1 ^ 0", false)]
    #[case("0 ^ 1", false)]
    #[case("0 ^ 0", false)]
    fn test_evaluate_conjunction(#[case] expr: &str, #[case] expected: bool) {
        let mut lexer = Lexer::new(expr);
        let (root, vars) = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root, &vars).unwrap();

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1 v 1", true)]
    #[case("1 v 0", true)]
    #[case("0 v 1", true)]
    #[case("0 v 0", false)]
    fn test_evaluate_disjunction(#[case] expr: &str, #[case] expected: bool) {
        let mut lexer = Lexer::new(expr);
        let (root, vars) = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root, &vars).unwrap();

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1 => 1", true)]
    #[case("1 => 0", false)]
    #[case("0 => 1", true)]
    #[case("0 => 0", true)]
    fn test_evaluate_implication(#[case] expr: &str, #[case] expected: bool) {
        let mut lexer = Lexer::new(expr);
        let (root, vars) = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root, &vars).unwrap();

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1 <=> 1", true)]
    #[case("1 <=> 0", false)]
    #[case("0 <=> 1", false)]
    #[case("0 <=> 0", true)]
    fn test_evaluate_equivalence(#[case] expr: &str, #[case] expected: bool) {
        let mut lexer = Lexer::new(expr);
        let (root, vars) = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root, &vars).unwrap();

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1 ^ 0 v 1", true)]
    #[case("(1 => 0) ^ 1)", false)]
    #[case("~(1 ^ 1)", false)]
    #[case("~1 v ~1 <=> 0", true)]
    #[case("~1 v ~0 <=> ~(1 ^ 0)", true)]
    #[case("((1 v 0) => 0) ^ 1", false)]
    #[case("p := 1 q := 0 r := 1 p ^ q ^ r", false)]
    fn test_evaluate_complex_expressions(#[case] expr: &str, #[case] expected: bool) {
        let mut lexer = Lexer::new(expr);
        let (root, vars) = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root, &vars).unwrap();

        assert_eq!(result, expected);
    }
}
