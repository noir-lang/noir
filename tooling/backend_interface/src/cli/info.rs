use acvm::acir::circuit::ExpressionWidth;

use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::BackendError;

use super::string_from_stderr;

pub(crate) struct InfoCommand {
    pub(crate) crs_path: PathBuf,
}

#[derive(Deserialize)]
struct InfoResponse {
    language: LanguageResponse,
}

#[derive(Deserialize)]
struct LanguageResponse {
    name: String,
    width: Option<usize>,
}

impl InfoCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<ExpressionWidth, BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command.arg("info").arg("-c").arg(self.crs_path).arg("-o").arg("-");

        let output = command.output()?;

        if !output.status.success() {
            return Err(BackendError::CommandFailed(string_from_stderr(&output.stderr)));
        }

        let backend_info: InfoResponse =
            serde_json::from_slice(&output.stdout).expect("Backend should return valid json");
        let expression_width: ExpressionWidth = match backend_info.language.name.as_str() {
            "PLONK-CSAT" => {
                let width = backend_info.language.width.unwrap();
                ExpressionWidth::Bounded { width }
            }
            "R1CS" => ExpressionWidth::Unbounded,
            _ => panic!("Unknown Expression width configuration"),
        };

        Ok(expression_width)
    }
}

#[test]
fn info_command() -> Result<(), BackendError> {
    let backend = crate::get_mock_backend()?;
    let crs_path = backend.backend_directory();

    let expression_width = InfoCommand { crs_path }.run(backend.binary_path())?;

    assert!(matches!(expression_width, ExpressionWidth::Bounded { width: 3 }));

    Ok(())
}
