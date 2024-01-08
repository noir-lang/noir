use acvm::acir::circuit::OpcodeLocation;
use acvm::compiler::AcirTransformationMap;

use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::mem;

use crate::Location;
use serde::{Deserialize, Serialize};

#[serde_as]
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct DebugInfo {
    /// Map opcode index of an ACIR circuit into the source code location
    /// Serde does not support mapping keys being enums for json, so we indicate
    /// that they should be serialized to/from strings.
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub locations: BTreeMap<OpcodeLocation, Vec<Location>>,
}

/// Holds OpCodes Counts for Acir and Brillig Opcodes
/// To be printed with `nargo info --profile-info`
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct OpCodesCount {
    pub acir_size: usize,
    pub brillig_size: usize,
}

impl DebugInfo {
    pub fn new(locations: BTreeMap<OpcodeLocation, Vec<Location>>) -> Self {
        DebugInfo { locations }
    }

    /// Updates the locations map when the [`Circuit`][acvm::acir::circuit::Circuit] is modified.
    ///
    /// The [`OpcodeLocation`]s are generated with the ACIR, but passing the ACIR through a transformation step
    /// renders the old `OpcodeLocation`s invalid. The AcirTransformationMap is able to map the old `OpcodeLocation` to the new ones.
    /// Note: One old `OpcodeLocation` might have transformed into more than one new `OpcodeLocation`.
    pub fn update_acir(&mut self, update_map: AcirTransformationMap) {
        log::trace!("Start debug info update");
        let old_locations = mem::take(&mut self.locations);

        for (old_opcode_location, source_locations) in old_locations {
            update_map.new_locations(old_opcode_location).for_each(|new_opcode_location| {
                self.locations.insert(new_opcode_location, source_locations.clone());
            });
        }
        log::trace!("Finish debug info update");
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
}
