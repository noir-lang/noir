use acvm::acir::circuit::OpcodeLocation;
use acvm::compiler::AcirTransformationMap;
use fm::FileId;

use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::collections::{BTreeMap, HashMap};
use std::mem;

use crate::Location;
use noirc_printable_type::PrintableType;
use serde::{Deserialize, Serialize};

pub type Variables = Vec<(u32, (String, u32))>;
pub type Types = Vec<(u32, PrintableType)>;
pub type VariableTypes = (Variables, Types);

#[serde_as]
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct DebugInfo {
    /// Map opcode index of an ACIR circuit into the source code location
    /// Serde does not support mapping keys being enums for json, so we indicate
    /// that they should be serialized to/from strings.
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub locations: BTreeMap<OpcodeLocation, Vec<Location>>,
    pub variables: HashMap<u32, (String, u32)>, // var_id => (name, type_id)
    pub types: HashMap<u32, PrintableType>,     // type_id => printable type
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
        var_types: VariableTypes,
    ) -> Self {
        Self {
            locations,
            variables: var_types.0.into_iter().collect(),
            types: var_types.1.into_iter().collect(),
        }
    }

    /// Updates the locations map when the [`Circuit`][acvm::acir::circuit::Circuit] is modified.
    ///
    /// The [`OpcodeLocation`]s are generated with the ACIR, but passing the ACIR through a transformation step
    /// renders the old `OpcodeLocation`s invalid. The AcirTransformationMap is able to map the old `OpcodeLocation` to the new ones.
    /// Note: One old `OpcodeLocation` might have transformed into more than one new `OpcodeLocation`.
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
                let opcodes = accumulator.entry(*location).or_insert(Vec::new());
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

    pub fn get_file_ids(&self) -> Vec<FileId> {
        self.locations
            .values()
            .filter_map(|call_stack| call_stack.last().map(|location| location.file))
            .collect()
    }
}
