use std::collections::{HashMap, HashSet};

use fm::FileId;
use noirc_errors::Location;

use crate::hir_def::expr::HirExpression;
use crate::node_interner::FuncId;

/// Track comptime evaluations, to facilitate code coverage in tests.
pub struct EvaluationTracker {
    /// Only locations whose file is in this set are recorded.
    allowed_files: HashSet<FileId>,

    /// Maps each source file to a map of expression start byte offsets to hit counts.
    /// Byte offsets are used rather than line numbers because the tracker has no access
    /// to source text; callers with a `FileManager` can convert offsets to line numbers.
    hits: HashMap<FileId, HashMap<u32, u64>>,

    /// Maps each called function to the number of times it was called. Used to produce
    /// `FunctionData` and `FunctionsHit` lcov records: nargo looks up the function name
    /// and definition line via the interner.
    function_hits: HashMap<FuncId, u64>,
}

impl EvaluationTracker {
    pub fn new(allowed_files: HashSet<FileId>) -> Self {
        Self { allowed_files, hits: HashMap::new(), function_hits: HashMap::new() }
    }

    pub fn track(&mut self, _expr: &HirExpression, location: Location) {
        if location.is_dummy() || !self.allowed_files.contains(&location.file) {
            return;
        }

        self.hits
            .entry(location.file)
            .or_default()
            .entry(location.span.start())
            .and_modify(|n| *n += 1)
            .or_insert(1);
    }

    pub fn track_function_call(&mut self, func_id: FuncId, location: Location) {
        if location.is_dummy() || !self.allowed_files.contains(&location.file) {
            return;
        }

        self.function_hits.entry(func_id).and_modify(|n| *n += 1).or_insert(1);
    }

    /// Returns the raw hit counts: for each file, a map from expression start byte
    /// offset to the number of times that expression was evaluated.
    pub fn hits(&self) -> &HashMap<FileId, HashMap<u32, u64>> {
        &self.hits
    }

    /// Returns per-function call counts. Nargo looks up each `FuncId` in the interner
    /// to get the function name and definition line for `FunctionData` lcov records.
    pub fn function_hits(&self) -> &HashMap<FuncId, u64> {
        &self.function_hits
    }
}
