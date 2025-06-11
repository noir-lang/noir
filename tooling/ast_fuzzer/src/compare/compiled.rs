//! Compare an arbitrary AST compiled into bytecode and executed with the VM.
use std::collections::BTreeMap;

use acir::{FieldElement, native_types::WitnessStack};
use acvm::pwg::{OpcodeResolutionError, ResolvedAssertionPayload};
use arbitrary::Unstructured;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use color_eyre::eyre::{self, WrapErr};
use nargo::{NargoError, errors::ExecutionError, foreign_calls::DefaultForeignCallBuilder};
use noirc_abi::{Abi, InputMap, input_parser::InputValue};
use noirc_evaluator::{ErrorType, ssa::SsaProgramArtifact};
use noirc_frontend::monomorphization::ast::Program;

use crate::{Config, arb_inputs, arb_program, program_abi};

use super::{Comparable, CompareOptions, CompareResult, ExecOutput, HasPrograms};

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

/// These are the error types in the `SsaProgramArtifact`, which are not the same as the ones in the ABI,
/// but they can provide extra information when comparing errors.
type SsaErrorTypes = BTreeMap<acir::circuit::ErrorSelector, ErrorType>;

/// The execution result is the value returned from the circuit and any output from `println`.
type ExecResult = (Result<WitnessStack<FieldElement>, NargoError<FieldElement>>, String);

pub struct NargoErrorWithTypes(NargoError<FieldElement>, SsaErrorTypes);

impl NargoErrorWithTypes {
    /// Copy of `NargoError::user_defined_failure_message` accepting `SsaErrorTypes` instead of ABI errors.
    fn user_defined_failure_message(&self) -> Option<String> {
        match &self.0 {
            NargoError::ExecutionError(error) => match error {
                ExecutionError::AssertionFailed(payload, _, _) => match payload {
                    ResolvedAssertionPayload::String(message) => Some(message.to_string()),
                    ResolvedAssertionPayload::Raw(raw) => {
                        let ssa_type = self.1.get(&raw.selector)?;
                        match ssa_type {
                            ErrorType::String(message) => Some(message.to_string()),
                            ErrorType::Dynamic(_hir_type) => {
                                // This would be the case if we have a format string that needs to be filled with the raw payload
                                // decoded as ABI type. The code generator shouldn't produce this kind. It shouldn't be too difficult
                                // to map the type, but the mapper in `crate::abi` doesn't handle format strings at the moment.
                                panic!("didn't expect dynamic error types")
                            }
                        }
                    }
                },
                ExecutionError::SolvingError(error, _) => match error {
                    OpcodeResolutionError::BlackBoxFunctionFailed(_, reason) => {
                        Some(reason.to_string())
                    }
                    _ => None,
                },
            },
            NargoError::ForeignCallError(error) => Some(error.to_string()),
            _ => None,
        }
    }
}

/// The result of the execution of compiled programs, decoded by their ABI.
pub type CompareCompiledResult = CompareResult<InputValue, NargoErrorWithTypes>;

impl CompareCompiledResult {
    pub fn new(
        abi: &Abi,
        ets1: &SsaErrorTypes,
        ets2: &SsaErrorTypes,
        (res1, print1): ExecResult,
        (res2, print2): ExecResult,
    ) -> eyre::Result<Self> {
        let decode = |ws: WitnessStack<FieldElement>| -> eyre::Result<Option<InputValue>> {
            let wm = &ws.peek().expect("there should be a main witness").witness;
            let (_, r) = abi.decode(wm).wrap_err("abi::decode")?;
            Ok(r)
        };

        match (res1, res2) {
            (Err(e1), Err(e2)) => Ok(CompareResult::BothFailed(
                NargoErrorWithTypes(e1, ets1.clone()),
                NargoErrorWithTypes(e2, ets2.clone()),
            )),
            (Err(e1), Ok(ws2)) => Ok(CompareResult::LeftFailed(
                NargoErrorWithTypes(e1, ets1.clone()),
                ExecOutput { return_value: decode(ws2)?, print_output: print2 },
            )),
            (Ok(ws1), Err(e2)) => Ok(CompareResult::RightFailed(
                ExecOutput { return_value: decode(ws1)?, print_output: print1 },
                NargoErrorWithTypes(e2, ets2.clone()),
            )),
            (Ok(ws1), Ok(ws2)) => {
                let o1 = ExecOutput { return_value: decode(ws1)?, print_output: print1 };
                let o2 = ExecOutput { return_value: decode(ws2)?, print_output: print2 };
                Ok(CompareResult::BothPassed(o1, o2))
            }
        }
    }
}

impl Comparable for NargoErrorWithTypes {
    fn equivalent(e1: &Self, e2: &Self) -> bool {
        use ExecutionError::*;

        // For now consider non-execution errors as failures we need to investigate.
        let NargoError::ExecutionError(ee1) = &e1.0 else {
            return false;
        };
        let NargoError::ExecutionError(ee2) = &e2.0 else {
            return false;
        };

        let msg1 = e1.user_defined_failure_message();
        let msg2 = e2.user_defined_failure_message();
        let is_same_msg = msg1.is_some() && msg2.is_some() && msg1 == msg2;

        match (ee1, ee2) {
            (
                AssertionFailed(ResolvedAssertionPayload::String(c), _, _),
                AssertionFailed(_, _, _),
            ) if c.contains("CustomDiagnostic") => {
                // Looks like the workaround we have for comptime failures originating from overflows and similar assertion failures.
                true
            }
            (AssertionFailed(p1, _, _), AssertionFailed(p2, _, _)) => p1 == p2 || is_same_msg,
            (SolvingError(s1, _), SolvingError(s2, _)) => format!("{s1}") == format!("{s2}"),
            (SolvingError(s, _), AssertionFailed(p, _, _))
            | (AssertionFailed(p, _, _), SolvingError(s, _)) => match (s, p) {
                (
                    OpcodeResolutionError::UnsatisfiedConstrain { .. },
                    ResolvedAssertionPayload::String(s),
                ) => s == "Attempted to divide by zero",
                _ => is_same_msg,
            },
        }
    }
}

impl Comparable for InputValue {
    fn equivalent(a: &Self, b: &Self) -> bool {
        a == b
    }
}

impl std::fmt::Display for NargoErrorWithTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::fmt::Debug for NargoErrorWithTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
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
    pub fn exec(&self) -> eyre::Result<CompareCompiledResult> {
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

        CompareCompiledResult::new(
            &self.abi,
            &self.ssa1.artifact.error_types,
            &self.ssa2.artifact.error_types,
            (res1, print1),
            (res2, print2),
        )
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

impl HasPrograms for ComparePipelines {
    fn programs(&self) -> Vec<&Program> {
        vec![&self.program]
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

impl HasPrograms for CompareMorph {
    fn programs(&self) -> Vec<&Program> {
        vec![&self.program.0, &self.program.1]
    }
}
