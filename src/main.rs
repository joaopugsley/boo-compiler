use std::{fs, time::Instant};

use bytecode::Bytecode;
use lexer::Lexer;
use parser::Parser;
use vm::VM;

mod analyzer;
mod bytecode;
mod lexer;
mod parser;
mod stdlib;
mod vm;

fn main() -> Result<(), String> {
    let contents = fs::read_to_string("test.boo").expect("Unable to read file to string");

    let mut lexer = Lexer::new(&contents);
    let tokens = lexer.tokenize();

    if tokens.is_err() {
        return Err(format!("Lexer error: {}", tokens.err().unwrap()));
    }

    println!("Tokens: {:#?}", tokens);

    let mut parser = Parser::new(tokens.unwrap());
    let ast = parser.parse_program();

    if ast.is_err() {
        return Err(format!("Parser error: {}", ast.err().unwrap()));
    }

    // println!("AST: {:#?}", ast);

    let mut typechecker = analyzer::TypeChecker::new(ast.clone().unwrap());
    let result = typechecker.check_program();

    if result.is_err() {
        return Err(format!("Typechecker error: {}", result.err().unwrap()));
    }

    let mut bytecode_compiler = Bytecode::new(ast.unwrap());
    let bytecode = bytecode_compiler.compile();

    if bytecode.is_err() {
        return Err(format!(
            "Bytecode compiler error: {}",
            bytecode.err().unwrap()
        ));
    }

    // println!("Bytecode: {:#?}", bytecode);

    let mut vm = VM::new(bytecode.unwrap());

    let start = Instant::now();

    let result = vm.run();

    let duration = start.elapsed();

    if result.is_err() {
        println!("VM error: {}", result.err().unwrap());
    }

    println!("Execution time: {:?}", duration);

    Ok(())
}
