use acvm::acir::circuit::opcodes::Opcode;
use acvm::Language;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

use crate::BackendError;

pub(crate) struct InfoCommand;

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

impl InfoCommand {
    pub(crate) fn run(
        self,
        binary_path: &Path,
    ) -> Result<(Language, Box<impl Fn(&Opcode) -> bool>), BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command.arg("info");

        let output = command.output().expect("Failed to execute command");

        if !output.status.success() {
            return Err(BackendError(String::from_utf8(output.stderr).unwrap()));
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

        let opcodes_set: HashSet<String> = backend_info.opcodes_supported.into_iter().collect();
        let black_box_functions_set: HashSet<String> =
            backend_info.black_box_functions_supported.into_iter().collect();

        let is_opcode_supported = move |opcode: &Opcode| -> bool {
            match opcode {
                Opcode::Arithmetic(_) => opcodes_set.contains("arithmetic"),
                Opcode::Directive(_) => opcodes_set.contains("directive"),
                Opcode::Brillig(_) => opcodes_set.contains("brillig"),
                Opcode::MemoryInit { .. } => opcodes_set.contains("memory_init"),
                Opcode::MemoryOp { .. } => opcodes_set.contains("memory_op"),
                Opcode::BlackBoxFuncCall(func) => {
                    black_box_functions_set.contains(func.get_black_box_func().name())
                }
            }
        };

        return Ok((language, Box::new(is_opcode_supported)));
    }
}

#[test]
#[serial_test::serial]
fn info_command() {
    use acvm::acir::circuit::black_box_functions::BlackBoxFunc;
    use acvm::acir::circuit::opcodes::{BlackBoxFuncCall, Opcode};

    use acvm::acir::native_types::Expression;

    let backend = crate::get_mock_backend();

    let (language, is_opcode_supported) = InfoCommand.run(&backend.binary_path()).unwrap();

    assert!(matches!(language, Language::PLONKCSat { width: 3 }));
    assert!(is_opcode_supported(&Opcode::Arithmetic(Expression::default())));

    assert!(!is_opcode_supported(&Opcode::BlackBoxFuncCall(
        #[allow(deprecated)]
        BlackBoxFuncCall::dummy(BlackBoxFunc::Keccak256)
    )));
}
