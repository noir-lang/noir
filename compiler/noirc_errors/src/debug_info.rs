// Debug info module - stubbed out for Sensei
// This module previously contained ACIR debugging information
// which is not needed without the ZK backend

use crate::Location;
use noirc_printable_type::PrintableType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugVarId(pub u32);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugTypeId(pub u32);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugFnId(pub u32);

#[derive(Default, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct DebugInfo {
    // Placeholder fields - to be defined based on Sensei's debugging needs
    pub locations: BTreeMap<usize, Location>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ProgramDebugInfo {
    pub debug_infos: Vec<DebugInfo>,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct DebugVariable {
    pub name: String,
    pub debug_type_id: DebugTypeId,
}

impl Default for DebugVariable {
    fn default() -> Self {
        Self {
            name: String::new(),
            debug_type_id: DebugTypeId(0),
        }
    }
}

#[derive(Default, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct DebugFunction {
    pub name: String,
    pub location: Option<Location>,
    pub arg_names: Vec<String>,
}

pub type DebugVariables = BTreeMap<DebugVarId, DebugVariable>;
pub type DebugFunctions = BTreeMap<DebugFnId, DebugFunction>;
pub type DebugTypes = BTreeMap<DebugTypeId, PrintableType>;

impl DebugInfo {
    pub fn new() -> Self {
        Self::default()
    }
}