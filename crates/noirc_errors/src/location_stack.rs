use crate::Location;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationStack(Vec<Location>);

impl LocationStack {
    pub fn new() -> Self {
        LocationStack(Vec::new())
    }

    pub fn push(&mut self, new_location: Location) {
        self.0.push(new_location)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn or(self, other: LocationStack) -> LocationStack {
        if self.is_empty() {
            return other;
        }
        self
    }
}

impl IntoIterator for LocationStack {
    type Item = Location;
    type IntoIter = <Vec<Location> as IntoIterator>::IntoIter; // so that you don't have to write std::vec::IntoIter, which nobody remembers anyway

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for LocationStack {
    type Target = Vec<Location>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Location>> for LocationStack {
    fn from(value: Vec<Location>) -> Self {
        LocationStack(value)
    }
}

impl Into<Vec<Location>> for LocationStack {
    fn into(self) -> Vec<Location> {
        self.0
    }
}
