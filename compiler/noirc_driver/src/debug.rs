use fm::{FileId, FileManager};
use noirc_errors::debug_info::DebugInfo;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

/// For a given file, we store the source code and the path to the file
/// so consumers of the debug artifact can reconstruct the original source code structure.
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct DebugFile {
    pub source: String,
    pub path: PathBuf,
}

pub(crate) fn filter_relevant_files(
    debug_symbols: &[DebugInfo],
    file_manager: &FileManager,
) -> BTreeMap<FileId, DebugFile> {
    let mut files_with_debug_symbols: BTreeSet<FileId> = debug_symbols
        .iter()
        .flat_map(|function_symbols| {
            function_symbols
                .locations
                .values()
                .flat_map(|call_stack| call_stack.iter().map(|location| location.file))
        })
        .collect();

    let files_with_brillig_debug_symbols: BTreeSet<FileId> = debug_symbols
        .iter()
        .flat_map(|function_symbols| {
            let brillig_location_maps =
                function_symbols.brillig_locations.values().flat_map(|brillig_location_map| {
                    brillig_location_map
                        .values()
                        .flat_map(|call_stack| call_stack.iter().map(|location| location.file))
                });
            brillig_location_maps
        })
        .collect();

    files_with_debug_symbols.extend(files_with_brillig_debug_symbols);

    let mut file_map = BTreeMap::new();

    for file_id in files_with_debug_symbols {
        let file_path = file_manager.path(file_id).expect("file should exist");
        let file_source = file_manager.fetch_file(file_id).expect("file should exist");

        file_map.insert(
            file_id,
            DebugFile { source: file_source.to_string(), path: file_path.to_path_buf() },
        );
    }
    file_map
}
