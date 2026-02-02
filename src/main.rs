mod ast;
mod interpreter;
mod lexer;
mod parser;
mod runtime;
mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file.ales>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} examples/hello.ales", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];

    // Read the source file
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", file_path, err);
            process::exit(1);
        }
    };

    // Lexer: tokenize the source
    let lexer = Lexer::new(&source);
    let tokens: Vec<_> = lexer.collect();

    // Parser: build AST
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(prog) => prog,
        Err(err) => {
            eprintln!(
                "Parse error at {}:{}: {}",
                err.line, err.column, err.message
            );
            process::exit(1);
        }
    };

    // Interpreter: execute the program
    let mut interpreter = Interpreter::new();
    if let Err(err) = interpreter.execute(&program) {
        eprintln!("Runtime error: {}", err.message);
        process::exit(1);
    }

    // Print the output captured by the interpreter
    print!("{}", interpreter.get_output());
}
