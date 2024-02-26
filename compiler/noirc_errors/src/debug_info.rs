use acvm::acir::circuit::OpcodeLocation;
use acvm::compiler::AcirTransformationMap;

use base64::Engine;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use serde::Deserializer;
use serde::Serializer;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::mem;

use crate::Location;
use noirc_printable_type::PrintableType;
use serde::{
    de::Error as DeserializationError, ser::Error as SerializationError, Deserialize, Serialize,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugVarId(pub u32);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugTypeId(pub u32);

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DebugVariable {
    pub name: String,
    pub debug_type_id: DebugTypeId,
}

pub type DebugVariables = BTreeMap<DebugVarId, DebugVariable>;
pub type DebugTypes = BTreeMap<DebugTypeId, PrintableType>;

#[serde_as]
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct DebugInfo {
    /// Map opcode index of an ACIR circuit into the source code location
    /// Serde does not support mapping keys being enums for json, so we indicate
    /// that they should be serialized to/from strings.
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub locations: BTreeMap<OpcodeLocation, Vec<Location>>,
    pub variables: DebugVariables,
    pub types: DebugTypes,
}

/// Holds OpCodes Counts for Acir and Brillig Opcodes
/// To be printed with `nargo info --profile-info`
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct OpCodesCount {
    pub acir_size: usize,
    pub brillig_size: usize,
}

impl DebugInfo {
    pub fn new(
        locations: BTreeMap<OpcodeLocation, Vec<Location>>,
        variables: DebugVariables,
        types: DebugTypes,
    ) -> Self {
        Self { locations, variables, types }
    }

    /// Updates the locations map when the [`Circuit`][acvm::acir::circuit::Circuit] is modified.
    ///
    /// The [`OpcodeLocation`]s are generated with the ACIR, but passing the ACIR through a transformation step
    /// renders the old `OpcodeLocation`s invalid. The AcirTransformationMap is able to map the old `OpcodeLocation` to the new ones.
    /// Note: One old `OpcodeLocation` might have transformed into more than one new `OpcodeLocation`.
    #[tracing::instrument(level = "trace", skip(self, update_map))]
    pub fn update_acir(&mut self, update_map: AcirTransformationMap) {
        let old_locations = mem::take(&mut self.locations);

        for (old_opcode_location, source_locations) in old_locations {
            update_map.new_locations(old_opcode_location).for_each(|new_opcode_location| {
                self.locations.insert(new_opcode_location, source_locations.clone());
            });
        }
    }

    pub fn opcode_location(&self, loc: &OpcodeLocation) -> Option<Vec<Location>> {
        self.locations.get(loc).cloned()
    }

    pub fn count_span_opcodes(&self) -> HashMap<Location, OpCodesCount> {
        let mut accumulator: HashMap<Location, Vec<&OpcodeLocation>> = HashMap::new();

        for (opcode_location, locations) in self.locations.iter() {
            for location in locations.iter() {
                let opcodes = accumulator.entry(*location).or_default();
                opcodes.push(opcode_location);
            }
        }

        let counted_opcodes = accumulator
            .iter()
            .map(|(location, opcodes)| {
                let acir_opcodes: Vec<_> = opcodes
                    .iter()
                    .filter(|opcode_location| matches!(opcode_location, OpcodeLocation::Acir(_)))
                    .collect();
                let brillig_opcodes: Vec<_> = opcodes
                    .iter()
                    .filter(|opcode_location| {
                        matches!(opcode_location, OpcodeLocation::Brillig { .. })
                    })
                    .collect();
                let opcodes_count = OpCodesCount {
                    acir_size: acir_opcodes.len(),
                    brillig_size: brillig_opcodes.len(),
                };
                (*location, opcodes_count)
            })
            .collect();

        counted_opcodes
    }

    pub fn serialize_compressed_base64_json<S>(
        debug_info: &DebugInfo,
        s: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json_str = serde_json::to_string(debug_info).map_err(S::Error::custom)?;

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json_str.as_bytes()).map_err(S::Error::custom)?;
        let compressed_data = encoder.finish().map_err(S::Error::custom)?;

        let encoded_b64 = base64::prelude::BASE64_STANDARD.encode(compressed_data);
        s.serialize_str(&encoded_b64)
    }

    pub fn deserialize_compressed_base64_json<'de, D>(
        deserializer: D,
    ) -> Result<DebugInfo, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded_b64: String = Deserialize::deserialize(deserializer)?;

        let compressed_data =
            base64::prelude::BASE64_STANDARD.decode(encoded_b64).map_err(D::Error::custom)?;

        let mut decoder = DeflateDecoder::new(&compressed_data[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data).map_err(D::Error::custom)?;

        let json_str = String::from_utf8(decompressed_data).map_err(D::Error::custom)?;
        serde_json::from_str(&json_str).map_err(D::Error::custom)
    }
}
