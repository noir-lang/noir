use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::errors::ConfigError;

macro_rules! config {
    ($($field_name:ident: $field_ty:ty, $default_value:expr_2021, $description:expr_2021 );+ $(;)*) => (
        pub struct Config {
            $(
                #[doc = $description]
                pub $field_name: $field_ty
            ),+
        }

        impl Config {
            pub fn fill_from_toml(&mut self, toml: TomlConfig) {
                $(
                    if let Some(value) = toml.$field_name {
                       self.$field_name = value;
                    }
                )+
            }
        }

        impl Default for Config {
            fn default() -> Self {
                Self {
                    $(
                        $field_name: $default_value,
                    )+
                }
            }
        }

        #[derive(serde::Deserialize, serde::Serialize, Clone)]
        pub struct TomlConfig {
            $(
                #[doc = $description]
                pub $field_name: Option<$field_ty>
            ),+
        }
    )
}

config! {
    max_width: usize, 100, "Maximum width of each line";
    tab_spaces: usize, 4, "Number of spaces per tab";
    remove_nested_parens: bool, true, "Remove nested parens";
    short_array_element_width_threshold: usize, 10, "Width threshold for an array element to be considered short";
    array_width: usize, 100, "Maximum width of an array literal before falling back to vertical formatting";
    fn_call_width: usize, 60, "Maximum width of the args of a function call before falling back to vertical formatting";
    single_line_if_else_max_width: usize, 50, "Maximum line length for single line if-else expressions";
    imports_granularity: ImportsGranularity, ImportsGranularity::Preserve, "How imports should be grouped into use statements.";
    reorder_imports: bool, true, "Reorder imports alphabetically";
}

impl Config {
    /// Reads a Config starting at the given path and going through the path parents
    /// until a `noirfmt.toml` file is found in one of them or the root is reached.
    pub fn read(mut path: &Path) -> Result<Self, ConfigError> {
        loop {
            let config_path = path.join("noirfmt.toml");
            if config_path.exists() {
                match std::fs::read_to_string(&config_path) {
                    Ok(input) => return Self::of(&input, &config_path),
                    Err(cause) => return Err(ConfigError::ReadFailed(config_path, cause)),
                };
            }

            let Some(parent_path) = path.parent() else {
                return Ok(Config::default());
            };

            path = parent_path;
        }
    }

    pub fn of(s: &str, path: &Path) -> Result<Self, ConfigError> {
        let mut config = Self::default();
        let toml =
            toml::from_str(s).map_err(|err| ConfigError::MalformedFile(path.to_path_buf(), err))?;
        config.fill_from_toml(toml);
        Ok(config)
    }
}

/// How imports should be grouped into use statements.
/// Imports will be merged or split to the configured level of granularity.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
pub enum ImportsGranularity {
    /// Do not change the granularity of any imports and preserve the original structure written by the developer.
    Preserve,
    /// Merge imports from the same crate into a single use statement.
    Crate,
}
