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

/// Parses a source string with `>|<` cursor and `[[...]]` target-range markers. Both
/// markers are stripped from the returned source. Positions are in UTF-16 code units, the
/// LSP wire format. Panics if the source does not contain exactly one of each marker pair.
///
/// Useful for goto-style tests where you want to show, inline in the source, both *where*
/// the request is invoked and *what* the response should point at.
pub(crate) fn parse_cursor_and_target_marker(src: &str) -> (String, Position, Range) {
    let mut clean = String::new();
    let mut line = 0u32;
    let mut character = 0u32;
    let mut cursor: Option<Position> = None;
    let mut target_start: Option<Position> = None;
    let mut target_end: Option<Position> = None;
    let mut chars = src.chars().peekable();

    while let Some(ch) = chars.next() {
        let two = chars.peek().copied();
        match ch {
            '>' if two == Some('|') => {
                // Look ahead for `>|<`
                chars.next();
                if chars.next() != Some('<') {
                    panic!("Found `>|` not followed by `<` while parsing markers");
                }
                if cursor.is_some() {
                    panic!("Multiple `>|<` cursors in source");
                }
                cursor = Some(Position { line, character });
            }
            '[' if two == Some('[') => {
                chars.next();
                if target_start.is_some() {
                    panic!("Multiple `[[` target-start markers in source");
                }
                target_start = Some(Position { line, character });
            }
            ']' if two == Some(']') => {
                chars.next();
                if target_end.is_some() {
                    panic!("Multiple `]]` target-end markers in source");
                }
                target_end = Some(Position { line, character });
            }
            '\n' => {
                clean.push('\n');
                line += 1;
                character = 0;
            }
            ch => {
                clean.push(ch);
                character += ch.len_utf16() as u32;
            }
        }
    }

    let cursor = cursor.expect("Expected exactly one `>|<` cursor marker");
    let start = target_start.expect("Expected `[[` to open the target range");
    let end = target_end.expect("Expected `]]` to close the target range");
    (clean, cursor, Range { start, end })
}

/// Returns the substring of `text` covered by `range`. Positions are interpreted in
/// UTF-16 code units (the LSP wire format), which means ASCII source code works
/// naturally and BMP characters like `é` still count as one column.
///
/// Tests use this to assert against the actual text a range covers ("`SomeStruct`")
/// instead of bare line/column numbers, which forces the reader to count characters.
pub(crate) fn text_at(text: &str, range: Range) -> String {
    let mut out = String::new();
    for (line_idx, line) in text.lines().enumerate() {
        let line_idx = line_idx as u32;
        if line_idx < range.start.line || line_idx > range.end.line {
            continue;
        }

        // Map a UTF-16 column position to a byte index within `line`.
        let to_byte_idx = |target_u16: u32| -> usize {
            let mut u16_idx = 0u32;
            for (byte_idx, ch) in line.char_indices() {
                if u16_idx >= target_u16 {
                    return byte_idx;
                }
                u16_idx += ch.len_utf16() as u32;
            }
            line.len()
        };

        let start_byte =
            if line_idx == range.start.line { to_byte_idx(range.start.character) } else { 0 };
        let end_byte =
            if line_idx == range.end.line { to_byte_idx(range.end.character) } else { line.len() };
        out.push_str(&line[start_byte..end_byte]);
        if line_idx < range.end.line {
            out.push('\n');
        }
    }
    out
}

/// Searches for all instances of `search_string` in `text` and returns a list of their locations.
pub(crate) fn search_in_text(text: &str, search_string: &str) -> Vec<Range> {
    text.lines()
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
