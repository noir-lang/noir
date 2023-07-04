use clap::{Args, Parser, Subcommand};
use const_format::formatcp;
use noirc_driver::CompileOptions;
use std::path::{Path, PathBuf};

use color_eyre::eyre;

use crate::find_package_root;

mod fs;

mod check_cmd;
mod codegen_verifier_cmd;
mod compile_cmd;
mod execute_cmd;
mod gates_cmd;
mod lsp_cmd;
mod new_cmd;
mod prove_cmd;
mod test_cmd;
mod verify_cmd;

const GIT_HASH: &str = env!("GIT_COMMIT");
const IS_DIRTY: &str = env!("GIT_DIRTY");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

static VERSION_STRING: &str =
    formatcp!("{} (git version hash: {}, is dirty: {})", CARGO_PKG_VERSION, GIT_HASH, IS_DIRTY);

#[derive(Parser, Debug)]
#[command(name="nargo", author, version=VERSION_STRING, about, long_about = None)]
struct NargoCli {
    #[command(subcommand)]
    command: NargoCommand,

    #[clap(flatten)]
    config: NargoConfig,
}

#[non_exhaustive]
#[derive(Args, Clone, Debug)]
pub(crate) struct NargoConfig {
    #[arg(short, long, hide=true, default_value_os_t = std::env::current_dir().unwrap())]
    program_dir: PathBuf,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum NargoCommand {
    Check(check_cmd::CheckCommand),
    CodegenVerifier(codegen_verifier_cmd::CodegenVerifierCommand),
    Compile(compile_cmd::CompileCommand),
    New(new_cmd::NewCommand),
    Execute(execute_cmd::ExecuteCommand),
    Prove(prove_cmd::ProveCommand),
    Verify(verify_cmd::VerifyCommand),
    Test(test_cmd::TestCommand),
    Gates(gates_cmd::GatesCommand),
    Lsp(lsp_cmd::LspCommand),
}

pub fn start_cli() -> eyre::Result<()> {
    let NargoCli { command, mut config } = NargoCli::parse();

    // Search through parent directories to find package root if necessary.
    if !matches!(command, NargoCommand::New(_) | NargoCommand::Lsp(_)) {
        config.program_dir = find_package_root(&config.program_dir)?;
    }

    let backend = crate::backends::ConcreteBackend::default();

    match command {
        NargoCommand::New(args) => new_cmd::run(&backend, args, config),
        NargoCommand::Check(args) => check_cmd::run(&backend, args, config),
        NargoCommand::Compile(args) => compile_cmd::run(&backend, args, config),
        NargoCommand::Execute(args) => execute_cmd::run(&backend, args, config),
        NargoCommand::Prove(args) => prove_cmd::run(&backend, args, config),
        NargoCommand::Verify(args) => verify_cmd::run(&backend, args, config),
        NargoCommand::Test(args) => test_cmd::run(&backend, args, config),
        NargoCommand::Gates(args) => gates_cmd::run(&backend, args, config),
        NargoCommand::CodegenVerifier(args) => codegen_verifier_cmd::run(&backend, args, config),
        NargoCommand::Lsp(args) => lsp_cmd::run(&backend, args, config),
    }?;

    Ok(())
}

// helper function which tests noir programs by trying to generate a proof and verify it without reading/writing to the filesystem
pub fn prove_and_verify(program_dir: &Path, experimental_ssa: bool) -> bool {
    use compile_cmd::compile_circuit;
    use fs::common_reference_string::update_common_reference_string;
    use nargo::ops::preprocess_program;

    let backend = crate::backends::ConcreteBackend::default();

    let compile_options = CompileOptions {
        show_ssa: false,
        print_acir: false,
        deny_warnings: false,
        show_output: false,
        experimental_ssa,
    };

    let program =
        compile_circuit(&backend, program_dir, &compile_options).expect("Compile should succeed");
    let common_reference_string = update_common_reference_string(
        &backend,
        // Empty CRS is always used since we don't read/write a cached version in these tests
        &[],
        &program.circuit,
    )
    .expect("Should fetch CRS");
    let preprocessed_program = preprocess_program(&backend, &common_reference_string, program)
        .expect("Preprocess should succeed");

    let nargo::artifacts::program::PreprocessedProgram {
        abi,
        bytecode,
        proving_key,
        verification_key,
        ..
    } = preprocessed_program;

    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) = fs::inputs::read_inputs_from_file(
        program_dir,
        crate::constants::PROVER_INPUT_FILE,
        noirc_abi::input_parser::Format::Toml,
        &abi,
    )
    .expect("Should read inputs");

    let solved_witness =
        match execute_cmd::execute_program(&backend, bytecode.clone(), &abi, &inputs_map) {
            Ok(witness) => witness,
            // Failure to execute is an invalid proof
            Err(_) => return false,
        };

    let public_abi = abi.public_abi();
    let (public_inputs, return_value) =
        public_abi.decode(&solved_witness).expect("Solved witness should decode");

    let proof = nargo::ops::prove_execution(
        &backend,
        &common_reference_string,
        &bytecode,
        solved_witness,
        &proving_key,
    )
    .expect("Circuit should prove");

    let public_inputs =
        public_abi.encode(&public_inputs, return_value).expect("Public inputs should encode");
    nargo::ops::verify_proof(
        &backend,
        &common_reference_string,
        &bytecode,
        &proof,
        public_inputs,
        &verification_key,
    )
    .expect("Proof should verify")
}

// FIXME: I not sure that this is the right place for this tests.
#[cfg(test)]
mod tests {
    use noirc_driver::Driver;
    use noirc_errors::reporter;
    use noirc_frontend::graph::CrateType;

    use std::path::{Path, PathBuf};

    const TEST_DATA_DIR: &str = "tests/compile_tests_data";

    /// Compiles a file and returns true if compilation was successful
    ///
    /// This is used for tests.
    fn file_compiles<P: AsRef<Path>>(root_file: P) -> bool {
        let mut driver = Driver::new(
            &acvm::Language::R1CS,
            #[allow(deprecated)]
            Box::new(acvm::pwg::default_is_opcode_supported(acvm::Language::R1CS)),
        );
        driver.create_local_crate(&root_file, CrateType::Binary);

        let result = driver.check_crate(false, false);
        let success = result.is_ok();

        let errors = match result {
            Ok(warnings) => warnings,
            Err(errors) => errors,
        };

        reporter::report_all(driver.file_manager(), &errors, false);
        success
    }

    #[test]
    fn compilation_pass() {
        let pass_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/pass"));

        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(file_compiles(&path), "path: {}", path.display());
        }
    }

    #[test]
    fn compilation_fail() {
        let fail_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/fail"));

        let paths = std::fs::read_dir(fail_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(!file_compiles(&path), "path: {}", path.display());
        }
    }
}
