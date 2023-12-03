use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::env;

use logic_solver::parser::{ASTNode, construct_ast, construct_rpn, construct_ast_from_rpn};
use logic_solver::lexer::{Lexer, Token};

fn parse(contents: &str) -> Result<ASTNode> {
    let mut root = ASTNode {
        token: Token::Empty,
        left: None,
        right: None,
    };
    let mut lexer = Lexer::new(contents);
    let queue = construct_rpn(&mut lexer);
    root = construct_ast_from_rpn(root, queue)?;
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
