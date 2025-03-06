use std::fs;

use bytecode::Bytecode;
use lexer::Lexer;
use parser::Parser;
use vm::VM;

mod analyzer;
mod bytecode;
mod lexer;
mod parser;
mod vm;

fn main() -> Result<(), String> {
    let contents = fs::read_to_string("test.boo").expect("Unable to read file to string");

    let mut lexer = Lexer::new(&contents);
    let tokens = lexer
        .tokenize()
        .map_err(|e| format!("Lexer error: {}", e))?;

    println!("Tokens: {:#?}", tokens);

    let mut parser = Parser::new(tokens);
    let ast = parser
        .parse_program()
        .map_err(|e| format!("Parser error: {}", e))?;

    println!("AST: {:#?}", ast);

    let mut typechecker = analyzer::TypeChecker::new();
    let result = typechecker.check_program(ast.clone());

    match result {
        Ok(_) => println!("Typechecker: OK"),
        Err(e) => println!("Typechecker error: {}", e),
    }

    let mut bytecode_compiler = Bytecode::new();
    let bytecode = bytecode_compiler.from_program(ast)?;
    println!("Bytecode: {:#?}", bytecode);

    let mut vm = VM::new(bytecode);
    let result = vm.run();

    match result {
        Ok(Some(value)) => println!("VM: OK, result: {:?}", value),
        Ok(None) => println!("VM: OK, no result"),
        Err(e) => println!("VM error: {}", e),
    }

    Ok(())
}
