use fm::{FileId, FileManager};
use noirc_errors::debug_info::DebugInfo;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

/// For a given file, we store the source code and the path to the file
/// so consumers of the debug artifact can reconstruct the original source code structure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebugFile {
    pub source: String,
    pub path: PathBuf,
}

pub(crate) fn filter_relevant_files(
    debug_symbols: &[DebugInfo],
    file_manager: &FileManager,
) -> BTreeMap<FileId, DebugFile> {
    let files_with_debug_symbols: BTreeSet<FileId> = debug_symbols
        .iter()
        .flat_map(|function_symbols| {
            function_symbols
                .locations
                .values()
                .filter_map(|call_stack| call_stack.last().map(|location| location.file))
        })
        .collect();

    let mut file_map = BTreeMap::new();

    for file_id in files_with_debug_symbols {
        let file_source = file_manager.fetch_file(file_id).source();

        file_map.insert(
            file_id,
            DebugFile {
                source: file_source.to_string(),
                path: file_manager.path(file_id).to_path_buf(),
            },
        );
    }
    file_map
}
