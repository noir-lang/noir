use crate::LspState;
use acvm::blackbox_solver::StubbedBlackBoxSolver;
use async_lsp::ClientSocket;
use lsp_types::{Position, Range, Url};

pub(crate) async fn init_lsp_server(directory: &str) -> (LspState, Url) {
    let client = ClientSocket::new_closed();
    let mut state = LspState::new(&client, StubbedBlackBoxSolver::default());

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
    let initialize_params = lsp_types::InitializeParams {
        process_id: Default::default(),
        root_path: None,
        root_uri,
        initialization_options: None,
        capabilities: Default::default(),
        trace: Some(lsp_types::TraceValue::Verbose),
        workspace_folders: None,
        client_info: None,
        locale: None,
    };

    let _initialize_response = crate::requests::on_initialize(&mut state, initialize_params)
        .await
        .expect("Could not initialize LSP server");

    (state, noir_text_document)
}

/// Searches for all instances of `search_string` in file `file_name` and returns a list of their locations.
pub(crate) fn search_in_file(filename: &str, search_string: &str) -> Vec<Range> {
    let file_contents = std::fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Couldn't read file {}", filename));
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
