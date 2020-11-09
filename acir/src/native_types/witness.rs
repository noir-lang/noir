
// Each witness can be identified with an String
// The string will be derived from the identifier name in the variable

// Lets make this only usize, and have a map in the compiler to map Witness string to usize

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd)]
pub struct Witness(pub String, pub usize);

use std::cmp::Ordering;
impl Ord for Witness {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

impl Default for Witness {
    // Place holder value
    fn default() -> Witness {
        Witness("zero".to_string(), 0)
    }
}

impl Witness {
    pub fn new(variable_name: String, witness_index: usize) -> Witness {
        Witness(variable_name, witness_index)
    }
    pub fn witness_index(&self) -> usize {
        self.1
    }
    pub fn variable_name(&self) -> &str {
        &self.0
    }
    pub const fn can_defer_constraint(&self) -> bool {
        true
    }
}