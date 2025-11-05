/// Abstract Syntax Tree definitions for alescript
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // Brew declaration: brew lager from water, 1 barley, 2 hops, 1 yeast.
    Brew {
        name: String,
        ingredients: Vec<Ingredient>,
    },

    // Wait statement: wait for 5 days.
    Wait { days: Expression },

    // Age statement: age lager until 5.2%.
    Age {
        brew_name: String,
        target_abv: f64,
    },

    // Taste statement: taste lager.
    Taste { brew_name: String },

    // Toast statement: toast "hello, world!".
    Toast { value: Expression },

    // Keg statement: keg lager.
    Keg { brew_name: String },

    // Mix statement: mix lager with stout.
    Mix {
        target: String,
        source: String,
    },

    // Double statement: double porter by 3.
    Double {
        brew_name: String,
        factor: Expression,
    },

    // Dilute statement: dilute ipa by 2.
    Dilute {
        brew_name: String,
        factor: Expression,
    },

    // Recipe declaration (function)
    Recipe {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        return_value: Option<Expression>,
    },

    // Relabel statement (assignment): relabel temp as a.
    Relabel { from: String, to: String },

    // Judge/If conditional
    Judge {
        condition: Condition,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },

    // Repeat loop
    Repeat {
        condition: LoopCondition,
        body: Vec<Statement>,
    },

    // Barrel declaration: barrel taplist = [lager, stout].
    BarrelDecl {
        name: String,
        elements: Vec<Expression>,
    },

    // Add to barrel: add lager to taplist.
    AddToBarrel {
        brew_name: String,
        barrel_name: String,
    },

    // Remove from barrel: remove lager from taplist.
    RemoveFromBarrel {
        brew_name: String,
        barrel_name: String,
    },

    // Expression statement (for function calls that don't need to be assigned)
    ExprStmt(Expression),

    // End marker (for parsing)
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Number literal
    Number(f64),

    // String literal
    String(String),

    // Variable reference (brew name or identifier)
    Identifier(String),

    // Function call
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },

    // Barrel access: taplist position 1.
    BarrelAccess {
        barrel_name: String,
        index: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ingredient {
    pub name: IngredientType,
    pub quantity: f64, // Default is 1
}

#[derive(Debug, Clone, PartialEq)]
pub enum IngredientType {
    Water,
    Barley,
    Hops,
    Yeast,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    // lager is stronger than 5.0%
    StrongerThan {
        brew_name: String,
        threshold: f64,
    },

    // lager is weaker than 5.0%
    WeakerThan {
        brew_name: String,
        threshold: f64,
    },

    // lager is not weaker than 5.0% (equivalent to stronger or equal)
    NotWeakerThan {
        brew_name: String,
        threshold: f64,
    },

    // lager is not stronger than 5.0% (equivalent to weaker or equal)
    NotStrongerThan {
        brew_name: String,
        threshold: f64,
    },

    // Simple identifier comparison: if n is 0
    IsZero { name: String },

    // Negation: if n is not 0
    IsNotZero { name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopCondition {
    // repeat until lager is not weaker than 5.0%
    Until(Box<Condition>),

    // repeat n times
    Times(Expression),

    // for brew in taplist
    ForEach {
        var_name: String,
        collection: String,
    },
}

impl IngredientType {
    pub fn growth_rate(&self) -> f64 {
        match self {
            IngredientType::Water => 0.0,
            IngredientType::Barley => 1.0,
            IngredientType::Hops => 0.5,
            IngredientType::Yeast => 1.5,
        }
    }
}
