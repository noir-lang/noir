use std::collections::BTreeMap;
use std::path::Path;

use acir::{FieldElement, native_types::WitnessStack};
use acvm::pwg::{OpcodeResolutionError, ResolvedAssertionPayload};
use arbitrary::{Arbitrary, Unstructured};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use color_eyre::eyre::{self, WrapErr, bail};
use nargo::{
    NargoError, errors::ExecutionError, foreign_calls::DefaultForeignCallBuilder, parse_all,
};
use noirc_abi::{Abi, InputMap, input_parser::InputValue};
use noirc_driver::{
    CompilationResult, CompileOptions, CompiledProgram, CrateId, compile_main,
    file_manager_with_stdlib, prepare_crate,
};
use noirc_evaluator::ssa::{SsaEvaluatorOptions, SsaProgramArtifact};
use noirc_frontend::{hir::Context, monomorphization::ast::Program};

use crate::{
    Config, DisplayAstAsNoirComptime, arb_inputs, arb_program, arb_program_comptime, program_abi,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ExecOutput {
    pub return_value: Option<InputValue>,
    pub print_output: String,
}

type ExecResult = (Result<WitnessStack<FieldElement>, NargoError<FieldElement>>, String);

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
pub fn prepare_and_compile_snippet(
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

/// Subset of [SsaEvaluatorOptions] that we want to vary.
///
/// It exists to reduce noise in the printed results, compared to showing the full `SsaEvaluatorOptions`.
#[derive(Debug, Clone, Default)]
pub struct CompareOptions {
    pub inliner_aggressiveness: i64,
}

impl Arbitrary<'_> for CompareOptions {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(Self { inliner_aggressiveness: *u.choose(&[i64::MIN, 0, i64::MAX])? })
    }
}

impl CompareOptions {
    /// Copy fields into an [SsaEvaluatorOptions] instance.
    pub fn onto(&self, mut options: SsaEvaluatorOptions) -> SsaEvaluatorOptions {
        options.inliner_aggressiveness = self.inliner_aggressiveness;
        options
    }
}

/// Possible outcomes of the differential execution of two equivalent programs.
///
/// Use [CompareResult::return_value_or_err] to do the final comparison between
/// the execution result.
pub enum CompareResult {
    BothFailed(NargoError<FieldElement>, NargoError<FieldElement>),
    LeftFailed(NargoError<FieldElement>, ExecOutput),
    RightFailed(ExecOutput, NargoError<FieldElement>),
    BothPassed(ExecOutput, ExecOutput),
}

impl CompareResult {
    fn new(
        abi: &Abi,
        (res1, print1): ExecResult,
        (res2, print2): ExecResult,
    ) -> eyre::Result<Self> {
        let decode = |ws: WitnessStack<FieldElement>| -> eyre::Result<Option<InputValue>> {
            let wm = &ws.peek().expect("there should be a main witness").witness;
            let (_, r) = abi.decode(wm).wrap_err("abi::decode")?;
            Ok(r)
        };

        match (res1, res2) {
            (Err(e1), Err(e2)) => Ok(CompareResult::BothFailed(e1, e2)),
            (Err(e1), Ok(ws2)) => Ok(CompareResult::LeftFailed(
                e1,
                ExecOutput { return_value: decode(ws2)?, print_output: print2 },
            )),
            (Ok(ws1), Err(e2)) => Ok(CompareResult::RightFailed(
                ExecOutput { return_value: decode(ws1)?, print_output: print1 },
                e2,
            )),
            (Ok(ws1), Ok(ws2)) => {
                let o1 = ExecOutput { return_value: decode(ws1)?, print_output: print1 };
                let o2 = ExecOutput { return_value: decode(ws2)?, print_output: print2 };
                Ok(CompareResult::BothPassed(o1, o2))
            }
        }
    }

    /// Check that the programs agree on a return value.
    ///
    /// Returns an error if anything is different.
    pub fn return_value_or_err(&self) -> eyre::Result<Option<&InputValue>> {
        match self {
            CompareResult::BothFailed(e1, e2) => {
                if Self::errors_match(e1, e2) {
                    // Both programs failed the same way.
                    Ok(None)
                } else {
                    bail!("both programs failed: {e1} vs {e2}\n{e1:?}\n{e2:?}")
                }
            }
            CompareResult::LeftFailed(e, _) => {
                bail!("first program failed: {e}\n{e:?}")
            }
            CompareResult::RightFailed(_, e) => {
                bail!("second program failed: {e}\n{e:?}")
            }
            CompareResult::BothPassed(o1, o2) => {
                if o1.return_value != o2.return_value {
                    bail!(
                        "programs disagree on return value:\n{:?}\n!=\n{:?}",
                        o1.return_value,
                        o2.return_value
                    )
                } else if o1.print_output != o2.print_output {
                    bail!(
                        "programs disagree on printed output:\n---\n{}\n\n---\n{}\n",
                        o1.print_output,
                        o2.print_output
                    )
                } else {
                    Ok(o1.return_value.as_ref())
                }
            }
        }
    }

