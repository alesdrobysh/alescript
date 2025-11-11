use crate::ast::*;
use crate::runtime::{RuntimeEnvironment, Value};
use std::collections::HashMap;

/// Interpreter for alescript
pub struct Interpreter {
    runtime: RuntimeEnvironment,
    recipes: HashMap<String, (Vec<String>, Vec<Statement>, Option<Expression>)>,
    scopes: Vec<HashMap<String, Value>>, // Scope stack for local variables
    output: String, // Output buffer for capturing print statements
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    fn new(message: String) -> Self {
        RuntimeError { message }
    }
}

type RuntimeResult<T> = Result<T, RuntimeError>;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            runtime: RuntimeEnvironment::new(),
            recipes: HashMap::new(),
            scopes: vec![HashMap::new()], // Start with global scope
            output: String::new(),
        }
    }

    pub fn get_output(&self) -> String {
        self.output.clone()
    }

    fn print(&mut self, text: String) {
        self.output.push_str(&text);
        self.output.push('\n');
    }

    pub fn execute(&mut self, program: &Program) -> RuntimeResult<()> {
        for statement in &program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement) -> RuntimeResult<Value> {
        match statement {
            Statement::Brew { name, ingredients } => {
                let growth_rate = self.calculate_growth_rate(ingredients);
                self.runtime.create_brew(name.clone(), growth_rate);
                Ok(Value::Void)
            }

            Statement::Wait { days } => {
                let days_value = self.evaluate_expression(days)?;
                let days_num = days_value
                    .as_number()
                    .map_err(|e| RuntimeError::new(e))?;
                self.runtime
                    .advance_time(days_num)
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(Value::Void)
            }

            Statement::Age {
                brew_name,
                target_abv,
            } => {
                self.runtime
                    .age_until(brew_name, *target_abv)
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(Value::Void)
            }

            Statement::Taste { brew_name } => {
                // Check if brew_name is a variable first
                let actual_brew_name = if let Some(value) = self.get_local_variable(brew_name) {
                    match value {
                        Value::BrewRef(name) => name,
                        _ => brew_name.clone(),
                    }
                } else {
                    brew_name.clone()
                };

                let abv = self
                    .runtime
                    .get_brew_abv(&actual_brew_name)
                    .map_err(|e| RuntimeError::new(e))?;
                self.print(format!("{}% ABV", abv));
                Ok(Value::Void)
            }

            Statement::Toast { value } => {
                let result = self.evaluate_expression(value)?;
                match result {
                    Value::String(s) => self.print(s),
                    Value::Number(n) => self.print(format!("{}% ABV", n)),
                    Value::BrewRef(name) => {
                        let abv = self
                            .runtime
                            .get_brew_abv(&name)
                            .map_err(|e| RuntimeError::new(e))?;
                        self.print(format!("{}% ABV", abv));
                    }
                    Value::Void => self.print("void".to_string()),
                }
                Ok(Value::Void)
            }

            Statement::Keg { brew_name } => {
                self.runtime
                    .keg_brew(brew_name)
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(Value::Void)
            }

            Statement::Mix { target, source } => {
                self.runtime
                    .mix_brews(target, source)
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(Value::Void)
            }

            Statement::Double { brew_name, factor } => {
                let factor_value = self.evaluate_expression(factor)?;
                let factor_num = factor_value
                    .as_number()
                    .map_err(|e| RuntimeError::new(e))?;
                self.runtime
                    .double_brew(brew_name, factor_num)
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(Value::Void)
            }

            Statement::Dilute { brew_name, factor } => {
                let factor_value = self.evaluate_expression(factor)?;
                let factor_num = factor_value
                    .as_number()
                    .map_err(|e| RuntimeError::new(e))?;
                self.runtime
                    .dilute_brew(brew_name, factor_num)
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(Value::Void)
            }

            Statement::Recipe {
                name,
                params,
                body,
                return_value,
            } => {
                self.recipes
                    .insert(name.clone(), (params.clone(), body.clone(), return_value.clone()));
                Ok(Value::Void)
            }

            Statement::Relabel { from, to } => {
                // "relabel X as Y" means Y = X (copy X's value to Y)
                self.runtime
                    .copy_brew(from, to.clone())
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(Value::Void)
            }

            Statement::Judge {
                condition,
                then_block,
                else_block,
            } => {
                if self.evaluate_condition(condition)? {
                    for stmt in then_block {
                        self.execute_statement(stmt)?;
                    }
                } else if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.execute_statement(stmt)?;
                    }
                }
                Ok(Value::Void)
            }

            Statement::Repeat { condition, body } => {
                match condition {
                    LoopCondition::Until(cond) => {
                        while !self.evaluate_condition(cond)? {
                            for stmt in body {
                                self.execute_statement(stmt)?;
                            }
                        }
                    }
                    LoopCondition::Times(expr) => {
                        let times_value = self.evaluate_expression(expr)?;
                        let times = times_value
                            .as_number()
                            .map_err(|e| RuntimeError::new(e))? as i32;
                        for _ in 0..times {
                            for stmt in body {
                                self.execute_statement(stmt)?;
                            }
                        }
                    }
                    LoopCondition::ForEach {
                        var_name,
                        collection,
                    } => {
                        let barrel = self
                            .runtime
                            .get_barrel(collection)
                            .map_err(|e| RuntimeError::new(e))?
                            .clone();
                        for value in barrel {
                            self.set_local_variable(var_name.clone(), value);
                            for stmt in body {
                                self.execute_statement(stmt)?;
                            }
                        }
                    }
                }
                Ok(Value::Void)
            }

            Statement::BarrelDecl { name, elements } => {
                let mut values = Vec::new();
                for expr in elements {
                    values.push(self.evaluate_expression(expr)?);
                }
                self.runtime.set_barrel(name.clone(), values);
                Ok(Value::Void)
            }

            Statement::AddToBarrel {
                brew_name,
                barrel_name,
            } => {
                let value = Value::BrewRef(brew_name.clone());
                let barrel = self
                    .runtime
                    .get_barrel_mut(barrel_name)
                    .map_err(|e| RuntimeError::new(e))?;
                barrel.push(value);
                Ok(Value::Void)
            }

            Statement::RemoveFromBarrel {
                brew_name,
                barrel_name,
            } => {
                let barrel = self
                    .runtime
                    .get_barrel_mut(barrel_name)
                    .map_err(|e| RuntimeError::new(e))?;
                barrel.retain(|v| match v {
                    Value::BrewRef(name) => name != brew_name,
                    _ => true,
                });
                Ok(Value::Void)
            }

            Statement::ExprStmt(expr) => self.evaluate_expression(expr),

            Statement::End => Ok(Value::Void),
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> RuntimeResult<Value> {
        match expr {
            Expression::Number(n) => Ok(Value::Number(*n)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Identifier(name) => {
                // Check local variables first
                if let Some(value) = self.get_local_variable(name) {
                    return Ok(value);
                }
                // Check runtime variables
                if let Ok(value) = self.runtime.get_variable(name) {
                    return Ok(value);
                }
                // Check if it's a brew reference
                if self.runtime.brews.contains_key(name) {
                    Ok(Value::BrewRef(name.clone()))
                } else {
                    // Assume it's a variable with value 0
                    Ok(Value::Number(0.0))
                }
            }
            Expression::FunctionCall { name, args } => self.call_function(name, args),
            Expression::BarrelAccess { barrel_name, index } => {
                let index_value = self.evaluate_expression(index)?;
                let index_num = index_value
                    .as_number()
                    .map_err(|e| RuntimeError::new(e))? as usize;
                let barrel = self
                    .runtime
                    .get_barrel(barrel_name)
                    .map_err(|e| RuntimeError::new(e))?;
                // alescript uses 1-based indexing
                if index_num < 1 || index_num > barrel.len() {
                    return Err(RuntimeError::new(format!(
                        "Index {} out of bounds for barrel '{}'",
                        index_num, barrel_name
                    )));
                }
                Ok(barrel[index_num - 1].clone())
            }
        }
    }

    fn call_function(&mut self, name: &str, args: &[Expression]) -> RuntimeResult<Value> {
        if let Some((params, body, return_value)) = self.recipes.get(name).cloned() {
            // Create new scope
            self.scopes.push(HashMap::new());

            // Bind parameters
            if params.len() != args.len() {
                return Err(RuntimeError::new(format!(
                    "Function '{}' expects {} arguments, got {}",
                    name,
                    params.len(),
                    args.len()
                )));
            }

            for (param, arg) in params.iter().zip(args.iter()) {
                let value = self.evaluate_expression(arg)?;
                self.set_local_variable(param.clone(), value);
            }

            // Execute body
            for stmt in &body {
                self.execute_statement(stmt)?;
            }

            // Evaluate return value
            let result = if let Some(ret_expr) = return_value {
                self.evaluate_expression(&ret_expr)?
            } else {
                Value::Void
            };

            // Pop scope
            self.scopes.pop();

            Ok(result)
        } else {
            Err(RuntimeError::new(format!("Unknown function '{}'", name)))
        }
    }

    fn evaluate_condition(&mut self, condition: &Condition) -> RuntimeResult<bool> {
        match condition {
            Condition::StrongerThan {
                brew_name,
                threshold,
            } => {
                let abv = self
                    .runtime
                    .get_brew_abv(brew_name)
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(self.runtime.fuzzy_compare(abv, *threshold, true))
            }
            Condition::WeakerThan {
                brew_name,
                threshold,
            } => {
                let abv = self
                    .runtime
                    .get_brew_abv(brew_name)
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(self.runtime.fuzzy_compare(abv, *threshold, false))
            }
            Condition::NotWeakerThan {
                brew_name,
                threshold,
            } => {
                let abv = self
                    .runtime
                    .get_brew_abv(brew_name)
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(!self.runtime.fuzzy_compare(abv, *threshold, false))
            }
            Condition::NotStrongerThan {
                brew_name,
                threshold,
            } => {
                let abv = self
                    .runtime
                    .get_brew_abv(brew_name)
                    .map_err(|e| RuntimeError::new(e))?;
                Ok(!self.runtime.fuzzy_compare(abv, *threshold, true))
            }
            Condition::IsZero { name } => {
                let value = self.evaluate_expression(&Expression::Identifier(name.clone()))?;
                let num = value.as_number().map_err(|e| RuntimeError::new(e))?;
                Ok(num == 0.0)
            }
            Condition::IsNotZero { name } => {
                let value = self.evaluate_expression(&Expression::Identifier(name.clone()))?;
                let num = value.as_number().map_err(|e| RuntimeError::new(e))?;
                Ok(num != 0.0)
            }
        }
    }

    fn calculate_growth_rate(&self, ingredients: &[Ingredient]) -> f64 {
        ingredients
            .iter()
            .map(|ing| ing.name.growth_rate() * ing.quantity)
            .sum()
    }

    fn set_local_variable(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    fn get_local_variable(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }
}
