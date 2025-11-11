use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents the state of a single brew
#[derive(Debug, Clone)]
pub struct BrewState {
    pub current_abv: f64,
    pub growth_rate: f64,
    pub is_kegged: bool,
    pub last_update_time: f64,
}

impl BrewState {
    pub fn new(growth_rate: f64, current_time: f64) -> Self {
        BrewState {
            current_abv: 0.0,
            growth_rate,
            is_kegged: false,
            last_update_time: current_time,
        }
    }

    pub fn update_to_time(&mut self, current_time: f64) {
        if !self.is_kegged {
            let time_delta = current_time - self.last_update_time;
            self.current_abv += self.growth_rate * time_delta;
            self.last_update_time = current_time;
        }
    }
}

/// Runtime value type
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    BrewRef(String), // Reference to a brew by name
    Void,
}

impl Value {
    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => Err(format!("Expected number, got {:?}", self)),
        }
    }

    pub fn as_string(&self) -> Result<String, String> {
        match self {
            Value::String(s) => Ok(s.clone()),
            _ => Err(format!("Expected string, got {:?}", self)),
        }
    }
}

/// Runtime environment managing all state
pub struct RuntimeEnvironment {
    pub global_time: f64,
    pub brews: HashMap<String, BrewState>,
    pub variables: HashMap<String, Value>,
    pub barrels: HashMap<String, Vec<Value>>,
}

impl RuntimeEnvironment {
    pub fn new() -> Self {
        RuntimeEnvironment {
            global_time: 0.0,
            brews: HashMap::new(),
            variables: HashMap::new(),
            barrels: HashMap::new(),
        }
    }

    /// Advance the global timeline and update all brews
    /// Returns an error if days is negative (time cannot go backward)
    pub fn advance_time(&mut self, days: f64) -> Result<(), String> {
        if days < 0.0 {
            return Err(format!(
                "Cannot advance time by negative amount: {} days",
                days
            ));
        }

        self.global_time += days;
        for brew in self.brews.values_mut() {
            brew.update_to_time(self.global_time);
        }
        Ok(())
    }

    /// Create a new brew with given growth rate
    pub fn create_brew(&mut self, name: String, growth_rate: f64) {
        let brew = BrewState::new(growth_rate, self.global_time);
        self.brews.insert(name, brew);
    }

    /// Get the current ABV of a brew
    pub fn get_brew_abv(&mut self, name: &str) -> Result<f64, String> {
        if let Some(brew) = self.brews.get_mut(name) {
            brew.update_to_time(self.global_time);
            Ok(brew.current_abv)
        } else {
            Err(format!("Brew '{}' not found", name))
        }
    }

    /// Keg a brew (stop its growth)
    pub fn keg_brew(&mut self, name: &str) -> Result<(), String> {
        if let Some(brew) = self.brews.get_mut(name) {
            brew.update_to_time(self.global_time);
            brew.is_kegged = true;
            Ok(())
        } else {
            Err(format!("Brew '{}' not found", name))
        }
    }

    /// Age a brew until it reaches target ABV
    /// If the brew is already at or above the target, this is a no-op.
    pub fn age_until(&mut self, name: &str, target_abv: f64) -> Result<(), String> {
        if let Some(brew) = self.brews.get_mut(name) {
            brew.update_to_time(self.global_time);

            if brew.is_kegged {
                return Err(format!("Cannot age kegged brew '{}'", name));
            }

            // If already at or above target, nothing to do
            if brew.current_abv >= target_abv {
                return Ok(());
            }

            if brew.growth_rate <= 0.0 {
                return Err(format!(
                    "Cannot age brew '{}' with zero or negative growth rate",
                    name
                ));
            }

            // Calculate time needed
            let time_needed = (target_abv - brew.current_abv) / brew.growth_rate;

            // Advance time for all brews
            self.advance_time(time_needed)?;
            Ok(())
        } else {
            Err(format!("Brew '{}' not found", name))
        }
    }

