use std::collections::HashMap;

//use noirc_errors::Span;
use serde::{Deserialize, Serialize};
//use fm::FileId;
use crate::Location;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct DebugInfo {
    /// Map opcode index of an ACIR circuit into the source code location
    pub locations: HashMap<usize, Location>,
}

impl DebugInfo {
    pub fn new(locations: HashMap<usize, Location>) -> Self {
        DebugInfo { locations }
    }

    pub fn update_acir(&mut self, opcode_idx: Vec<usize>) {
        let mut new_locations = HashMap::new();
        for (i, idx) in opcode_idx.iter().enumerate() {
            if self.locations.contains_key(idx) {
                new_locations.insert(i, self.locations[idx]);
            }
        }
        self.locations = new_locations;
    }

    pub fn opcode_location(&self, idx: usize) -> Option<&Location> {
        self.locations.get(&idx)
        // if let Some((start, end, file_id)) = self.locations.get(&idx) {

        //     let span = Span::exclusive(*start,*end);
        //     let f = FileId::new(*file_id);
        //     return Some(Location::new(span, f));
        // }
        // None
    }
}
