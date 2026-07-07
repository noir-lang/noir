#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

//! This module defines the structure of Nargo's different compilation artifacts.
//!
//! These artifacts are intended to remain independent of any applications being built on top of Noir.
//! Should any projects require/desire a different artifact format, it's expected that they will write a transformer
//! to generate them using these artifacts as a starting point.

use serde::{Deserializer, Serializer, de::Visitor};

pub mod contract;
pub mod debug;
mod debug_vars;
pub mod program;
pub mod ssa;

/// Serialize `hash` as `String`, so that it doesn't get truncated in Javascript.
fn serialize_hash<S>(hash: &u64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&hash.to_string())
}

/// Deserialize `hash` from `String` in JSON.
fn deserialize_hash<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    // Backwards compatible with `hash` serialized as a number.
    struct StringOrU64;

    impl Visitor<'_> for StringOrU64 {
        type Value = u64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("String or u64")
        }

        fn visit_str<E>(self, value: &str) -> Result<u64, E>
        where
            E: Error,
        {
            value.parse().map_err(E::custom)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
            Ok(value)
        }
    }
    deserializer.deserialize_any(StringOrU64)
}
