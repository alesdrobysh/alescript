mod token;
mod lexer;

use lexer::Lexer;
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

    println!("Tokenizing: {}", file_path);
    println!("{:-<60}\n", "");

    let lexer = Lexer::new(&source);

    for token in lexer {
        println!("{}", token);
    }
}
