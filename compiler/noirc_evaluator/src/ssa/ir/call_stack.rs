use serde::{Deserialize, Serialize};

use noirc_errors::Location;

pub(crate) type CallStack = im::Vector<Location>;

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

    /// Construct a CallStack from a CallStackId
    pub(crate) fn get_call_stack(&self, locations: &[LocationNode]) -> CallStack {
        let mut call_stack = im::Vector::new();
        let mut current_location = *self;
        while let Some(parent) = locations[current_location.index()].parent {
            call_stack.push_back(locations[current_location.index()].value);
            current_location = parent;
        }
        call_stack
    }

    /// Adds a location to the call stack
    pub(crate) fn add_child(
        &self,
        location: Location,
        locations: &mut Vec<LocationNode>,
    ) -> CallStackId {
        if let Some(result) = locations[self.index()]
            .children
            .iter()
            .rev()
            .take(1000)
            .find(|child| locations[child.index()].value == location)
        {
            return *result;
        }
        locations.push(LocationNode { parent: Some(*self), children: vec![], value: location });
        let new_location = CallStackId::new(locations.len() - 1);
        locations[self.index()].children.push(new_location);
        new_location
    }

    /// Returns a new CallStackId which extends the current one with the provided call_stack.
    pub(crate) fn extend(
        &self,
        call_stack: &CallStack,
        locations: &mut Vec<LocationNode>,
    ) -> CallStackId {
        let mut result = *self;
        for location in call_stack {
            result = result.add_child(*location, locations);
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LocationNode {
    pub(crate) parent: Option<CallStackId>,
    pub(crate) children: Vec<CallStackId>,
    pub(crate) value: Location,
}
