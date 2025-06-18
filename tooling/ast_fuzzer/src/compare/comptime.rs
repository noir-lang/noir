//! Compare an arbitrary AST executed as Noir with the comptime
//! interpreter vs compiled into bytecode and ran through a VM.
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{cell::RefCell, collections::BTreeMap};

use arbitrary::Unstructured;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use color_eyre::eyre::{self, WrapErr};
use nargo::NargoError;
use nargo::errors::ExecutionError;
use nargo::{foreign_calls::DefaultForeignCallBuilder, parse_all};
use noirc_abi::Abi;
use noirc_driver::{
    CompilationResult, CompileOptions, CompiledProgram, CrateId, compile_main,
    file_manager_with_stdlib, prepare_crate,
};
use noirc_errors::Location;
use noirc_evaluator::ssa::SsaProgramArtifact;
use noirc_frontend::{
    hir::Context,
    monomorphization::{
        Monomorphizer,
        ast::{Expression, Program},
        debug_types::DebugTypeTracker,
    },
};

use super::{CompareArtifact, CompareCompiledResult, CompareOptions, HasPrograms};
use crate::{
    Config, DisplayAstAsNoirComptime, arb_program_comptime, program_abi, program_wrap_expression,
};

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
) -> (CompilationResult<CompiledProgram>, W) {
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
    let res = compile_main(&mut context, root_crate_id, &options, None);
    drop(context);
    let output = Rc::into_inner(output).expect("context is gone").into_inner();
    (res, output)
}

/// Interpret source code using the elaborator, without
/// parsing it with nargo.
fn interpret(src: &str) -> Expression {
    use fm::{FileId, FileManager};
    use noirc_frontend::elaborator::{Elaborator, ElaboratorOptions};
    use noirc_frontend::hir::def_collector::dc_crate::DefCollector;
    use noirc_frontend::hir::def_collector::dc_mod::collect_defs;
    use noirc_frontend::hir::def_map::{CrateDefMap, ModuleData};
    use noirc_frontend::hir::{Context, ParsedFiles};
    use noirc_frontend::parse_program;

    let file = FileId::default();

    let location = Location::new(Default::default(), file);
    let root_module = ModuleData::new(
        None,
        location,
        Vec::new(),
        Vec::new(),
        false, // is contract
        false, // is struct
    );

    let file_manager = FileManager::new(&PathBuf::new());
    let parsed_files = ParsedFiles::new();
    let mut context = Context::new(file_manager, parsed_files);
    context.def_interner.populate_dummy_operator_traits();

    let krate = context.crate_graph.add_crate_root(FileId::dummy());

    let (module, errors) = parse_program(src, file);
    // Skip warnings
    let errors: Vec<_> = errors.iter().filter(|e| !e.is_warning()).collect();
    assert_eq!(errors.len(), 0);
    let ast = module.into_sorted();

    let def_map = CrateDefMap::new(krate, root_module);
    let root_module_id = def_map.root();
    let mut collector = DefCollector::new(def_map);

    collect_defs(&mut collector, ast, FileId::dummy(), root_module_id, krate, &mut context);
    context.def_maps.insert(krate, collector.def_map);

    let main = context.get_main_function(&krate).expect("Expected 'main' function");

    let mut elaborator = Elaborator::elaborate_and_return_self(
        &mut context,
        krate,
        collector.items,
        ElaboratorOptions::test_default(),
    );

    // Skip compiler warnings
    let errors: Vec<_> = elaborator.errors.iter().filter(|&e| e.is_error()).cloned().collect();
    if !errors.is_empty() {
        log::debug!("elaborator errors: {:?}", errors);
    }
    assert_eq!(errors.len(), 0);

    let mut interpreter = elaborator.setup_interpreter();

    // The most straightforward way to convert the interpreter result into
    // an acceptable monomorphized AST expression seems to be converting it
    // into HIR first and then processing it with the monomorphizer
    let expr_id =
        match interpreter.call_function(main, Vec::new(), Default::default(), Location::dummy()) {
            Err(e) => panic!("interpreter error: {:?}", e),
            Ok(value) => match value.into_hir_expression(elaborator.interner, Location::dummy()) {
                Err(e) => panic!("could not convert interpreter result into HIR: {:?}", e),
                Ok(expr_id) => expr_id,
            },
        };

    let mut monomorphizer = Monomorphizer::new(elaborator.interner, DebugTypeTracker::default());
    monomorphizer.expr(expr_id).expect("monomorphization error while converting interpreter execution result, should not be possible")
}

/// Compare the execution of a Noir program in pure comptime (via interpreter)
/// vs normal SSA execution.
pub struct CompareComptime {
    pub program: Program,
    pub abi: Abi,
    pub source: String,
    pub ssa: CompareArtifact,
    pub force_brillig: bool,
    /// If the comptime code is executed directly, its results
    /// are wrapped in a `main` function returning them, for further comparison.
    pub comptime_ssa: Option<CompareArtifact>,
}

