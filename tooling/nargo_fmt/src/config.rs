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
            $(pub $field_name: Option<$field_ty>),+
        }
    )
}

config! {
    tab_spaces: usize, 4, "Number of spaces per tab";
}

impl Config {
    pub fn read(path: &Path) -> Result<Self, ConfigError> {
        let mut config = Self::default();
        let config_path = path.join("noirfmt.toml");

        let raw_toml = match std::fs::read_to_string(&config_path) {
            Ok(t) => t,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(cause) => return Err(ConfigError::ReadFailed(config_path, cause)),
        };
        let toml = toml::from_str(&raw_toml).map_err(ConfigError::MalformedFile)?;

        config.fill_from_toml(toml);
        Ok(config)
    }
}
