use std::fs;

use lexer::Lexer;
use parser::Parser;

mod analyzer;
mod lexer;
mod parser;

fn main() -> Result<(), String> {
    let contents = fs::read_to_string("test.boo").expect("Unable to read file to string");

    let mut lexer = Lexer::new(&contents);
    let tokens = lexer
        .tokenize()
        .map_err(|e| format!("Lexer error: {}", e))?;

    let mut parser = Parser::new(tokens);
    let ast = parser
        .parse_program()
        .map_err(|e| format!("Parser error: {}", e))?;

    println!("AST: {:#?}", ast);

    let mut typechecker = analyzer::TypeChecker::new();
    let result = typechecker.check_program(ast);

    match result {
        Ok(_) => println!("Typechecker: OK"),
        Err(e) => println!("Typechecker error: {}", e),
    }

    Ok(())
}
