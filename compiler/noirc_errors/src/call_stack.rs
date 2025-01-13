use fxhash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::Location;

pub type CallStack = Vec<Location>;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallStackId(u32);

impl CallStackId {
    pub fn root() -> Self {
        Self::new(0)
    }

    pub fn new(id: usize) -> Self {
        Self(id as u32)
    }

    pub fn index(&self) -> usize {
        self.0 as usize
    }

    pub fn is_root(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct LocationNodeDebugInfo {
    pub parent: Option<CallStackId>,
    pub value: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Hash)]
pub struct LocationTree {
    pub locations: Vec<LocationNodeDebugInfo>,
}

impl LocationTree {
    /// Construct a CallStack from a CallStackId
    pub fn get_call_stack(&self, mut call_stack: CallStackId) -> CallStack {
        let mut result = Vec::new();
        while let Some(parent) = self.locations[call_stack.index()].parent {
            result.push(self.locations[call_stack.index()].value);
            call_stack = parent;
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationNode {
    pub parent: Option<CallStackId>,
    pub children: Vec<CallStackId>,
    pub children_hash: FxHashMap<u64, CallStackId>,
    pub value: Location,
}

impl LocationNode {
    pub fn new(parent: Option<CallStackId>, value: Location) -> Self {
        LocationNode { parent, children: Vec::new(), children_hash: FxHashMap::default(), value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStackHelper {
    pub locations: Vec<LocationNode>,
}

impl Default for CallStackHelper {
    /// Generates a new helper, with an empty location tree
    fn default() -> Self {
        let mut result = CallStackHelper { locations: Vec::new() };
        result.add_location_to_root(Location::dummy());
        result
    }
}

impl CallStackHelper {
    /// Construct a CallStack from a CallStackId
    pub fn get_call_stack(&self, mut call_stack: CallStackId) -> CallStack {
        let mut result = Vec::new();
        while let Some(parent) = self.locations[call_stack.index()].parent {
            result.push(self.locations[call_stack.index()].value);
            call_stack = parent;
        }
        result
    }

    /// Returns a new CallStackId which extends the call_stack with the provided call_stack.
    pub fn extend_call_stack(
        &mut self,
        mut call_stack: CallStackId,
        locations: &CallStack,
    ) -> CallStackId {
        for location in locations {
            call_stack = self.add_child(call_stack, *location);
        }
        call_stack
    }

    /// Adds a location to the call stack
    pub fn add_child(&mut self, call_stack: CallStackId, location: Location) -> CallStackId {
        let key = fxhash::hash64(&location);
        if let Some(result) = self.locations[call_stack.index()].children_hash.get(&key) {
            if self.locations[result.index()].value == location {
                return *result;
            }
        }
        let new_location = LocationNode::new(Some(call_stack), location);
        let key = fxhash::hash64(&new_location.value);
        self.locations.push(new_location);
        let new_location_id = CallStackId::new(self.locations.len() - 1);

        self.locations[call_stack.index()].children.push(new_location_id);
        self.locations[call_stack.index()].children_hash.insert(key, new_location_id);
        new_location_id
    }

    /// Retrieve the CallStackId corresponding to call_stack with the last 'len' locations removed.
    pub fn unwind_call_stack(&self, mut call_stack: CallStackId, mut len: usize) -> CallStackId {
        while len > 0 {
            if let Some(parent) = self.locations[call_stack.index()].parent {
                len -= 1;
                call_stack = parent;
            } else {
                break;
            }
        }
        call_stack
    }

    pub fn add_location_to_root(&mut self, location: Location) -> CallStackId {
        if self.locations.is_empty() {
            self.locations.push(LocationNode::new(None, location));
            CallStackId::root()
        } else {
            self.add_child(CallStackId::root(), location)
        }
    }

    /// Get (or create) a CallStackId corresponding to the given locations
    pub fn get_or_insert_locations(&mut self, locations: &CallStack) -> CallStackId {
        self.extend_call_stack(CallStackId::root(), locations)
    }

    // Clone the locations into a LocationTree
    pub fn to_location_tree(&self) -> LocationTree {
        LocationTree {
            locations: self
                .locations
                .clone()
                .into_iter()
                .map(|node| LocationNodeDebugInfo { value: node.value, parent: node.parent })
                .collect(),
        }
    }
}
