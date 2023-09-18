use acvm::Language;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::{BackendError, BackendOpcodeSupport};

pub(crate) struct InfoCommand {
    pub(crate) crs_path: PathBuf,
}

#[derive(Deserialize)]
struct InfoResponse {
    language: LanguageResponse,
    opcodes_supported: Vec<String>,
    black_box_functions_supported: Vec<String>,
}

#[derive(Deserialize)]
struct LanguageResponse {
    name: String,
    width: Option<usize>,
}

impl BackendOpcodeSupport {
    fn new(info: InfoResponse) -> Self {
        let opcodes: HashSet<String> = info.opcodes_supported.into_iter().collect();
        let black_box_functions: HashSet<String> =
            info.black_box_functions_supported.into_iter().collect();
        Self { opcodes, black_box_functions }
    }
}

impl InfoCommand {
    pub(crate) fn run(
        self,
        binary_path: &Path,
    ) -> Result<(Language, BackendOpcodeSupport), BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command.arg("info").arg("-c").arg(self.crs_path).arg("-o").arg("-");

        let output = command.output()?;

        if !output.status.success() {
            return Err(BackendError::CommandFailed(output.stderr));
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

        Ok((language, BackendOpcodeSupport::new(backend_info)))
    }
}

#[test]
fn info_command() -> Result<(), BackendError> {
    use acvm::acir::circuit::opcodes::Opcode;

    use acvm::acir::native_types::Expression;

    let backend = crate::get_mock_backend()?;
    let crs_path = backend.backend_directory();

    let (language, opcode_support) = InfoCommand { crs_path }.run(backend.binary_path())?;

    assert!(matches!(language, Language::PLONKCSat { width: 3 }));
    assert!(opcode_support.is_opcode_supported(&Opcode::Arithmetic(Expression::default())));

    Ok(())
}
