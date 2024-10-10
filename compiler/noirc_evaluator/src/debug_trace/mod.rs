use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct DebugTraceSourcePoint {
    pub file: String,
    pub line_number: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugTraceAsmListIndexRange {
    pub start: usize,
    pub end: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugTraceList {
    pub list: Vec<String>,
    pub source_map: HashMap<DebugTraceSourcePoint, Vec<DebugTraceAsmListIndexRange>>,
}

impl DebugTraceList {
    pub fn new() -> DebugTraceList {
        DebugTraceList { list: vec![], source_map: HashMap::new() }
    }
}
