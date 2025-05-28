//! Compare an arbitrary AST executed as Noir with the comptime
//! interpreter vs compiled into bytecode and ran through a VM.
use std::path::Path;
use std::rc::Rc;
use std::{cell::RefCell, collections::BTreeMap};

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

use super::{CompareArtifact, CompareCompiledResult, CompareOptions, HasPrograms};
use crate::{Config, DisplayAstAsNoirComptime, arb_program_comptime, program_abi};

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
fn prepare_and_compile_snippet<W: std::io::Write + 'static>(
    source: String,
    force_brillig: bool,
    output: W,
) -> CompilationResult<(CompiledProgram, W)> {
    let output = Rc::new(RefCell::new(output));
    let (mut context, root_crate_id) = prepare_snippet(source);
    context.set_comptime_printing(output.clone());
    let options = CompileOptions {
        force_brillig,
        silence_warnings: true,
        skip_underconstrained_check: true,
        skip_brillig_constraints_check: true,
        ..Default::default()
    };
    let (program, warnings) = compile_main(&mut context, root_crate_id, &options, None)?;
    drop(context);
    let output = Rc::into_inner(output).expect("context is gone").into_inner();
    Ok(((program, output), warnings))
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
    pub fn exec(&self) -> eyre::Result<CompareCompiledResult> {
        log::debug!("comptime src:\n{}", self.source);
        let (program1, output1) = match prepare_and_compile_snippet(
            self.source.clone(),
            self.force_brillig,
            Vec::new(),
        ) {
            Ok(((program, output), _)) => (program, output),
            Err(e) => panic!("failed to compile program:\n{}\n{e:?}", self.source),
        };
        let comptime_print = String::from_utf8(output1).expect("should be valid utf8 string");

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

        CompareCompiledResult::new(&self.abi, (res1, comptime_print + &print1), (res2, print2))
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

#[cfg(test)]
mod tests {
    use super::prepare_and_compile_snippet;

    /// Comptime compilation can fail with stack overflow because of how the interpreter is evaluating instructions.
    /// We could apply `#[inline(always)]` on some of the `Interpreter::elaborate_` functions to make it go further,
    /// at the cost of compilation speed. Instead, we just need to make sure that compile tests have lower loop and
    /// recursion limits.
    #[test]
    fn test_prepare_and_compile_snippet() {
        let src = r#"
fn main() -> pub ((str<2>, str<2>, bool, str<2>), bool, [str<2>; 3]) {
    comptime {
        let mut ctx_limit = 10;
        unsafe { func_1_proxy(ctx_limit) }
    }
}
unconstrained fn func_1(ctx_limit: &mut u32) -> ((str<2>, str<2>, bool, str<2>), bool, [str<2>; 3]) {
    if ((*ctx_limit) == 0) {
        (("BD", "GT", false, "EV"), false, ["LJ", "BB", "CE"])
    } else {
        *ctx_limit = ((*ctx_limit) - 1);
        let g = if true {
            let f = {
                {
                    let mut idx_a = 0;
                    loop {
                        if (idx_a == 4) {
                            break
                        } else {
                            idx_a = (idx_a + 1);
                            let mut e = if true {
                                if func_1(ctx_limit).0.2 {
                                    {
                                        let b = 38;
                                        {
                                            let mut c = false;
                                            let d = if c {
                                                c = false;
                                                [("ZO", "AF", false, "NY"), ("HJ", "NF", c, "RV"), ("SN", "VK", true, "QJ")]
                                            } else {
                                                [("SN", "YR", (b != (27 >> b)), "LS"), ("SW", "ZQ", false, "TQ"), ("AD", "YD", c, "EF")]
                                            };
                                            (d[1].3, d[1].1, (!c), "LO")
                                        }
                                    }
                                } else {
                                    ("HF", "WQ", true, "FZ")
                                }
                            } else {
                                ("YP", "CH", true, "ZG")
                            };
                            e = (e.1, "NU", e.2, e.0);
                        }
                    }
                };
                true
            };
            (("HW", "EI", true, "IY"), (!false), ["TO", "WI", "PC"])
        } else {
            (("OX", "CE", true, "OV"), false, ["OS", "DT", "CH"])
        };
        g
    }
}
unconstrained fn func_1_proxy(mut ctx_limit: u32) -> ((str<2>, str<2>, bool, str<2>), bool, [str<2>; 3]) {
    func_1((&mut ctx_limit))
}
        "#;

        let _ = prepare_and_compile_snippet(src.to_string(), false, std::io::stdout());
    }
}
