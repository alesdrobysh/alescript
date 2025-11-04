use crate::token::{Token, TokenType};
use std::iter::Peekable;
use std::str::Chars;

/// Lexer for alescript source code
/// Produces an iterable stream of tokens
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    current_line: usize,
    current_column: usize,
    indent_stack: Vec<usize>, // Track indentation levels
    pending_tokens: Vec<Token>, // Queue for DEDENT tokens
    at_line_start: bool,
    eof_emitted: bool, // Track if EOF has been emitted
    _phantom: std::marker::PhantomData<&'a ()>, // Keep lifetime parameter
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            chars: source.chars().peekable(),
            current_line: 1,
            current_column: 0,
            indent_stack: vec![0], // Start with base indentation
            pending_tokens: Vec::new(),
            at_line_start: true,
            eof_emitted: false,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Peek at the next character without consuming it
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Advance to the next character
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.chars.next() {
            if ch == '\n' {
                self.current_line += 1;
                self.current_column = 0;
            } else {
                self.current_column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    /// Skip whitespace (spaces and tabs), but not newlines
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Handle indentation at the start of a line
    fn handle_indentation(&mut self) -> Option<Token> {
        let mut indent_level = 0;
        let start_column = self.current_column;

        // Count spaces/tabs
        while let Some(ch) = self.peek() {
            if ch == ' ' {
                indent_level += 1;
                self.advance();
            } else if ch == '\t' {
                indent_level += 4; // Treat tab as 4 spaces
                self.advance();
            } else {
                break;
            }
        }

        // Skip empty lines
        if let Some(ch) = self.peek() {
            if ch == '\n' {
                return None; // Skip empty line
            }
        }

        let current_indent = *self.indent_stack.last().unwrap();

        if indent_level > current_indent {
            // INDENT
            self.indent_stack.push(indent_level);
            Some(Token::new(
                TokenType::Indent,
                " ".repeat(indent_level),
                self.current_line,
                start_column,
            ))
        } else if indent_level < current_indent {
            // DEDENT (possibly multiple)
            let mut dedent_count = 0;
            while let Some(&stack_level) = self.indent_stack.last() {
                if stack_level <= indent_level {
                    break;
                }
                self.indent_stack.pop();
                dedent_count += 1;
            }

            // Generate DEDENT tokens
            for _ in 0..dedent_count {
                self.pending_tokens.push(Token::new(
                    TokenType::Dedent,
                    String::new(),
                    self.current_line,
                    start_column,
                ));
            }

            // Return the first DEDENT
            self.pending_tokens.pop()
        } else {
            None // Same indentation level, no token
        }
    }

    /// Scan an identifier or keyword
    fn scan_identifier(&mut self) -> Token {
        let start_line = self.current_line;
        let start_column = self.current_column;
        let mut lexeme = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                lexeme.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let token_type = TokenType::keyword(&lexeme)
            .unwrap_or_else(|| TokenType::Identifier(lexeme.clone()));

        Token::new(token_type, lexeme, start_line, start_column)
    }

    /// Scan a number (integer or float) or percentage
    fn scan_number(&mut self) -> Token {
        let start_line = self.current_line;
        let start_column = self.current_column;
        let mut lexeme = String::new();

        // Scan integer part
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                lexeme.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for decimal point
        if self.peek() == Some('.') {
            // Peek ahead to see if it's followed by a digit
            let mut temp_chars = self.chars.clone();
            temp_chars.next(); // Skip the '.'
            if let Some(next_ch) = temp_chars.peek() {
                if next_ch.is_ascii_digit() {
                    lexeme.push('.');
                    self.advance();

                    // Scan fractional part
                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_digit() {
                            lexeme.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // Check for percentage
        if self.peek() == Some('%') {
            self.advance();
            let value = lexeme.parse::<f64>().unwrap();
            Token::new(
                TokenType::Percentage(value),
                format!("{}%", lexeme),
                start_line,
                start_column,
            )
        } else {
            let value = lexeme.parse::<f64>().unwrap();
            Token::new(TokenType::Number(value), lexeme, start_line, start_column)
        }
    }

    /// Scan a string literal
    fn scan_string(&mut self) -> Token {
        let start_line = self.current_line;
        let start_column = self.current_column;
        self.advance(); // Skip opening quote

        let mut value = String::new();
        let mut lexeme = String::from("\"");

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                lexeme.push('"');
                break;
            } else if ch == '\\' {
                // Handle escape sequences
                self.advance();
                lexeme.push('\\');
                if let Some(escaped) = self.advance() {
                    lexeme.push(escaped);
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        _ => {
                            value.push('\\');
                            value.push(escaped);
                        }
                    }
                }
            } else {
                value.push(ch);
                lexeme.push(ch);
                self.advance();
            }
        }

        Token::new(TokenType::String(value), lexeme, start_line, start_column)
    }

    /// Scan a comment
    fn scan_comment(&mut self) -> Token {
        let start_line = self.current_line;
        let start_column = self.current_column;
        self.advance(); // Skip first '/'
        self.advance(); // Skip second '/'

        let mut comment = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            comment.push(ch);
            self.advance();
        }

        Token::new(
            TokenType::Comment(comment.trim().to_string()),
            format!("//{}", comment),
            start_line,
            start_column,
        )
    }

    /// Get the next token from the source
    fn next_token(&mut self) -> Option<Token> {
        // Return pending tokens first (e.g., DEDENT)
        if let Some(token) = self.pending_tokens.pop() {
            return Some(token);
        }

        // Handle indentation at line start
        if self.at_line_start {
            self.at_line_start = false;
            if let Some(token) = self.handle_indentation() {
                return Some(token);
            }
        }

        self.skip_whitespace();

        let ch = self.peek()?;
        let line = self.current_line;
        let column = self.current_column;

        match ch {
            '\n' => {
                self.advance();
                self.at_line_start = true;
                Some(Token::new(TokenType::Newline, "\n".to_string(), line, column))
            }
            '.' => {
                self.advance();
                Some(Token::new(TokenType::Period, ".".to_string(), line, column))
            }
            ':' => {
                self.advance();
                Some(Token::new(TokenType::Colon, ":".to_string(), line, column))
            }
            ',' => {
                self.advance();
                Some(Token::new(TokenType::Comma, ",".to_string(), line, column))
            }
            '(' => {
                self.advance();
                Some(Token::new(TokenType::LeftParen, "(".to_string(), line, column))
            }
            ')' => {
                self.advance();
                Some(Token::new(TokenType::RightParen, ")".to_string(), line, column))
            }
            '[' => {
                self.advance();
                Some(Token::new(
                    TokenType::LeftBracket,
                    "[".to_string(),
                    line,
                    column,
                ))
            }
            ']' => {
                self.advance();
                Some(Token::new(
                    TokenType::RightBracket,
                    "]".to_string(),
                    line,
                    column,
                ))
            }
            '{' => {
                self.advance();
                Some(Token::new(TokenType::LeftBrace, "{".to_string(), line, column))
            }
            '}' => {
                self.advance();
                Some(Token::new(
                    TokenType::RightBrace,
                    "}".to_string(),
                    line,
                    column,
                ))
            }
            '=' => {
                self.advance();
                Some(Token::new(TokenType::Equal, "=".to_string(), line, column))
            }
            '/' => {
                self.advance();
                if self.peek() == Some('/') {
                    // Put back the first '/'
                    self.current_column -= 1;
                    Some(self.scan_comment())
                } else {
                    // Just a regular slash (not used in alescript, but handle it)
                    Some(Token::new(TokenType::Identifier("/".to_string()), "/".to_string(), line, column))
                }
            }
            '"' => Some(self.scan_string()),
            _ if ch.is_ascii_digit() => Some(self.scan_number()),
            _ if ch.is_alphabetic() || ch == '_' => Some(self.scan_identifier()),
            _ => {
                // Unknown character, skip it
                self.advance();
                self.next_token()
            }
        }
    }
}

/// Implement Iterator trait for Lexer
impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // If EOF was already emitted, return None
        if self.eof_emitted {
            return None;
        }

        let token = self.next_token();

        // If we've reached the end, emit DEDENT tokens for remaining indentation
        if token.is_none() && self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            return Some(Token::new(
                TokenType::Dedent,
                String::new(),
                self.current_line,
                self.current_column,
            ));
        }

        // Emit EOF at the very end
        if token.is_none() && self.indent_stack.len() == 1 && !self.eof_emitted {
            self.eof_emitted = true;
            return Some(Token::new(
                TokenType::Eof,
                String::new(),
                self.current_line,
                self.current_column,
            ));
        }

        token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_brew() {
        let source = "brew lager from water, barley.";
        let lexer = Lexer::new(source);
        let tokens: Vec<Token> = lexer.collect();

        assert_eq!(tokens[0].token_type, TokenType::Brew);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("lager".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::From);
        assert_eq!(tokens[3].token_type, TokenType::Water);
        assert_eq!(tokens[4].token_type, TokenType::Comma);
        assert_eq!(tokens[5].token_type, TokenType::Barley);
        assert_eq!(tokens[6].token_type, TokenType::Period);
    }

    #[test]
    fn test_numbers_and_percentages() {
        let source = "5 3.14 5.2%";
        let lexer = Lexer::new(source);
        let tokens: Vec<Token> = lexer.collect();

        assert_eq!(tokens[0].token_type, TokenType::Number(5.0));
        assert_eq!(tokens[1].token_type, TokenType::Number(3.14));
        assert_eq!(tokens[2].token_type, TokenType::Percentage(5.2));
    }

    #[test]
    fn test_string_literal() {
        let source = r#"toast "hello, world!"."#;
        let lexer = Lexer::new(source);
        let tokens: Vec<Token> = lexer.collect();

        assert_eq!(tokens[0].token_type, TokenType::Toast);
        assert_eq!(tokens[1].token_type, TokenType::String("hello, world!".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Period);
    }

    #[test]
    fn test_comment() {
        let source = "// this is a comment\ntaste lager.";
        let lexer = Lexer::new(source);
        let tokens: Vec<Token> = lexer.collect();

        assert_eq!(tokens[0].token_type, TokenType::Comment("this is a comment".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::Newline);
        assert_eq!(tokens[2].token_type, TokenType::Taste);
    }
}
