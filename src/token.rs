use std::fmt;

/// Token types for the alescript language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords - brewing operations
    Brew,
    From,
    Wait,
    For,
    Days,
    Day,
    Age,
    Until,
    Is,
    Abv,

    // Keywords - output
    Taste,
    Toast,

    // Keywords - arithmetic
    Mix,
    With,
    Double,
    By,
    Dilute,

    // Keywords - conditionals
    If,
    Else,
    Judge,

    // Keywords - comparisons
    Stronger,
    Than,
    Weaker,
    Not,

    // Keywords - loops
    Repeat,
    Times,
    Each,
    In,

    // Keywords - kegging
    Keg,

    // Keywords - barrels (arrays)
    Barrel,
    Add,
    To,
    Remove,
    Position,

    // Keywords - recipes (functions)
    Recipe,
    Relabel,
    As,

    // Ingredient keywords
    Water,
    Barley,
    Hops,
    Yeast,

    // Literals
    Identifier(String),
    Number(f64),
    String(String),
    Percentage(f64), // e.g., 5.2%

    // Symbols
    Period,          // .
    Colon,          // :
    Comma,          // ,
    LeftParen,      // (
    RightParen,     // )
    LeftBracket,    // [
    RightBracket,   // ]
    LeftBrace,      // {
    RightBrace,     // }
    Equal,          // =

    // Special
    Newline,
    Indent,
    Dedent,
    Comment(String),
    Eof,
}

/// Represents a token with its type, lexeme, and position in source
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} {:?} '{}'",
            self.line, self.column, self.token_type, self.lexeme
        )
    }
}

impl TokenType {
    /// Checks if a word is a keyword and returns the appropriate TokenType
    pub fn keyword(word: &str) -> Option<TokenType> {
        match word {
            // Brewing operations
            "brew" => Some(TokenType::Brew),
            "from" => Some(TokenType::From),
            "wait" => Some(TokenType::Wait),
            "for" => Some(TokenType::For),
            "days" => Some(TokenType::Days),
            "day" => Some(TokenType::Day),
            "age" => Some(TokenType::Age),
            "until" => Some(TokenType::Until),
            "is" => Some(TokenType::Is),
            "abv" => Some(TokenType::Abv),

            // Output
            "taste" => Some(TokenType::Taste),
            "toast" => Some(TokenType::Toast),

            // Arithmetic
            "mix" => Some(TokenType::Mix),
            "with" => Some(TokenType::With),
            "double" => Some(TokenType::Double),
            "by" => Some(TokenType::By),
            "dilute" => Some(TokenType::Dilute),

            // Conditionals
            "if" => Some(TokenType::If),
            "else" => Some(TokenType::Else),
            "judge" => Some(TokenType::Judge),

            // Comparisons
            "stronger" => Some(TokenType::Stronger),
            "than" => Some(TokenType::Than),
            "weaker" => Some(TokenType::Weaker),
            "not" => Some(TokenType::Not),

            // Loops
            "repeat" => Some(TokenType::Repeat),
            "times" => Some(TokenType::Times),
            "each" => Some(TokenType::Each),
            "in" => Some(TokenType::In),

            // Kegging
            "keg" => Some(TokenType::Keg),

            // Barrels
            "barrel" => Some(TokenType::Barrel),
            "add" => Some(TokenType::Add),
            "to" => Some(TokenType::To),
            "remove" => Some(TokenType::Remove),
            "position" => Some(TokenType::Position),

            // Recipes
            "recipe" => Some(TokenType::Recipe),
            "relabel" => Some(TokenType::Relabel),
            "as" => Some(TokenType::As),

            // Ingredients
            "water" => Some(TokenType::Water),
            "barley" => Some(TokenType::Barley),
            "hops" => Some(TokenType::Hops),
            "yeast" => Some(TokenType::Yeast),

            _ => None,
        }
    }
}
