use crate::LspState;
use acvm::blackbox_solver::StubbedBlackBoxSolver;
use async_lsp::ClientSocket;
use lsp_types::Url;

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
