use std::path::Path;

use crate::errors::ConfigError;

macro_rules! config {
    ($($field_name:ident: $field_ty:ty, $default_value:expr, $description:expr );+ $(;)*) => (
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
    error_on_lost_comment: bool, false, "Error if unable to get comments";
    short_array_element_width_threshold: usize, 10, "Width threshold for an array element to be considered short";
    array_width: usize, 100, "Maximum width of an array literal before falling back to vertical formatting";
    fn_call_width: usize, 60, "Maximum width of the args of a function call before falling back to vertical formatting";
    single_line_if_else_max_width: usize, 50, "Maximum line length for single line if-else expressions";
}

impl Config {
    pub fn read(path: &Path) -> Result<Self, ConfigError> {
        let config_path = path.join("noirfmt.toml");

        let input = match std::fs::read_to_string(&config_path) {
            Ok(input) => input,
            Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(cause) => return Err(ConfigError::ReadFailed(config_path, cause)),
        };

        Self::of(&input)
    }

    pub fn of(s: &str) -> Result<Self, ConfigError> {
        let mut config = Self::default();
        let toml = toml::from_str(s).map_err(ConfigError::MalformedFile)?;
        config.fill_from_toml(toml);
        Ok(config)
    }
}
