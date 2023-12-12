use crate::lexer::{Operator, Token, Value};
use crate::parser::ASTNode;

pub fn evaluate(node: ASTNode) -> Option<bool> {
    let l_value = node.left.and_then(|left| evaluate(*left));
    let r_value = node.right.and_then(|right| evaluate(*right));

    match node.token {
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::lexer::Lexer;
    use crate::parser::construct_ast;

    use super::{eval_binary, evaluate};

    #[test]
    fn test_eval_binary() {
        let result = eval_binary(Some(true), Some(true), |a, b| a && b);

        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_eval_binary_should_handle_missing_value() {
        let result = eval_binary(Some(true), None, |a, b| a && b);

        assert_eq!(result, None);
    }

    // Truth tables tests
    //
    // Assumption warning!
    // It's assumed here that the lexer and the parser are working correctly
    // And can be used for testing this module.

    #[rstest]
    #[case("1", Some(true))]
    #[case("0", Some(false))]
    fn test_evaluate_base_bool_evaluation(#[case] expr: &str, #[case] expected: Option<bool>) {
        let mut lexer = Lexer::new(expr);
        let root = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("~1", Some(false))]
    #[case("~0", Some(true))]
    fn test_evaluate_negation(#[case] expr: &str, #[case] expected: Option<bool>) {
        let mut lexer = Lexer::new(expr);
        let root = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1 ^ 1", Some(true))]
    #[case("1 ^ 0", Some(false))]
    #[case("0 ^ 1", Some(false))]
    #[case("0 ^ 0", Some(false))]
    fn test_evaluate_conjunction(#[case] expr: &str, #[case] expected: Option<bool>) {
        let mut lexer = Lexer::new(expr);
        let root = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1 v 1", Some(true))]
    #[case("1 v 0", Some(true))]
    #[case("0 v 1", Some(true))]
    #[case("0 v 0", Some(false))]
    fn test_evaluate_disjunction(#[case] expr: &str, #[case] expected: Option<bool>) {
        let mut lexer = Lexer::new(expr);
        let root = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1 => 1", Some(true))]
    #[case("1 => 0", Some(false))]
    #[case("0 => 1", Some(true))]
    #[case("0 => 0", Some(true))]
    fn test_evaluate_implication(#[case] expr: &str, #[case] expected: Option<bool>) {
        let mut lexer = Lexer::new(expr);
        let root = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1 <=> 1", Some(true))]
    #[case("1 <=> 0", Some(false))]
    #[case("0 <=> 1", Some(false))]
    #[case("0 <=> 0", Some(true))]
    fn test_evaluate_equivalence(#[case] expr: &str, #[case] expected: Option<bool>) {
        let mut lexer = Lexer::new(expr);
        let root = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1 ^ 0 v 1", Some(true))]
    #[case("(1 => 0) ^ 1)", Some(false))]
    #[case("~(1 ^ 1)", Some(false))]
    #[case("~1 v ~1 <=> 0", Some(true))]
    #[case("~1 v ~0 <=> ~(1 ^ 0)", Some(true))]
    #[case("((1 v 0) => 0) ^ 1", Some(false))]
    fn test_evaluate_complex_expressions(#[case] expr: &str, #[case] expected: Option<bool>) {
        let mut lexer = Lexer::new(expr);
        let root = construct_ast(&mut lexer).unwrap();

        let result = evaluate(root);

        assert_eq!(result, expected);
    }
}
