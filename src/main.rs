mod token;
mod lexer;

use lexer::Lexer;

fn main() {
    let source = r#"
// Example alescript program
brew lager from water, 1 barley, 2 hops, 1 yeast.

wait for 5 days.

taste lager.

toast "hello, world!".

if lager is stronger than 5.0%:
    toast "strong brew!"
else:
    toast "mild brew."

recipe fibonacci(n) {
    brew a from water, barley.
    wait for 1 day.
    b
}
"#;

    println!("Tokenizing alescript source:\n{}", source);
    println!("\n{:-<60}", "");
    println!("Tokens:");
    println!("{:-<60}\n", "");

    let lexer = Lexer::new(source);

    for token in lexer {
        println!("{}", token);
    }
}
