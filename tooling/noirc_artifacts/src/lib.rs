#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

//! This module defines the structure of Nargo's different compilation artifacts.
//!
//! These artifacts are intended to remain independent of any applications being built on top of Noir.
//! Should any projects require/desire a different artifact format, it's expected that they will write a transformer
//! to generate them using these artifacts as a starting point.
//! The serialized format is the compatibility boundary. This crate's Rust API is an internal
//! implementation detail and may change between Noir releases.

use serde::{Deserialize, Deserializer, Serializer, de::Visitor};

pub mod contract;
pub mod debug;
mod debug_vars;
pub mod program;
pub mod ssa;

/// The version of the artifact schema emitted by this version of Noir.
pub const ARTIFACT_VERSION: u32 = 1;

/// The schema version of artifact JSON written before `ARTIFACT_VERSION` existed. This is a
/// historical fact about those files and must not change when `ARTIFACT_VERSION` is incremented.
const LEGACY_ARTIFACT_VERSION: u32 = 1;

pub(crate) const fn default_artifact_version() -> u32 {
    LEGACY_ARTIFACT_VERSION
}

pub(crate) fn deserialize_artifact_version<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let version = u32::deserialize(deserializer)?;
    if version == ARTIFACT_VERSION {
        Ok(version)
    } else {
        Err(serde::de::Error::custom(format!(
            "unsupported artifact schema version {version}; expected {ARTIFACT_VERSION}"
        )))
    }
}

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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use acvm::{FieldElement, acir::circuit::Program};
    use noirc_abi::Abi;

    use crate::{
        ARTIFACT_VERSION,
        contract::{ContractArtifact, ContractOutputsArtifact},
        debug::ProgramDebugInfo,
        program::ProgramArtifact,
    };

    fn program_artifact() -> ProgramArtifact {
        ProgramArtifact {
            artifact_version: ARTIFACT_VERSION,
            noir_version: "1.0.0".to_owned(),
            hash: 0,
            abi: Abi::default(),
            bytecode: Program::<FieldElement>::default(),
            debug_symbols: ProgramDebugInfo::default(),
            file_map: BTreeMap::new(),
        }
    }

    fn contract_artifact() -> ContractArtifact {
        ContractArtifact {
            artifact_version: ARTIFACT_VERSION,
            noir_version: "1.0.0".to_owned(),
            name: "contract".to_owned(),
            functions: Vec::new(),
            outputs: ContractOutputsArtifact { structs: HashMap::new(), globals: HashMap::new() },
            file_map: BTreeMap::new(),
        }
    }

    #[test]
    fn artifact_schema_version_is_serialized() {
        let program = serde_json::to_value(program_artifact()).unwrap();
        assert_eq!(program["artifact_version"], ARTIFACT_VERSION);

        let contract = serde_json::to_value(contract_artifact()).unwrap();
        assert_eq!(contract["artifact_version"], ARTIFACT_VERSION);
    }

    #[test]
    fn legacy_artifacts_default_to_version_one() {
        let mut program = serde_json::to_value(program_artifact()).unwrap();
        let program = program.as_object_mut().unwrap();
        program.remove("artifact_version");
        let program: ProgramArtifact = serde_json::from_value(program.clone().into()).unwrap();
        assert_eq!(program.artifact_version, 1);

        let mut contract = serde_json::to_value(contract_artifact()).unwrap();
        let contract = contract.as_object_mut().unwrap();
        contract.remove("artifact_version");
        let contract: ContractArtifact = serde_json::from_value(contract.clone().into()).unwrap();
        assert_eq!(contract.artifact_version, 1);
    }

    #[test]
    fn unsupported_artifact_schema_version_is_rejected() {
        let mut artifact = serde_json::to_value(program_artifact()).unwrap();
        artifact["artifact_version"] = (ARTIFACT_VERSION + 1).into();

        let error = serde_json::from_value::<ProgramArtifact>(artifact).unwrap_err();
        assert!(error.to_string().contains("unsupported artifact schema version"));
    }
}
