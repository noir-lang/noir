use crate::{
    LspState,
    types::{NargoStdSourceCodeParams, NargoStdSourceCodeResult},
};
use async_lsp::{ErrorCode, ResponseError};
use noirc_driver::stdlib_paths_with_source;

pub(crate) fn on_std_source_code_request(
    _state: &mut LspState,
    params: NargoStdSourceCodeParams,
) -> Result<NargoStdSourceCodeResult, ResponseError> {
    let path = format!("{}{}", params.uri.host_str().unwrap(), params.uri.path());
    for (std_path, source) in stdlib_paths_with_source() {
        if std_path == path {
            return Ok(source);
        }
    }
    Err(ResponseError::new(ErrorCode::REQUEST_FAILED, "File not found".to_string()))
}
