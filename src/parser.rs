use crate::ast::*;
use crate::token::{Token, TokenType};
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    current: Option<Token>,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    fn new(message: String, token: &Token) -> Self {
        ParseError {
            message,
            line: token.line,
            column: token.column,
        }
    }
}

type ParseResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Parser {
            tokens: tokens.into_iter().peekable(),
            current: None,
        };
        parser.advance();
        parser
    }

    fn advance(&mut self) {
        self.current = self.tokens.next();
    }

    fn current_token(&self) -> Result<&Token, ParseError> {
        self.current.as_ref().ok_or_else(|| ParseError {
            message: "Unexpected end of input".to_string(),
            line: 0,
            column: 0,
        })
    }

    fn expect(&mut self, expected: TokenType) -> ParseResult<Token> {
        let token = self.current_token()?.clone();
        if std::mem::discriminant(&token.token_type) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(token)
        } else {
            Err(ParseError::new(
                format!("Expected {:?}, got {:?}", expected, token.token_type),
                &token,
            ))
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(token) = &self.current {
            match token.token_type {
                TokenType::Newline | TokenType::Comment(_) => self.advance(),
                _ => break,
            }
        }
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut statements = Vec::new();
        self.skip_whitespace();

        while let Some(token) = &self.current {
            if token.token_type == TokenType::Eof {
                break;
            }
            self.skip_whitespace();
            if let Some(token) = &self.current {
                if token.token_type == TokenType::Eof {
                    break;
                }
                statements.push(self.parse_statement()?);
            }
            self.skip_whitespace();
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> ParseResult<Statement> {
        self.skip_whitespace();
        let token = self.current_token()?.clone();

        match &token.token_type {
            TokenType::Brew => self.parse_brew(),
            TokenType::Wait => self.parse_wait(),
            TokenType::Age => self.parse_age(),
            TokenType::Taste => self.parse_taste(),
            TokenType::Toast => self.parse_toast(),
            TokenType::Keg => self.parse_keg(),
            TokenType::Blend => self.parse_blend(),
            TokenType::Fortify => self.parse_fortify(),
            TokenType::Filter => self.parse_filter(),
            TokenType::Recipe => self.parse_recipe(),
            TokenType::Relabel => self.parse_relabel(),
            TokenType::Judge | TokenType::If => self.parse_judge(),
            TokenType::Repeat => self.parse_repeat(),
            TokenType::Barrel => self.parse_barrel_decl(),
            TokenType::Add => self.parse_add_to_barrel(),
            TokenType::Remove => self.parse_remove_from_barrel(),
            TokenType::End => {
                self.advance();
                Ok(Statement::End)
            }
            TokenType::Identifier(_) => {
                // Could be a function call
                let expr = self.parse_expression()?;
                self.expect(TokenType::Period)?;
                Ok(Statement::ExprStmt(expr))
            }
            _ => Err(ParseError::new(
                format!("Unexpected token: {:?}", token.token_type),
                &token,
            )),
        }
    }

    // brew lager from water, 1 barley, 2 hops, 1 yeast.
    // OR: brew selected = taplist position 1.
    fn parse_brew(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Brew)?;
        let name_token = self.current_token()?.clone();
        let name = match &name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected brew name".to_string(),
                    &name_token,
                ));
            }
        };
        self.advance();

        let token = self.current_token()?.clone();
        if token.token_type == TokenType::Equal {
            self.advance();
            let value = self.parse_expression()?;
            self.expect(TokenType::Period)?;
            Ok(Statement::Relabel { name, value })
        } else {
            self.expect(TokenType::From)?;

            let ingredients = self.parse_ingredients()?;
            self.expect(TokenType::Period)?;

            Ok(Statement::Brew { name, ingredients })
        }
    }

    fn parse_ingredients(&mut self) -> ParseResult<Vec<Ingredient>> {
        let mut ingredients = Vec::new();

        loop {
            let token = self.current_token()?.clone();

            // Check for quantity (number before ingredient)
            let quantity = if let TokenType::Number(n) = token.token_type {
                self.advance();
                n
            } else {
                1.0
            };

            // Parse ingredient type
            let token = self.current_token()?.clone();
            let ingredient_type = match token.token_type {
                TokenType::Water => IngredientType::Water,
                TokenType::Barley => IngredientType::Barley,
                TokenType::Hops => IngredientType::Hops,
                TokenType::Yeast => IngredientType::Yeast,
                _ => break,
            };
            self.advance();

            ingredients.push(Ingredient {
                name: ingredient_type,
                quantity,
            });

            // Check for comma (more ingredients)
            if let Some(token) = &self.current {
                if token.token_type == TokenType::Comma {
                    self.advance();
                    continue;
                }
            }
            break;
        }

        Ok(ingredients)
    }

    // wait for 5 days.
    fn parse_wait(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Wait)?;
        self.expect(TokenType::For)?;
        let days = self.parse_expression()?;
        // Optional: day or days
        if let Some(token) = &self.current {
            if matches!(token.token_type, TokenType::Day | TokenType::Days) {
                self.advance();
            }
        }
        self.expect(TokenType::Period)?;

        Ok(Statement::Wait { days })
    }

    // age lager until 5.2%.
    fn parse_age(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Age)?;
        let brew_name_token = self.current_token()?.clone();
        let brew_name = match &brew_name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected brew name".to_string(),
                    &brew_name_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::Until)?;

        // Optional 'is'
        if let Some(token) = &self.current {
            if token.token_type == TokenType::Is {
                self.advance();
            }
        }

        let target_token = self.current_token()?.clone();
        let target_abv = match target_token.token_type {
            TokenType::Percentage(p) => p,
            TokenType::Number(n) => n, // Allow bare numbers too
            _ => {
                return Err(ParseError::new(
                    "Expected percentage or number".to_string(),
                    &target_token,
                ));
            }
        };
        self.advance();

        // Optional 'abv'
        if let Some(token) = &self.current {
            if token.token_type == TokenType::Abv {
                self.advance();
            }
        }

        self.expect(TokenType::Period)?;

        Ok(Statement::Age {
            brew_name,
            target_abv,
        })
    }

    // taste lager.
    fn parse_taste(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Taste)?;
        let brew_name_token = self.current_token()?.clone();
        let brew_name = match &brew_name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected brew name".to_string(),
                    &brew_name_token,
                ));
            }
        };
        self.advance();
        self.expect(TokenType::Period)?;

        Ok(Statement::Taste { brew_name })
    }

    // toast "hello, world!".
    fn parse_toast(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Toast)?;
        let value = self.parse_expression()?;
        self.expect(TokenType::Period)?;

        Ok(Statement::Toast { value })
    }

    // keg lager.
    fn parse_keg(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Keg)?;
        let brew_name_token = self.current_token()?.clone();
        let brew_name = match &brew_name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected brew name".to_string(),
                    &brew_name_token,
                ));
            }
        };
        self.advance();
        self.expect(TokenType::Period)?;

        Ok(Statement::Keg { brew_name })
    }

    // blend lager with stout.
    fn parse_blend(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Blend)?;
        let target_token = self.current_token()?.clone();
        let target = match &target_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected brew name".to_string(),
                    &target_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::With)?;

        let source_token = self.current_token()?.clone();
        let source = match &source_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected brew name".to_string(),
                    &source_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::Period)?;

        Ok(Statement::Blend { target, source })
    }

    // fortify porter by 3.
    fn parse_fortify(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Fortify)?;
        let brew_name_token = self.current_token()?.clone();
        let brew_name = match &brew_name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected brew name".to_string(),
                    &brew_name_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::By)?;

        let factor = self.parse_expression()?;
        self.expect(TokenType::Period)?;

        Ok(Statement::Fortify { brew_name, factor })
    }

    // filter ipa by 2.
    fn parse_filter(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Filter)?;
        let brew_name_token = self.current_token()?.clone();
        let brew_name = match &brew_name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected brew name".to_string(),
                    &brew_name_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::By)?;

        let factor = self.parse_expression()?;
        self.expect(TokenType::Period)?;

        Ok(Statement::Filter { brew_name, factor })
    }

    // relabel temp as a.
    fn parse_relabel(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Relabel)?;
        let name_token = self.current_token()?.clone();
        let name = match &name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected identifier".to_string(),
                    &name_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::As)?;

        let value = self.parse_expression()?;

        self.expect(TokenType::Period)?;

        Ok(Statement::Relabel { name, value })
    }

    // recipe fibonacci(n) ... end.
    fn parse_recipe(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Recipe)?;
        let name_token = self.current_token()?.clone();
        let name = match &name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected recipe name".to_string(),
                    &name_token,
                ));
            }
        };
        self.advance();

        let params = self.parse_params()?;
        self.skip_whitespace();

        // Parse body as a block until we hit 'end'
        let mut body = Vec::new();
        let mut return_value = None;
        let mut indent_count = 0;

        // Expect indent for recipe body
        if let Some(token) = &self.current {
            if token.token_type == TokenType::Indent {
                indent_count += 1;
                self.advance();
                self.skip_whitespace();
            }
        }

        loop {
            self.skip_whitespace();
            if let Some(token) = &self.current {
                match token.token_type {
                    TokenType::End => {
                        self.advance();
                        self.expect(TokenType::Period)?;
                        break;
                    }
                    TokenType::Dedent => {
                        indent_count -= 1;
                        self.advance();
                        if indent_count == 0 {
                            // Should hit 'end' after this
                            continue;
                        }
                    }
                    _ => {
                        // Check if this is the last line (return value) - it's an expression without a statement keyword
                        let is_return_expr = matches!(
                            token.token_type,
                            TokenType::Identifier(_) | TokenType::Number(_) | TokenType::String(_)
                        );

                        if is_return_expr {
                            // Peek ahead to see if there's an 'end' or dedent after this
                            let expr = self.parse_expression()?;
                            self.skip_whitespace();

                            if let Some(next_token) = &self.current {
                                if matches!(
                                    next_token.token_type,
                                    TokenType::End | TokenType::Dedent
                                ) {
                                    return_value = Some(expr);
                                    continue;
                                } else if next_token.token_type == TokenType::Period {
                                    // It's a statement ending with period
                                    self.advance();
                                    body.push(Statement::ExprStmt(expr));
                                    continue;
                                }
                            }
                            return_value = Some(expr);
                        } else {
                            body.push(self.parse_statement()?);
                        }
                    }
                }
            } else {
                break;
            }
        }

        Ok(Statement::Recipe {
            name,
            params,
            body,
            return_value,
        })
    }

    fn parse_params(&mut self) -> ParseResult<Vec<String>> {
        self.expect(TokenType::LeftParen)?;
        let mut params = Vec::new();

        if let Some(token) = &self.current {
            if token.token_type == TokenType::RightParen {
                self.advance();
                return Ok(params);
            }
        }

        loop {
            let token = self.current_token()?.clone();
            match &token.token_type {
                TokenType::Identifier(s) => {
                    params.push(s.clone());
                    self.advance();
                }
                _ => {
                    return Err(ParseError::new(
                        "Expected parameter name".to_string(),
                        &token,
                    ));
                }
            }

            if let Some(token) = &self.current {
                if token.token_type == TokenType::Comma {
                    self.advance();
                    continue;
                } else if token.token_type == TokenType::RightParen {
                    self.advance();
                    break;
                }
            }
        }

        Ok(params)
    }

    // judge if lager is stronger than 5.0%: ... else: ... end.
    fn parse_judge(&mut self) -> ParseResult<Statement> {
        // Skip 'judge' or 'if'
        self.advance();

        // Optional 'if'
        if let Some(token) = &self.current {
            if token.token_type == TokenType::If {
                self.advance();
            }
        }

        let condition = self.parse_condition()?;
        self.expect(TokenType::Colon)?;
        self.skip_whitespace();

        // Parse then block
        let then_block = self.parse_block()?;

        // Check for else
        let else_block = if let Some(token) = &self.current {
            if token.token_type == TokenType::Else {
                self.advance();
                self.expect(TokenType::Colon)?;
                self.skip_whitespace();
                Some(self.parse_block()?)
            } else {
                None
            }
        } else {
            None
        };

        // Expect 'end.'
        self.expect(TokenType::End)?;
        self.expect(TokenType::Period)?;

        Ok(Statement::Judge {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_condition(&mut self) -> ParseResult<Condition> {
        let token = self.current_token()?.clone();
        let name = match &token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected identifier in condition".to_string(),
                    &token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::Is)?;

        // Check for 'not'
        let negated = if let Some(token) = &self.current {
            if token.token_type == TokenType::Not {
                self.advance();
                true
            } else {
                false
            }
        } else {
            false
        };

        let token = self.current_token()?.clone();
        match &token.token_type {
            TokenType::Stronger => {
                self.advance();
                self.expect(TokenType::Than)?;
                let threshold = self.parse_number_or_percentage()?;
                Ok(if negated {
                    Condition::NotStrongerThan {
                        brew_name: name,
                        threshold,
                    }
                } else {
                    Condition::StrongerThan {
                        brew_name: name,
                        threshold,
                    }
                })
            }
            TokenType::Weaker => {
                self.advance();
                self.expect(TokenType::Than)?;
                let threshold = self.parse_number_or_percentage()?;
                Ok(if negated {
                    Condition::NotWeakerThan {
                        brew_name: name,
                        threshold,
                    }
                } else {
                    Condition::WeakerThan {
                        brew_name: name,
                        threshold,
                    }
                })
            }
            TokenType::Number(n) => {
                let threshold = *n;
                self.advance();
                if threshold == 0.0 {
                    Ok(if negated {
                        Condition::IsNotZero { name }
                    } else {
                        Condition::IsZero { name }
                    })
                } else {
                    Ok(if negated {
                        Condition::NotEquals {
                            brew_name: name,
                            threshold,
                        }
                    } else {
                        Condition::Equals {
                            brew_name: name,
                            threshold,
                        }
                    })
                }
            }
            TokenType::Percentage(n) => {
                let threshold = *n;
                self.advance();
                Ok(if negated {
                    Condition::NotEquals {
                        brew_name: name,
                        threshold,
                    }
                } else {
                    Condition::Equals {
                        brew_name: name,
                        threshold,
                    }
                })
            }
            _ => Err(ParseError::new(
                "Expected 'stronger' or 'weaker'".to_string(),
                &token,
            )),
        }
    }

    fn parse_number_or_percentage(&mut self) -> ParseResult<f64> {
        let token = self.current_token()?.clone();
        let value = match token.token_type {
            TokenType::Number(n) => n,
            TokenType::Percentage(p) => p,
            _ => {
                return Err(ParseError::new(
                    "Expected number or percentage".to_string(),
                    &token,
                ));
            }
        };
        self.advance();
        Ok(value)
    }

    // repeat until/times ... end.
    fn parse_repeat(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Repeat)?;

        let token = self.current_token()?.clone();
        let condition = match token.token_type {
            TokenType::Until => {
                self.advance();
                let cond = self.parse_condition()?;
                LoopCondition::Until(Box::new(cond))
            }
            TokenType::Number(_) => {
                let times = self.parse_expression()?;
                self.expect(TokenType::Times)?;
                LoopCondition::Times(times)
            }
            TokenType::Identifier(_) => {
                // Could be "repeat n times" or "repeat each brew in barrel"
                let first_ident = if let TokenType::Identifier(s) = &token.token_type {
                    s.clone()
                } else {
                    unreachable!()
                };
                self.advance();

                // Check what comes next
                let next_token = self.current_token()?.clone();
                match next_token.token_type {
                    TokenType::Times => {
                        // "repeat n times"
                        self.advance();
                        LoopCondition::Times(Expression::Identifier(first_ident))
                    }
                    TokenType::In => {
                        // "repeat brew in barrel" (treating first ident as "each")
                        // Actually this should be "repeat each brew in barrel"
                        // Let's check if first_ident is "each"
                        if first_ident != "each" {
                            return Err(ParseError::new(
                                "Expected 'each' for collection iteration".to_string(),
                                &next_token,
                            ));
                        }
                        self.advance(); // skip 'in'

                        // Get variable name (the one after "each")
                        // Wait, we already consumed it. We need to get the collection now.
                        let collection_token = self.current_token()?.clone();
                        let _collection = match &collection_token.token_type {
                            TokenType::Identifier(s) => s.clone(),
                            _ => {
                                return Err(ParseError::new(
                                    "Expected collection name".to_string(),
                                    &collection_token,
                                ))
                            }
                        };
                        self.advance();

                        // This doesn't work - we need better lookahead
                        // Let's require explicit "repeat each <var> in <collection>"
                        return Err(ParseError::new(
                            "For iteration, use: repeat each <var> in <collection>:".to_string(),
                            &next_token,
                        ));
                    }
                    _ => {
                        return Err(ParseError::new(
                            "Expected 'times' or use 'repeat each <var> in <collection>:' for iteration".to_string(),
                            &next_token,
                        ))
                    }
                }
            }
            TokenType::Each => {
                // "repeat each brew in barrel:"
                self.advance(); // skip 'each'

                let var_token = self.current_token()?.clone();
                let var_name = match &var_token.token_type {
                    TokenType::Identifier(s) => s.clone(),
                    _ => {
                        return Err(ParseError::new(
                            "Expected variable name after 'each'".to_string(),
                            &var_token,
                        ));
                    }
                };
                self.advance();

                self.expect(TokenType::In)?;

                let collection_token = self.current_token()?.clone();
                let collection = match &collection_token.token_type {
                    TokenType::Identifier(s) => s.clone(),
                    _ => {
                        return Err(ParseError::new(
                            "Expected collection name".to_string(),
                            &collection_token,
                        ));
                    }
                };
                self.advance();

                LoopCondition::ForEach {
                    var_name,
                    collection,
                }
            }
            _ => {
                return Err(ParseError::new(
                    "Expected 'until', 'each', number, or identifier after 'repeat'".to_string(),
                    &token,
                ));
            }
        };

        self.expect(TokenType::Colon)?;
        self.skip_whitespace();

        let body = self.parse_block()?;

        // Expect 'end.'
        self.expect(TokenType::End)?;
        self.expect(TokenType::Period)?;

        Ok(Statement::Repeat { condition, body })
    }

    fn parse_block(&mut self) -> ParseResult<Vec<Statement>> {
        let mut statements = Vec::new();
        let mut indent_count = 0;

        // Check for indent
        if let Some(token) = &self.current {
            if token.token_type == TokenType::Indent {
                indent_count += 1;
                self.advance();
                self.skip_whitespace();
            }
        }

        loop {
            self.skip_whitespace();
            if let Some(token) = &self.current {
                match token.token_type {
                    TokenType::End | TokenType::Else => break,
                    TokenType::Dedent => {
                        indent_count -= 1;
                        self.advance();
                        if indent_count == 0 {
                            break;
                        }
                    }
                    _ => {
                        statements.push(self.parse_statement()?);
                    }
                }
            } else {
                break;
            }
        }

        Ok(statements)
    }

    // barrel taplist = [lager, stout].
    fn parse_barrel_decl(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Barrel)?;
        let name_token = self.current_token()?.clone();
        let name = match &name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected barrel name".to_string(),
                    &name_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::Equal)?;
        self.expect(TokenType::LeftBracket)?;

        let mut elements = Vec::new();
        if let Some(token) = &self.current {
            if token.token_type != TokenType::RightBracket {
                loop {
                    elements.push(self.parse_expression()?);
                    if let Some(token) = &self.current {
                        if token.token_type == TokenType::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        self.expect(TokenType::RightBracket)?;
        self.expect(TokenType::Period)?;

        Ok(Statement::BarrelDecl { name, elements })
    }

    // add lager to taplist.
    fn parse_add_to_barrel(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Add)?;
        let brew_name_token = self.current_token()?.clone();
        let brew_name = match &brew_name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected brew name".to_string(),
                    &brew_name_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::To)?;

        let barrel_name_token = self.current_token()?.clone();
        let barrel_name = match &barrel_name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected barrel name".to_string(),
                    &barrel_name_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::Period)?;

        Ok(Statement::AddToBarrel {
            brew_name,
            barrel_name,
        })
    }

    // remove lager from taplist.
    fn parse_remove_from_barrel(&mut self) -> ParseResult<Statement> {
        self.expect(TokenType::Remove)?;
        let brew_name_token = self.current_token()?.clone();
        let brew_name = match &brew_name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected brew name".to_string(),
                    &brew_name_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::From)?;

        let barrel_name_token = self.current_token()?.clone();
        let barrel_name = match &barrel_name_token.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => {
                return Err(ParseError::new(
                    "Expected barrel name".to_string(),
                    &barrel_name_token,
                ));
            }
        };
        self.advance();

        self.expect(TokenType::Period)?;

        Ok(Statement::RemoveFromBarrel {
            brew_name,
            barrel_name,
        })
    }

    fn parse_expression(&mut self) -> ParseResult<Expression> {
        let token = self.current_token()?.clone();

        match &token.token_type {
            TokenType::Number(n) => {
                let val = *n;
                self.advance();
                Ok(Expression::Number(val))
            }
            TokenType::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expression::String(val))
            }
            TokenType::Identifier(s) => {
                let name = s.clone();
                self.advance();

                // Check for function call
                if let Some(token) = &self.current {
                    if token.token_type == TokenType::LeftParen {
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect(TokenType::RightParen)?;
                        return Ok(Expression::FunctionCall { name, args });
                    } else if token.token_type == TokenType::Position {
                        // Barrel access: taplist position 1
                        self.advance();
                        let index = Box::new(self.parse_expression()?);
                        return Ok(Expression::BarrelAccess {
                            barrel_name: name,
                            index,
                        });
                    }
                }

                Ok(Expression::Identifier(name))
            }
            _ => Err(ParseError::new(
                format!("Unexpected token in expression: {:?}", token.token_type),
                &token,
            )),
        }
    }

    fn parse_args(&mut self) -> ParseResult<Vec<Expression>> {
        let mut args = Vec::new();

        if let Some(token) = &self.current {
            if token.token_type == TokenType::RightParen {
                return Ok(args);
            }
        }

        loop {
            args.push(self.parse_expression()?);
            if let Some(token) = &self.current {
                if token.token_type == TokenType::Comma {
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(args)
    }
}
