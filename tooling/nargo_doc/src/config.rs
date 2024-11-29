use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::errors::ConfigError;

pub struct TomlConfig {
    pub no_js: Option<bool>,
}

pub struct Config {
    /// Generate no Javascript (disables MathJax)
    pub no_js: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { no_js: false }
    }
}

impl Config {
    pub fn fill_from_toml(&mut self, toml: TomlConfig) {
        Self { no_js: toml.no_js.unwrap_or(self.no_js) }
    }

    fn new(no_js: Option<bool>) -> Result<Self, ConfigError> {
        Ok(Self { no_js: no_js.unwrap_or(false) })
    }
}
