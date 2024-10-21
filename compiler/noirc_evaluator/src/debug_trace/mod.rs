use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct SourcePoint {
    pub file: String,
    pub line_number: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AsmListIndexRange {
    pub start: usize,
    pub end: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugTraceList {
    pub list: Vec<String>,
    pub source_map: HashMap<SourcePoint, VecDeque<AsmListIndexRange>>,
}

impl DebugTraceList {
    pub fn new() -> DebugTraceList {
        DebugTraceList { list: vec![], source_map: HashMap::new() }
    }
}
