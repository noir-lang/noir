use fxhash::FxHashMap;
use serde::{Deserialize, Serialize};

use noirc_errors::Location;

pub(crate) type CallStack = Vec<Location>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct CallStackId(u32);

impl CallStackId {
    pub(crate) fn root() -> Self {
        Self::new(0)
    }

    fn new(id: usize) -> Self {
        Self(id as u32)
    }

    pub(crate) fn index(&self) -> usize {
        self.0 as usize
    }

    pub(crate) fn is_root(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LocationNode {
    pub(crate) parent: Option<CallStackId>,
    pub(crate) children: Vec<CallStackId>,
    pub(crate) children_hash: FxHashMap<u64, CallStackId>,
    pub(crate) value: Location,
}

impl LocationNode {
    pub(crate) fn new(parent: Option<CallStackId>, value: Location) -> Self {
        LocationNode { parent, children: Vec::new(), children_hash: FxHashMap::default(), value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CallStackHelper {
    locations: Vec<LocationNode>,
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
    pub(crate) fn get_call_stack(&self, mut call_stack: CallStackId) -> CallStack {
        let mut result = Vec::new();
        while let Some(parent) = self.locations[call_stack.index()].parent {
            result.push(self.locations[call_stack.index()].value);
            call_stack = parent;
        }
        result
    }

    /// Returns a new CallStackId which extends the call_stack with the provided call_stack.
    pub(crate) fn extend_call_stack(
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
    pub(crate) fn add_child(&mut self, call_stack: CallStackId, location: Location) -> CallStackId {
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
    pub(crate) fn unwind_call_stack(
        &self,
        mut call_stack: CallStackId,
        mut len: usize,
    ) -> CallStackId {
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

    pub(crate) fn add_location_to_root(&mut self, location: Location) -> CallStackId {
        if self.locations.is_empty() {
            self.locations.push(LocationNode::new(None, location));
            CallStackId::root()
        } else {
            self.add_child(CallStackId::root(), location)
        }
    }

    /// Get (or create) a CallStackId corresponding to the given locations
    pub(crate) fn get_or_insert_locations(&mut self, locations: CallStack) -> CallStackId {
        self.extend_call_stack(CallStackId::root(), &locations)
    }
}
