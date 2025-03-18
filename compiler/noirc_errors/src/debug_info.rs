use acvm::acir::circuit::AcirOpcodeLocation;
use acvm::acir::circuit::BrilligOpcodeLocation;
use acvm::acir::circuit::OpcodeLocation;
use acvm::acir::circuit::brillig::BrilligFunctionId;
use acvm::compiler::AcirTransformationMap;

use base64::Engine;
use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use serde::Deserializer;
use serde::Serializer;
//use serde_with::DisplayFromStr;
//use serde_with::serde_as;
use std::collections::BTreeMap;
use std::io::Read;
use std::io::Write;
use std::mem;

use crate::Location;
use crate::call_stack::CallStackId;
use crate::call_stack::LocationTree;
use noirc_printable_type::PrintableType;
use serde::{
    Deserialize, Serialize, de::Error as DeserializationError, ser::Error as SerializationError,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugVarId(pub u32);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugFnId(pub u32);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugTypeId(pub u32);

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DebugVariable {
    pub name: String,
    pub debug_type_id: DebugTypeId,
}

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DebugFunction {
    pub name: String,
    pub arg_names: Vec<String>,
}

pub type DebugVariables = BTreeMap<DebugVarId, DebugVariable>;
pub type DebugFunctions = BTreeMap<DebugFnId, DebugFunction>;
pub type DebugTypes = BTreeMap<DebugTypeId, PrintableType>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct ProcedureDebugId(pub u32);

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct ProgramDebugInfo {
    pub debug_infos: Vec<DebugInfo>,
}

impl ProgramDebugInfo {
    pub fn serialize_compressed_base64_json<S>(
        debug_info: &ProgramDebugInfo,
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
    ) -> Result<ProgramDebugInfo, D::Error>
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

#[derive(Default, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct DebugInfo {
    pub brillig_locations:
        BTreeMap<BrilligFunctionId, BTreeMap<BrilligOpcodeLocation, CallStackId>>,
    pub location_tree: LocationTree,
    /// Map opcode index of an ACIR circuit into the source code location
    pub acir_locations: BTreeMap<AcirOpcodeLocation, CallStackId>,
    pub variables: DebugVariables,
    pub functions: DebugFunctions,
    pub types: DebugTypes,
    /// This a map per brillig function representing the range of opcodes where a procedure is activated.
    pub brillig_procedure_locs:
        BTreeMap<BrilligFunctionId, BTreeMap<ProcedureDebugId, (usize, usize)>>,
}

impl DebugInfo {
    pub fn new(
        brillig_locations: BTreeMap<
            BrilligFunctionId,
            BTreeMap<BrilligOpcodeLocation, CallStackId>,
        >,
        location_map: BTreeMap<AcirOpcodeLocation, CallStackId>,
        location_tree: LocationTree,
        variables: DebugVariables,
        functions: DebugFunctions,
        types: DebugTypes,
        brillig_procedure_locs: BTreeMap<
            BrilligFunctionId,
            BTreeMap<ProcedureDebugId, (usize, usize)>,
        >,
    ) -> Self {
        Self {
            brillig_locations,
            acir_locations: location_map,
            location_tree,
            variables,
            functions,
            types,
            brillig_procedure_locs,
        }
    }

    /// Updates the locations map when the [`Circuit`][acvm::acir::circuit::Circuit] is modified.
    ///
    /// The [`OpcodeLocation`]s are generated with the ACIR, but passing the ACIR through a transformation step
    /// renders the old `OpcodeLocation`s invalid. The AcirTransformationMap is able to map the old `OpcodeLocation` to the new ones.
    /// Note: One old `OpcodeLocation` might have transformed into more than one new `OpcodeLocation`.
    #[tracing::instrument(level = "trace", skip(self, update_map))]
    pub fn update_acir(&mut self, update_map: AcirTransformationMap) {
        let old_locations = mem::take(&mut self.acir_locations);

        for (old_opcode_location, source_locations) in old_locations {
            update_map.new_acir_locations(old_opcode_location).for_each(|new_opcode_location| {
                self.acir_locations.insert(new_opcode_location, source_locations);
            });
        }
    }

    pub fn acir_opcode_location(&self, loc: &AcirOpcodeLocation) -> Option<Vec<Location>> {
        self.acir_locations
            .get(loc)
            .map(|call_stack_id| self.location_tree.get_call_stack(*call_stack_id))
    }

    pub fn opcode_location(&self, loc: &OpcodeLocation) -> Option<Vec<Location>> {
        match loc {
            OpcodeLocation::Brillig { .. } => None, //TODO: need brillig function id in order to look into brillig_locations
            OpcodeLocation::Acir(loc) => self.acir_opcode_location(&AcirOpcodeLocation::new(*loc)),
        }
    }
}
