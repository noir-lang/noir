use acvm::acir::circuit::OpcodeLocation;
use acvm::compiler::AcirTransformationMap;

use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::collections::BTreeMap;

use crate::Location;
use serde::{Deserialize, Serialize};

#[serde_as]
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct DebugInfo {
    /// Map opcode index of an ACIR circuit into the source code location
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub locations: BTreeMap<OpcodeLocation, Vec<Location>>,
}

impl DebugInfo {
    pub fn new(locations: BTreeMap<OpcodeLocation, Vec<Location>>) -> Self {
        DebugInfo { locations }
    }

    /// Updates the locations map when the circuit is modified
    pub fn update_acir(&mut self, update_map: AcirTransformationMap) {
        let mut new_locations_map = BTreeMap::new();

        for (old_opcode_location, source_locations) in &self.locations {
            let new_opcode_locations = update_map.new_locations(*old_opcode_location);
            for new_opcode_location in new_opcode_locations {
                new_locations_map.insert(new_opcode_location, source_locations.clone());
            }
        }

        self.locations = new_locations_map;
    }

    pub fn opcode_location(&self, loc: &OpcodeLocation) -> Option<Vec<Location>> {
        self.locations.get(loc).cloned()
    }
}
