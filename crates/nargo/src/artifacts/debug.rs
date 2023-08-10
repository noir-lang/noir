use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use fm::FileId;
use noirc_errors::debug_info::DebugInfo;
use noirc_frontend::hir::Context;

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugFile {
    pub source: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugArtifact {
    pub debug_symbols: Vec<DebugInfo>,
    pub file_map: BTreeMap<FileId, DebugFile>,
}

impl DebugArtifact {
    pub fn new(debug_symbols: Vec<DebugInfo>, compilation_context: &Context) -> Self {
        let mut file_map = BTreeMap::new();

        let files_with_debug_symbols: BTreeSet<FileId> = debug_symbols
            .iter()
            .flat_map(|function_symbols| function_symbols.locations.values().map(|loc| loc.file))
            .collect();

        for file_id in files_with_debug_symbols {
            let file_source = compilation_context.file_manager.fetch_file(file_id).source();

            file_map.insert(
                file_id,
                DebugFile {
                    source: file_source.to_string(),
                    path: compilation_context
                        .file_manager
                        .path(file_id)
                        .to_string_lossy()
                        .to_string(),
                },
            );
        }

        Self { debug_symbols, file_map }
    }
}
