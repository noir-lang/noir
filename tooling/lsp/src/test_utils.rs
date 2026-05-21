use crate::LspState;
use crate::notifications::on_did_open_text_document;
use acvm::blackbox_solver::StubbedBlackBoxSolver;
use async_lsp::ClientSocket;
use async_lsp::lsp_types::{
    DidOpenTextDocumentParams, InitializeParams, Position, Range, TextDocumentItem, Url,
    WorkDoneProgressParams,
};

pub(crate) async fn init_lsp_server(directory: &str) -> (LspState, Url) {
    let client = ClientSocket::new_closed();
    let mut state = LspState::new(&client, StubbedBlackBoxSolver);

    let root_path = std::env::current_dir()
        .unwrap()
        .join("test_programs")
        .join(directory)
        .canonicalize()
        .expect("Could not resolve root path");
    let noir_text_document = Url::from_file_path(root_path.join("src/main.nr").as_path())
        .expect("Could not convert text document path to URI");
    let root_uri =
        Some(Url::from_file_path(root_path.as_path()).expect("Could not convert root path to URI"));

    #[allow(deprecated)]
    let initialize_params = InitializeParams {
        process_id: Default::default(),
        root_path: None,
        root_uri,
        initialization_options: None,
        capabilities: Default::default(),
        trace: Some(async_lsp::lsp_types::TraceValue::Verbose),
        workspace_folders: None,
        client_info: None,
        locale: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let _initialize_response = crate::requests::on_initialize(&mut state, initialize_params)
        .await
        .expect("Could not initialize LSP server");

    (state, noir_text_document)
}

/// Boots the LSP server against the on-disk workspace at `test_programs/<workspace_directory>`,
/// then opens `relative_file_path` (resolved against the workspace root) with `src` as its
/// contents. The override sticks for the duration of the test, so on-disk contents of that
/// file are irrelevant — the workspace directory is only needed as a Nargo root and to supply
/// dependency crates the test refers to (e.g. `one`, `std`).
///
/// Returns the LSP state and the opened file's URI.
pub(crate) async fn init_lsp_server_with_inline_source(
    workspace_directory: &str,
    relative_file_path: &str,
    src: &str,
) -> (LspState, Url) {
    let (mut state, root_marker_uri) = init_lsp_server(workspace_directory).await;

    // `init_lsp_server` returns a URI pointing at `<workspace>/src/main.nr` regardless of layout;
    // step up to the workspace root, then descend to the file we actually want to open.
    let workspace_dir =
        root_marker_uri.to_file_path().unwrap().parent().unwrap().parent().unwrap().to_path_buf();

    let file_uri = Url::from_file_path(workspace_dir.join(relative_file_path)).unwrap();

    let _ = on_did_open_text_document(
        &mut state,
        DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: file_uri.clone(),
                language_id: "noir".to_string(),
                version: 0,
                text: src.to_string(),
            },
        },
    );

    (state, file_uri)
}

/// Like `init_lsp_server_with_inline_source`, but `src` is expected to contain exactly one
/// `>|<` cursor marker. The marker is stripped before the document is opened, and its
/// position is returned alongside the cleaned source so the caller can issue a request at
/// the cursor and (e.g.) apply text edits against the cleaned source.
pub(crate) async fn init_lsp_server_with_inline_source_and_cursor(
    workspace_directory: &str,
    relative_file_path: &str,
    src: &str,
) -> (LspState, Url, Position, String) {
    let (line, column, src) = crate::utils::get_cursor_line_and_column(src);
    let (state, file_uri) =
        init_lsp_server_with_inline_source(workspace_directory, relative_file_path, &src).await;
    let position = Position { line: line as u32, character: column as u32 };
    (state, file_uri, position, src)
}

/// Searches for all instances of `search_string` in file `file_name` and returns a list of their locations.
pub(crate) fn search_in_file(filename: &str, search_string: &str) -> Vec<Range> {
    let file_contents = std::fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Couldn't read file {filename}"));
    let file_lines: Vec<&str> = file_contents.lines().collect();
    file_lines
        .iter()
        .enumerate()
        .flat_map(|(line_num, line)| {
            line.match_indices(search_string).map(move |(index, _)| {
                let start = Position { line: line_num as u32, character: index as u32 };
                let end = Position {
                    line: line_num as u32,
                    character: (index + search_string.len()) as u32,
                };
                Range { start, end }
            })
        })
        .collect()
}
