use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::fmt;

#[cfg(doc)]
use crate::core::Target;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Profile(SmolStr);

impl Profile {
    pub const RELEASE: Self = Self(SmolStr::new_inline("release"));
    pub const DEV: Self = Self(SmolStr::new_inline("dev"));

    /// Create new `Profile` struct.
    /// Validates profile name to ensure it can be used as a valid subdirectory name.
    pub fn new(name: SmolStr) -> Result<Self> {
        ensure!(
            name.as_str() != "",
            "cannot use empty string as profile name"
        );
        ensure!(
            !["_", "package", "build", "debug", "doc", "test"].contains(&name.as_str()),
            format!("profile name `{name}` is not allowed")
        );
        ensure!(
            !name.to_string().starts_with(".."),
            format!("profile name cannot start with `..` prefix")
        );
        ensure!(
            name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
            format!("profile name `{name}` is not allowed, only alphanumeric characters and `-` can be used")
        );
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn is_dev(&self) -> bool {
        self.0.as_str() == "dev"
    }
    pub fn is_release(&self) -> bool {
        self.0.as_str() == "release"
    }
    pub fn is_custom(&self) -> bool {
        !self.is_dev() && !self.is_release()
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::DEV
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Profile> for SmolStr {
    fn from(profile: Profile) -> Self {
        profile.0
    }
}

pub trait DefaultForProfile {
    fn default_for_profile(profile: &Profile) -> Self;
}