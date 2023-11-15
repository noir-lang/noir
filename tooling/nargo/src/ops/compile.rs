use acvm::{acir::circuit::Opcode, Language};
use fm::FileManager;
use noirc_driver::{CompilationResult, CompileOptions, CompiledProgram};

use crate::prepare_package;
use crate::{package::Package, workspace::Workspace};

pub fn compile_program(
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> (FileManager, CompilationResult<CompiledProgram>) {
    let (mut context, crate_id) =
        prepare_package(package, Box::new(|path| std::fs::read_to_string(path)));

    let program_artifact_path = workspace.package_build_path(package);
    let mut debug_artifact_path = program_artifact_path.clone();
    debug_artifact_path.set_file_name(format!("debug_{}.json", package.name));

    let (program, warnings) =
        match noirc_driver::compile_main(&mut context, crate_id, compile_options, None, true) {
            Ok(program_and_warnings) => program_and_warnings,
            Err(errors) => {
                return (context.file_manager, Err(errors));
            }
        };

    // TODO: we say that pedersen hashing is supported by all backends for now
    let is_opcode_supported_pedersen_hash = |opcode: &Opcode| -> bool {
        if let Opcode::BlackBoxFuncCall(
            acvm::acir::circuit::opcodes::BlackBoxFuncCall::PedersenHash { .. },
        ) = opcode
        {
            true
        } else {
            is_opcode_supported(opcode)
        }
    };

    // Apply backend specific optimizations.
    let optimized_program =
        crate::ops::optimize_program(program, np_language, &is_opcode_supported_pedersen_hash)
            .expect("Backend does not support an opcode that is in the IR");

    (context.file_manager, Ok((optimized_program, warnings)))
}
