//use serde_with::DisplayFromStr;
//use serde_with::serde_as;
use noirc_printable_type::PrintableType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugVarId(pub u32);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugFnId(pub u32);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DebugTypeId(pub u32);

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DebugVariable {
    pub name: String,
    pub debug_type_id: DebugTypeId,
}

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DebugFunction {
    pub name: String,
    pub arg_names: Vec<String>,
}

pub type DebugVariables = BTreeMap<DebugVarId, DebugVariable>;
pub type DebugFunctions = BTreeMap<DebugFnId, DebugFunction>;
pub type DebugTypes = BTreeMap<DebugTypeId, PrintableType>;
