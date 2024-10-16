use std::borrow::Borrow;
use std::fmt;
use std::str::FromStr;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(into = "SmolStr", try_from = "SmolStr")]
pub struct TargetKind(SmolStr);

impl TargetKind {
    pub const CAIRO_PLUGIN: Self = TargetKind(SmolStr::new_inline("cairo-plugin"));
    pub const LIB: Self = TargetKind(SmolStr::new_inline("lib"));
    pub const TEST: Self = TargetKind(SmolStr::new_inline("test"));
    pub const STARKNET_CONTRACT: Self = TargetKind(SmolStr::new_inline("starknet-contract"));

    /// Constructs and validates new [`TargetKind`].
    ///
    /// Panics if name does not conform to package naming rules.
    pub fn new(name: impl AsRef<str>) -> Self {
        Self::try_new(name).unwrap()
    }

    /// Constructs and validates new [`TargetKind`].
    pub fn try_new(name: impl AsRef<str>) -> Result<Self> {
        Self::try_new_impl(name.as_ref().into())
    }

    fn try_new_impl(name: SmolStr) -> Result<Self> {
        if name.is_empty() {
            bail!("empty string cannot be used as target kind");
        }

        if name == "_" {
            bail!("underscore cannot be used as target kind");
        }

        if name == "-" {
            bail!("dash cannot be used as target kind");
        }

        if name != name.to_ascii_lowercase() {
            bail!(
                "invalid target kind: `{name}`\n\
                note: usage of ASCII uppercase letters in the target kind has been disallowed\n\
                help: change target kind to: {}",
                name.to_ascii_lowercase()
            )
        }

        let mut chars = name.chars();

        // Validate first letter.
        if let Some(ch) = chars.next() {
            // A specific error for a potentially common case.
            if ch.is_ascii_digit() {
                bail!(
                    "the name `{name}` cannot be used as a target kind, \
                    names cannot start with a digit"
                );
            }

            if !(ch.is_ascii_alphabetic() || ch == '-') {
                bail!(
                    "invalid character `{ch}` in target kind: `{name}`, \
                    the first character must be an ASCII lowercase letter or dash"
                )
            }
        }

        // Validate rest.
        for ch in chars {
            if !(ch.is_ascii_alphanumeric() || ch == '-') {
                bail!(
                    "invalid character `{ch}` in target kind: `{name}`, \
                    characters must be ASCII lowercase letters, ASCII numbers or dash"
                )
            }
        }

        Ok(Self(name))
    }

    pub fn is_test(&self) -> bool {
        self == &Self::TEST
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    #[inline(always)]
    pub fn to_smol_str(&self) -> SmolStr {
        self.0.clone()
    }
}

impl AsRef<str> for TargetKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<TargetKind> for SmolStr {
    fn from(value: TargetKind) -> Self {
        value.0
    }
}

impl TryFrom<&str> for TargetKind {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        TargetKind::try_new(value)
    }
}

impl TryFrom<String> for TargetKind {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        TargetKind::try_new(value)
    }
}

impl TryFrom<SmolStr> for TargetKind {
    type Error = anyhow::Error;

    fn try_from(value: SmolStr) -> Result<Self> {
        TargetKind::try_new(value.as_str())
    }
}

impl FromStr for TargetKind {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> Result<Self> {
        TargetKind::try_new(name)
    }
}

impl Borrow<str> for TargetKind {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl From<TargetKind> for String {
    fn from(value: TargetKind) -> Self {
        value.to_string()
    }
}

impl fmt::Display for TargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for TargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TargetKind({self})")
    }
}