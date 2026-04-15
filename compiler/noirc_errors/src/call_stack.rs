use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::hash::BuildHasher;

use crate::Location;

/// A non-empty list of [Location]s.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CallStack(Vec<Location>);

impl CallStack {
    /// Construct a new call stack from a potentially empty list of [Location]s.
    pub fn new(locations: Vec<Location>) -> Self {
        Self(locations)
    }

    /// Constructor to use when we don't have location information.
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Check if the call stack is empty.
    ///
    /// A call stack can be non-empty and end still end in a dummy location.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the last location, or a dummy one if it's empty.
    pub fn last_or_dummy(&self) -> Location {
        self.0.last().copied().unwrap_or(Location::dummy())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for CallStack {
    type Item = Location;

    type IntoIter = <Vec<Location> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a CallStack {
    type Item = &'a Location;
    type IntoIter = std::slice::Iter<'a, Location>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl AsRef<[Location]> for CallStack {
    fn as_ref(&self) -> &[Location] {
        &self.0
    }
}

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
    pub fn locations(&self) -> &[LocationNode] {
        &self.locations
    }

    /// Construct a CallStack from a CallStackId
    pub fn get_call_stack(&self, mut call_stack: CallStackId) -> CallStack {
        let mut result = Vec::new();
        while let Some(parent) = self.locations[call_stack.index()].parent {
            result.push(self.locations[call_stack.index()].value);
            call_stack = parent;
        }
        result.reverse();
        CallStack::new(result)
    }

    /// Returns a new [CallStackId] which extends the `call_stack` with the provided `locations`.
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

    /// Adds a location to the call stack, maintaining the location cache along the way.
    pub fn add_child(&mut self, call_stack: CallStackId, location: Location) -> CallStackId {
        let key = rustc_hash::FxBuildHasher.hash_one(location);
        if let Some(result) = self.locations[call_stack.index()].children_hash.get(&key)
            && self.locations[result.index()].value == location
        {
            return *result;
        }
        let new_location = LocationNode::new(Some(call_stack), location);
        let key = rustc_hash::FxBuildHasher.hash_one(new_location.value);
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

    /// Get (or create) a CallStackId corresponding to the given locations.
    pub fn get_or_insert_locations(&mut self, locations: &CallStack) -> CallStackId {
        self.extend_call_stack(CallStackId::root(), locations)
    }
}

#[cfg(test)]
mod tests {
    use crate::call_stack::{CallStackHelper, CallStackId};

    #[test]
    fn root_call_stack_is_empty() {
        let helper = CallStackHelper::default();
        let root_call_stack = helper.get_call_stack(CallStackId::root());
        assert!(root_call_stack.is_empty());
    }
}
