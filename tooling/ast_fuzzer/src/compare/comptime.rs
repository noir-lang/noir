use std::collections::BTreeMap;
use std::path::Path;

use arbitrary::Unstructured;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use color_eyre::eyre::{self, WrapErr};
use nargo::{foreign_calls::DefaultForeignCallBuilder, parse_all};
use noirc_abi::Abi;
use noirc_driver::{
    CompilationResult, CompileOptions, CompiledProgram, CrateId, compile_main,
    file_manager_with_stdlib, prepare_crate,
};
use noirc_evaluator::ssa::SsaProgramArtifact;
use noirc_frontend::{hir::Context, monomorphization::ast::Program};

use crate::{
    Config, DisplayAstAsNoirComptime, arb_program_comptime, compare::CompareResult, program_abi,
};

use super::{CompareArtifact, CompareOptions, HasPrograms};

/// Prepare a code snippet.
/// (copied from nargo_cli/tests/common.rs)
fn prepare_snippet(source: String) -> (Context<'static, 'static>, CrateId) {
    let root = Path::new("");
    let file_name = Path::new("main.nr");
    let mut file_manager = file_manager_with_stdlib(root);
    file_manager.add_file_with_source(file_name, source).expect(
        "Adding source buffer to file manager should never fail when file manager is empty",
    );
    let parsed_files = parse_all(&file_manager);

    let mut context = Context::new(file_manager, parsed_files);
    let root_crate_id = prepare_crate(&mut context, file_name);

    (context, root_crate_id)
}

/// Compile the main function in a code snippet.
///
/// Use `force_brillig` to test it as an unconstrained function without having to change the code.
/// This is useful for methods that use the `runtime::is_unconstrained()` method to change their behavior.
/// (copied from nargo_cli/tests/common.rs)
fn prepare_and_compile_snippet(
    source: String,
    force_brillig: bool,
) -> CompilationResult<CompiledProgram> {
    let (mut context, root_crate_id) = prepare_snippet(source);
    let options = CompileOptions {
        force_brillig,
        silence_warnings: true,
        skip_underconstrained_check: true,
        skip_brillig_constraints_check: true,
        ..Default::default()
    };
    compile_main(&mut context, root_crate_id, &options, None)
}

/// Compare the execution of a Noir program in pure comptime (via interpreter)
/// vs normal SSA execution.
pub struct CompareComptime {
    pub program: Program,
    pub abi: Abi,
    pub source: String,
    pub ssa: CompareArtifact,
    pub force_brillig: bool,
}

impl CompareComptime {
    /// Execute the Noir code and the SSA, then compare the results.
    pub fn exec(&self) -> eyre::Result<CompareResult> {
        let program1 = match prepare_and_compile_snippet(self.source.clone(), self.force_brillig) {
            Ok((program, _)) => program,
            Err(e) => panic!("failed to compile program:\n{}\n{e:?}", self.source),
        };

        let blackbox_solver = Bn254BlackBoxSolver(false);
        let initial_witness = self.abi.encode(&BTreeMap::new(), None).wrap_err("abi::encode")?;

        let do_exec = |program| {
            let mut print = Vec::new();

            let mut foreign_call_executor = DefaultForeignCallBuilder::default()
                .with_mocks(false)
                .with_output(&mut print)
                .build();

            let res = nargo::ops::execute_program(
                program,
                initial_witness.clone(),
                &blackbox_solver,
                &mut foreign_call_executor,
            );

            let print = String::from_utf8(print).expect("should be valid utf8 string");
            (res, print)
        };

        let (res1, print1) = do_exec(&program1.program);
        let (res2, print2) = do_exec(&self.ssa.artifact.program);

        CompareResult::new(&self.abi, (res1, print1), (res2, print2))
    }

    /// Generate a random comptime-viable AST, reverse it into
    /// Noir code and also compile it into SSA.
    pub fn arb(
        u: &mut Unstructured,
        c: Config,
        f: impl FnOnce(
            &mut Unstructured,
            Program,
        ) -> arbitrary::Result<(SsaProgramArtifact, CompareOptions)>,
    ) -> arbitrary::Result<Self> {
        let force_brillig = c.force_brillig;
        let program = arb_program_comptime(u, c)?;
        let abi = program_abi(&program);

        let ssa = CompareArtifact::from(f(u, program.clone())?);

        let source = format!("{}", DisplayAstAsNoirComptime(&program));

        Ok(Self { program, abi, source, ssa, force_brillig })
    }
}

impl HasPrograms for CompareComptime {
    fn programs(&self) -> Vec<&Program> {
        vec![&self.program]
    }
}
