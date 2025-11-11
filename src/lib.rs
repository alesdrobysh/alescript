mod ast;
mod interpreter;
mod lexer;
mod parser;
mod runtime;
mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_alescript(source: &str) -> String {
    // Set panic hook for better error messages in browser console
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // Lexer: tokenize the source
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    // Parser: build AST
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(prog) => prog,
        Err(err) => {
            return format!("Parse error at {}:{}: {}", err.line, err.column, err.message);
        }
    };

    // Interpreter: execute the program
    let mut interpreter = Interpreter::new();
    match interpreter.execute(&program) {
        Ok(_) => {
            // Get the output that was collected during execution
            interpreter.get_output()
        }
        Err(err) => format!("Runtime error: {}", err.message),
    }
}
