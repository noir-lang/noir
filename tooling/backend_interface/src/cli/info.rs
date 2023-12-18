use acvm::Language;
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
    #[allow(dead_code)]
    #[deprecated(note = "This field is deprecated and will be removed in the future")]
    opcodes_supported: Vec<String>,
    #[allow(dead_code)]
    #[deprecated(note = "This field is deprecated and will be removed in the future")]
    black_box_functions_supported: Vec<String>,
}

#[derive(Deserialize)]
struct LanguageResponse {
    name: String,
    width: Option<usize>,
}

impl InfoCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<Language, BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command.arg("info").arg("-c").arg(self.crs_path).arg("-o").arg("-");

        let output = command.output()?;

        if !output.status.success() {
            return Err(BackendError::CommandFailed(string_from_stderr(&output.stderr)));
        }

        let backend_info: InfoResponse =
            serde_json::from_slice(&output.stdout).expect("Backend should return valid json");
        let language: Language = match backend_info.language.name.as_str() {
            "PLONK-CSAT" => {
                let width = backend_info.language.width.unwrap();
                Language::PLONKCSat { width }
            }
            "R1CS" => Language::R1CS,
            _ => panic!("Unknown langauge"),
        };

        Ok(language)
    }
}

#[test]
fn info_command() -> Result<(), BackendError> {
    let backend = crate::get_mock_backend()?;
    let crs_path = backend.backend_directory();

    let language = InfoCommand { crs_path }.run(backend.binary_path())?;

    assert!(matches!(language, Language::PLONKCSat { width: 3 }));

    Ok(())
}