    /// Mix two brews (add second's ABV to first)
    pub fn mix_brews(&mut self, target: &str, source: &str) -> Result<(), String> {
        // Update both brews to current time first
        let target_abv = self.get_brew_abv(target)?;
        let source_abv = self.get_brew_abv(source)?;

        if let Some(target_brew) = self.brews.get_mut(target) {
            target_brew.current_abv = target_abv + source_abv;
            Ok(())
        } else {
            Err(format!("Brew '{}' not found", target))
        }
    }

    /// Multiply a brew's ABV by a factor (advances time)
    /// Note: In alescript, "double by N" means multiply by N, not multiply by 2.
    /// For example, "double porter by 3" multiplies the ABV by 3.
    /// This operation advances time based on the brew's growth rate.
    pub fn double_brew(&mut self, name: &str, factor: f64) -> Result<(), String> {
        let current_abv = self.get_brew_abv(name)?;

        if let Some(brew) = self.brews.get_mut(name) {
            let target_abv = current_abv * factor;
            let time_needed = if brew.growth_rate > 0.0 {
                (target_abv - current_abv) / brew.growth_rate
            } else {
                // If no growth rate, we can't double by waiting
                0.0
            };

            // Advance time
            if time_needed > 0.0 {
                self.advance_time(time_needed)?;
            } else {
                // Just set the value directly
                brew.current_abv = target_abv;
            }
            Ok(())
        } else {
            Err(format!("Brew '{}' not found", name))
        }
    }

    /// Dilute a brew's ABV by a factor
    pub fn dilute_brew(&mut self, name: &str, factor: f64) -> Result<(), String> {
        let current_abv = self.get_brew_abv(name)?;

        if let Some(brew) = self.brews.get_mut(name) {
            brew.current_abv = current_abv / factor;
            Ok(())
        } else {
            Err(format!("Brew '{}' not found", name))
        }
    }

    /// Fuzzy comparison with ±10% imprecision
    pub fn fuzzy_compare(&self, value: f64, threshold: f64, stronger: bool) -> bool {
        let imprecision = value.abs() * 0.1;

        // Simple PRNG using system time
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random = ((nanos % 10000) as f64) / 10000.0; // 0.0 to 1.0
        let random_offset = (random - 0.5) * 2.0 * imprecision; // -imprecision to +imprecision

        let fuzzy_value = value + random_offset;

        if stronger {
            fuzzy_value > threshold
        } else {
            fuzzy_value < threshold
        }
    }

    /// Set a variable
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Get a variable
    pub fn get_variable(&self, name: &str) -> Result<Value, String> {
        self.variables
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Variable '{}' not found", name))
    }

    /// Create or update a barrel
    pub fn set_barrel(&mut self, name: String, elements: Vec<Value>) {
        self.barrels.insert(name, elements);
    }

    /// Get a barrel
    pub fn get_barrel(&self, name: &str) -> Result<&Vec<Value>, String> {
        self.barrels
            .get(name)
            .ok_or_else(|| format!("Barrel '{}' not found", name))
    }

    /// Get a mutable barrel
    pub fn get_barrel_mut(&mut self, name: &str) -> Result<&mut Vec<Value>, String> {
        self.barrels
            .get_mut(name)
            .ok_or_else(|| format!("Barrel '{}' not found", name))
    }

    /// Copy a brew (for relabel)
    pub fn copy_brew(&mut self, from: &str, to: String) -> Result<(), String> {
        if let Some(brew) = self.brews.get(from).cloned() {
            self.brews.insert(to, brew);
            Ok(())
        } else {
            // Try copying a variable
            if let Ok(value) = self.get_variable(from) {
                self.set_variable(to, value);
                Ok(())
            } else {
                Err(format!("Cannot copy '{}': not found", from))
            }
        }
    }
}
