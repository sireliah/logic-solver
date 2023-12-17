use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Read;
use std::env;
use std::path::Path;

use env_logger::Env;

use logic_solver::parser::{ASTNode, StoredVariables, construct_ast};
use logic_solver::lexer::Lexer;
use logic_solver::interpreter::evaluate;

fn parse(contents: &str) -> Result<(ASTNode, StoredVariables)> {
    let mut lexer = Lexer::new(contents);
    let (root, variables) = construct_ast(&mut lexer)?;
    Ok((root, variables))
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = match args.len() {
        1 => return Err(anyhow!("Please provide file path to the statement")),
        2 => &args[1],
        _ => return Err(anyhow!("Expected just one file path")),
    };

    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);
    let mut file = File::open(file_path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let (ast_root, variables) = parse(&buffer)?;

    let graph_path = Path::new("graph.dot");
    ast_root.visualize_graph(&graph_path)?;

    let res = evaluate(ast_root, &variables)?;
    println!("Result: {}", res);
    Ok(())
}