    /// Check whether two errors can be considered equivalent.
    fn errors_match(e1: &NargoError<FieldElement>, e2: &NargoError<FieldElement>) -> bool {
        use ExecutionError::*;

        // For now consider non-execution errors as failures we need to investigate.
        let NargoError::ExecutionError(ee1) = e1 else {
            return false;
        };
        let NargoError::ExecutionError(ee2) = e2 else {
            return false;
        };

        match (ee1, ee2) {
            (AssertionFailed(p1, _, _), AssertionFailed(p2, _, _)) => p1 == p2,
            (SolvingError(s1, _), SolvingError(s2, _)) => format!("{s1}") == format!("{s2}"),
            (SolvingError(s, _), AssertionFailed(p, _, _))
            | (AssertionFailed(p, _, _), SolvingError(s, _)) => match (s, p) {
                (
                    OpcodeResolutionError::UnsatisfiedConstrain { .. },
                    ResolvedAssertionPayload::String(s),
                ) => s == "Attempted to divide by zero",
                _ => false,
            },
        }
    }
}

pub struct CompareArtifact {
    pub options: CompareOptions,
    pub artifact: SsaProgramArtifact,
}

impl CompareArtifact {
    fn new(artifact: SsaProgramArtifact, options: CompareOptions) -> Self {
        Self { artifact, options }
    }
}

impl From<(SsaProgramArtifact, CompareOptions)> for CompareArtifact {
    fn from((artifact, options): (SsaProgramArtifact, CompareOptions)) -> Self {
        Self::new(artifact, options)
    }
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
        println!("{}", self.source);
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

/// Compare the execution of equivalent programs, compiled in different ways.
pub struct CompareCompiled<P> {
    pub program: P,
    pub abi: Abi,
    pub input_map: InputMap,
    pub ssa1: CompareArtifact,
    pub ssa2: CompareArtifact,
}

impl<P> CompareCompiled<P> {
    /// Execute the two SSAs and compare the results.
    pub fn exec(&self) -> eyre::Result<CompareResult> {
        let blackbox_solver = Bn254BlackBoxSolver(false);
        let initial_witness = self.abi.encode(&self.input_map, None).wrap_err("abi::encode")?;

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

        let (res1, print1) = do_exec(&self.ssa1.artifact.program);
        let (res2, print2) = do_exec(&self.ssa2.artifact.program);

        CompareResult::new(&self.abi, (res1, print1), (res2, print2))
    }
}

/// Compare the execution the same program compiled in two different ways.
pub type ComparePipelines = CompareCompiled<Program>;

impl CompareCompiled<Program> {
    /// Generate a random AST and compile it into SSA in two different ways.
    pub fn arb(
        u: &mut Unstructured,
        c: Config,
        f: impl FnOnce(
            &mut Unstructured,
            Program,
        ) -> arbitrary::Result<(SsaProgramArtifact, CompareOptions)>,
        g: impl FnOnce(
            &mut Unstructured,
            Program,
        ) -> arbitrary::Result<(SsaProgramArtifact, CompareOptions)>,
    ) -> arbitrary::Result<Self> {
        let program = arb_program(u, c)?;
        let abi = program_abi(&program);

        let ssa1 = CompareArtifact::from(f(u, program.clone())?);
        let ssa2 = CompareArtifact::from(g(u, program.clone())?);

        let input_program = &ssa1.artifact.program;
        let input_map = arb_inputs(u, input_program, &abi)?;

        Ok(Self { program, abi, input_map, ssa1, ssa2 })
    }
}

/// Compare two equivalent variants of the same program, compiled the same way.
pub type CompareMorph = CompareCompiled<(Program, Program)>;

impl CompareMorph {
    /// Generate a random AST, a random metamorph of it, then compile both into SSA with the same options.
    pub fn arb(
        u: &mut Unstructured,
        c: Config,
        f: impl Fn(&mut Unstructured, Program) -> arbitrary::Result<(Program, CompareOptions)>,
        g: impl Fn(Program, &CompareOptions) -> SsaProgramArtifact,
    ) -> arbitrary::Result<Self> {
        let program1 = arb_program(u, c)?;
        let (program2, options) = f(u, program1.clone())?;
        let abi = program_abi(&program1);

        let ssa1 = g(program1.clone(), &options);
        let ssa2 = g(program2.clone(), &options);

        let input_program = &ssa1.program;
        let input_map = arb_inputs(u, input_program, &abi)?;

        Ok(Self {
            program: (program1, program2),
            abi,
            input_map,
            ssa1: CompareArtifact::new(ssa1, options.clone()),
            ssa2: CompareArtifact::new(ssa2, options),
        })
    }
}

/// Help iterate over the program(s) in the comparable artifact.
pub trait HasPrograms {
    fn programs(&self) -> Vec<&Program>;
}

impl HasPrograms for ComparePipelines {
    fn programs(&self) -> Vec<&Program> {
        vec![&self.program]
    }
}

impl HasPrograms for CompareMorph {
    fn programs(&self) -> Vec<&Program> {
        vec![&self.program.0, &self.program.1]
    }
}
