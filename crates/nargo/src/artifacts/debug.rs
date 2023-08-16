use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use fm::FileId;
use noirc_errors::debug_info::DebugInfo;
use noirc_frontend::hir::Context;

/// For a given file, we store the source code and the path to the file
/// so consumers of the debug artifact can reconstruct the original source code structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct DebugFile {
    pub source: String,
    pub path: PathBuf,
}

/// A Debug Artifact stores, for a given program, the debug info for every function
/// along with a map of file Id to the source code so locations in debug info can be mapped to source code they point to.
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
            .flat_map(|function_symbols| {
                function_symbols
                    .locations
                    .values()
                    .filter_map(|call_stack| call_stack.last().map(|location| location.file))
            })
            .collect();

        for file_id in files_with_debug_symbols {
            let file_source = compilation_context.file_manager.fetch_file(file_id).source();

            file_map.insert(
                file_id,
                DebugFile {
                    source: file_source.to_string(),
                    path: compilation_context.file_manager.path(file_id).to_path_buf(),
                },
            );
        }

        Self { debug_symbols, file_map }
    }
}
