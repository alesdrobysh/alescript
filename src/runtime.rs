use std::collections::HashMap;

/// ABV (Alcohol By Volume) value constrained to 0-100%
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Abv(f64);

impl Abv {
    pub fn new(value: f64) -> Self {
        Abv(value.clamp(0.0, 100.0))
    }

    pub fn zero() -> Self {
        Abv(0.0)
    }

    pub fn get(&self) -> f64 {
        self.0
    }

    pub fn add(self, other: Abv) -> Self {
        Abv((self.0 + other.0).clamp(0.0, 100.0))
    }

    pub fn multiply(self, factor: f64) -> Self {
        Abv((self.0 * factor).clamp(0.0, 100.0))
    }

    pub fn divide(self, factor: f64) -> Self {
        Abv((self.0 / factor).clamp(0.0, 100.0))
    }
}

/// Represents the state of a single brew
#[derive(Debug, Clone)]
pub struct BrewState {
    pub current_abv: Abv,
    pub growth_rate: f64,
    pub is_kegged: bool,
    pub last_update_time: f64,
}

impl BrewState {
    pub fn new(growth_rate: f64, current_time: f64) -> Self {
        BrewState {
            current_abv: Abv::zero(),
            growth_rate,
            is_kegged: false,
            last_update_time: current_time,
        }
    }

    pub fn update_to_time(&mut self, current_time: f64) {
        if !self.is_kegged {
            let time_delta = current_time - self.last_update_time;
            let increase = Abv::new(self.growth_rate * time_delta);
            self.current_abv = self.current_abv.add(increase);
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
}

/// Runtime environment managing all state
pub struct RuntimeEnvironment {
    pub global_time: f64,
    pub brews: HashMap<String, BrewState>,
    pub variables: HashMap<String, Value>,
    pub barrels: HashMap<String, Vec<Value>>,
    rng_state: u64,
}

impl RuntimeEnvironment {
    pub fn new() -> Self {
        RuntimeEnvironment {
            global_time: 0.0,
            brews: HashMap::new(),
            variables: HashMap::new(),
            barrels: HashMap::new(),
            rng_state: 12345,
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
            Ok(brew.current_abv.get())
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
            if brew.current_abv.get() >= target_abv {
                return Ok(());
            }

            if brew.growth_rate <= 0.0 {
                return Err(format!(
                    "Cannot age brew '{}' with zero or negative growth rate",
                    name
                ));
            }

            // Calculate time needed
            let time_needed = (target_abv - brew.current_abv.get()) / brew.growth_rate;

            // Advance time for all brews
            self.advance_time(time_needed)?;
            Ok(())
        } else {
            Err(format!("Brew '{}' not found", name))
        }
    }

    /// Blend two brews (add second's ABV to first)
    pub fn blend_brews(&mut self, target: &str, source: &str) -> Result<(), String> {
        // Update both brews to current time first
        let target_abv = self.get_brew_abv(target)?;
        let source_abv = self.get_brew_abv(source)?;

        if let Some(target_brew) = self.brews.get_mut(target) {
            target_brew.current_abv = Abv::new(target_abv).add(Abv::new(source_abv));
            Ok(())
        } else {
            Err(format!("Brew '{}' not found", target))
        }
    }

    /// Multiply a brew's ABV by a factor (advances time)
    /// Note: In alescript, "fortify by N" means multiply by N, not multiply by 2.
    /// For example, "fortify porter by 3" multiplies the ABV by 3.
    /// This operation advances time based on the brew's growth rate.
    pub fn fortify_brew(&mut self, name: &str, factor: f64) -> Result<(), String> {
        let current_abv = self.get_brew_abv(name)?;

        if let Some(brew) = self.brews.get_mut(name) {
            let target_abv = Abv::new(current_abv).multiply(factor);
            let time_needed = if brew.growth_rate > 0.0 {
                (target_abv.get() - current_abv) / brew.growth_rate
            } else {
                // If no growth rate, we can't fortify by waiting
                0.0
            };

            // Advance time
            if time_needed > 0.0 {
                self.advance_time(time_needed)?;
            } else {
                brew.current_abv = target_abv;
            }
            Ok(())
        } else {
            Err(format!("Brew '{}' not found", name))
        }
    }

    /// Filter a brew's ABV by a factor
    pub fn filter_brew(&mut self, name: &str, factor: f64) -> Result<(), String> {
        let current_abv = self.get_brew_abv(name)?;

        if let Some(brew) = self.brews.get_mut(name) {
            brew.current_abv = Abv::new(current_abv).divide(factor);
            Ok(())
        } else {
            Err(format!("Brew '{}' not found", name))
        }
    }

    /// Fuzzy comparison with ±10% imprecision
    pub fn fuzzy_compare(&mut self, value: f64, threshold: f64, stronger: bool) -> bool {
        let imprecision = value.abs() * 0.1;

        // Simple xorshift PRNG (works on all platforms including WASM)
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        let random = (self.rng_state % 10000) as f64 / 10000.0; // 0.0 to 1.0
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
