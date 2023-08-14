use std::collections::BTreeMap;

use crate::location_stack::LocationStack;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct DebugInfo {
    /// Map opcode index of an ACIR circuit into the source code location
    pub locations: BTreeMap<usize, LocationStack>>,
}

impl DebugInfo {
    pub fn new(locations: BTreeMap<usize, LocationStack>) -> Self {
        DebugInfo { locations }
    }

    /// Updates the locations map when the circuit is modified
    ///
    /// When the circuit is generated, the indices are 0,1,..,n
    /// When the circuit is modified, the opcodes are eventually
    /// mixed, removed, or with new ones. For instance 5,2,6,n+1,0,12,..
    /// Since new opcodes (n+1 in the ex) don't have a location
    /// we use the index of the old opcode that they replace.
    /// This is the case during fallback or width 'optimization'
    /// opcode_indices is this list of mixed indices
    pub fn update_acir(&mut self, opcode_indices: Vec<usize>) {
        let mut new_locations = BTreeMap::new();
        for (i, idx) in opcode_indices.iter().enumerate() {
            if self.locations.contains_key(idx) {
                new_locations.insert(i, self.locations[idx].clone());
            }
        }
        self.locations = new_locations;
    }

    pub fn opcode_location(&self, idx: usize) -> Option<&LocationStack> {
        self.locations.get(&idx)
    }
}
