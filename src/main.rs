use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::env;

use logic_solver::parser::{ASTNode, construct_ast};
use logic_solver::lexer::Lexer;

fn parse(contents: &str) -> Result<ASTNode> {
    let mut lexer = Lexer::new(contents);
    let root = construct_ast(&mut lexer)?;
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
