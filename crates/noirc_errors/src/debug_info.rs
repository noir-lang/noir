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
    /// Serde does not support mapping keys being enums for json, so we indicate
    /// that they should be serialized to/from strings.
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub locations: BTreeMap<OpcodeLocation, Vec<Location>>,
}

impl DebugInfo {
    pub fn new(locations: BTreeMap<OpcodeLocation, Vec<Location>>) -> Self {
        DebugInfo { locations }
    }

    /// Updates the locations map when the circuit is modified.
    /// The OpcodeLocations are generated with the ACIR, but passing the ACIR through a transformation step
    /// renders the old OpcodeLocations invalid. The AcirTransformationMap is able to map the old OpcodeLocation to the new ones.
    /// Note: One old OpcodeLocation might have transformed into more than one new OpcodeLocation.
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