impl CompareComptime {
    /// Execute the Noir code passed through the interpreter
    /// and the SSA, then compare the results.
    pub fn exec_direct(&self) -> eyre::Result<CompareCompiledResult> {
        let comptime_ssa = match &self.comptime_ssa {
            Some(comptime_ssa) => comptime_ssa,
            None => unreachable!("SSA returning the comptime execution result should be available"),
        };

        let blackbox_solver = Bn254BlackBoxSolver(false);
        let initial_witness = self.abi.encode(&BTreeMap::new(), None).wrap_err("abi::encode")?;

        let do_exec = |program| {
            let mut print = Vec::new();

            let mut foreign_call_executor = DefaultForeignCallBuilder::default()
                .with_mocks(false)
                .with_output(&mut print)
                .build();

            nargo::ops::execute_program(
                program,
                initial_witness.clone(),
                &blackbox_solver,
                &mut foreign_call_executor,
            )
        };

        let res1 = do_exec(&comptime_ssa.artifact.program);
        let res2 = do_exec(&self.ssa.artifact.program);

        CompareCompiledResult::new(
            &self.abi,
            &Default::default(),
            &self.ssa.artifact.error_types,
            (res1, "".into()),
            (res2, "".into()),
        )
    }

    /// Execute the Noir code (via nargo) and the SSA, then compare the results.
    pub fn exec(&self) -> eyre::Result<CompareCompiledResult> {
        let blackbox_solver = Bn254BlackBoxSolver(false);

        // These comptime programs have no inputs.
        let initial_witness = self.abi.encode(&BTreeMap::new(), None).wrap_err("abi::encode")?;

        let decode_print = |print| String::from_utf8(print).expect("should be valid utf8 string");

        // Execute a compiled Program.
        let do_exec = |program| {
            let mut output = Vec::new();

            let mut foreign_call_executor = DefaultForeignCallBuilder::default()
                .with_mocks(false)
                .with_output(&mut output)
                .build();

            let res = nargo::ops::execute_program(
                program,
                initial_witness.clone(),
                &blackbox_solver,
                &mut foreign_call_executor,
            );
            let print = decode_print(output);

            (res, print)
        };

        // Execute the 2nd (Brillig) program.
        let (res2, print2) = do_exec(&self.ssa.artifact.program);

        // Try to compile the 1st (comptime) version from string.
        log::debug!("comptime src:\n{}", self.source);
        let (program1, output1) = match prepare_and_compile_snippet(
            self.source.clone(),
            self.force_brillig,
            Vec::new(),
        ) {
            (Ok((program, _)), output) => (program, output),
            (Err(e), output) => {
                // If the comptime code failed to compile, it could be because it executed the code
                // and encountered an overflow, which would be a runtime error in Brillig.
                let is_assertion = e.iter().any(|e| {
                    e.secondaries.iter().any(|s| s.message == "Assertion failed")
                        || e.message.contains("overflow")
                        || e.message.contains("divide by zero")
                });
                if is_assertion {
                    let msg = format!("{e:?}");
                    let err = ExecutionError::AssertionFailed(
                        acvm::pwg::ResolvedAssertionPayload::String(msg),
                        vec![],
                        None,
                    );
                    let res1 = Err(NargoError::ExecutionError(err));
                    let print1 = decode_print(output);
                    return CompareCompiledResult::new(
                        &self.abi,
                        &Default::default(), // We failed to compile the program, so no error types.
                        &self.ssa.artifact.error_types,
                        (res1, print1),
                        (res2, print2),
                    );
                } else {
                    panic!("failed to compile program:\n{e:?}\n{}", self.source);
                }
            }
        };
        // Capture any println that happened during the compilation, which in these tests should be the whole program.
        let comptime_print = String::from_utf8(output1).expect("should be valid utf8 string");

        // Execute the 1st (comptime) program.
        let (res1, print1) = do_exec(&program1.program);

        CompareCompiledResult::new(
            &self.abi,
            &Default::default(), // We have a fully compiled program at this point, no access to the SSA error types, just ABI error types.
            &self.ssa.artifact.error_types,
            (res1, comptime_print + &print1),
            (res2, print2),
        )
    }

    /// Generate a random comptime-viable AST, reverse it into
    /// Noir code and also compile it into SSA.
    pub fn arb(
        u: &mut Unstructured,
        c: Config,
        f: impl FnOnce(Program) -> arbitrary::Result<(SsaProgramArtifact, CompareOptions)>,
    ) -> arbitrary::Result<Self> {
        let force_brillig = c.force_brillig;
        let program = arb_program_comptime(u, c)?;
        let abi = program_abi(&program);

        let ssa = CompareArtifact::from(f(program.clone())?);

        let source = format!("{}", DisplayAstAsNoirComptime(&program));

        Ok(Self { program, abi, source, ssa, force_brillig, comptime_ssa: None })
    }

    /// Generate a random comptime-viable AST, reverse it into
    /// Noir code and also compile it into SSA.
    /// Then, execute the resulting code with the comptime
    /// interpreter and prepare SSA returning the result
    /// literal for comparison.
    pub fn arb_direct(
        u: &mut Unstructured,
        c: Config,
        f: impl FnOnce(Program) -> arbitrary::Result<(SsaProgramArtifact, CompareOptions)>,
        f_comptime: impl FnOnce(Program) -> arbitrary::Result<(SsaProgramArtifact, CompareOptions)>,
    ) -> arbitrary::Result<Self> {
        let force_brillig = c.force_brillig;
        let program = arb_program_comptime(u, c.clone())?;
        let abi = program_abi(&program);

        let ssa = CompareArtifact::from(f(program.clone())?);

        let source = format!("{}", DisplayAstAsNoirComptime(&program));
        // log source code before interpreting
        log::debug!("comptime src:\n{}", source);

        let comptime_res = interpret(&format!("comptime {}", source));

        let program_comptime = program_wrap_expression(c, comptime_res)?;
        let comptime_ssa = Some(CompareArtifact::from(f_comptime(program_comptime)?));

        Ok(Self { program, abi, source, ssa, force_brillig, comptime_ssa })
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
