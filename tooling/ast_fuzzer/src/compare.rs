use acir::{FieldElement, native_types::WitnessStack};
use arbitrary::Unstructured;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use color_eyre::eyre::{self, WrapErr, bail};
use nargo::{NargoError, foreign_calls::DefaultForeignCallBuilder};
use noirc_abi::{Abi, InputMap, input_parser::InputValue};
use noirc_evaluator::ssa::SsaProgramArtifact;
use noirc_frontend::monomorphization::ast::Program;

use crate::{Config, arb_inputs, arb_program, program_abi};

#[derive(Clone, Debug, PartialEq)]
pub struct ExecOutput {
    pub return_value: Option<InputValue>,
    pub print_output: String,
}

type ExecResult = (Result<WitnessStack<FieldElement>, NargoError<FieldElement>>, String);

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
                // For now raise an error to catch anything unexpected, but in the future if
                // both fail the same way (e.g. assertion failure) then it should be okay.
                bail!("both programs failed: {e1}; {e2}")
            }
            CompareResult::LeftFailed(e, _) => {
                bail!("first program failed: {e}")
            }
            CompareResult::RightFailed(_, e) => {
                bail!("second program failed: {e}")
            }
            CompareResult::BothPassed(o1, o2) => {
                if o1.return_value != o2.return_value {
                    bail!(
                        "programs disagree on return value: {:?} != {:?}",
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
}

/// Compare the execution of different SSA representations of equivalent program(s).
pub struct CompareSsa<P> {
    pub program: P,
    pub abi: Abi,
    pub input_map: InputMap,
    pub ssa1: SsaProgramArtifact,
    pub ssa2: SsaProgramArtifact,
}

impl<P> CompareSsa<P> {
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

        let (res1, print1) = do_exec(&self.ssa1.program);
        let (res2, print2) = do_exec(&self.ssa2.program);

        CompareResult::new(&self.abi, (res1, print1), (res2, print2))
    }
}

/// Compare the execution the same program compiled in two different ways.
pub type ComparePasses = CompareSsa<Program>;

impl CompareSsa<Program> {
    /// Generate a random AST and compile it into SSA in two different ways.
    pub fn arb(
        u: &mut Unstructured,
        c: Config,
        f: impl FnOnce(Program) -> SsaProgramArtifact,
        g: impl FnOnce(Program) -> SsaProgramArtifact,
    ) -> arbitrary::Result<Self> {
        let program = arb_program(u, c)?;
        let abi = program_abi(&program);

        let ssa1 = f(program.clone());
        let ssa2 = g(program.clone());

        let input_map = arb_inputs(u, &ssa1.program, &abi)?;

        Ok(Self { program, abi, input_map, ssa1, ssa2 })
    }
}

/// Compare two equivalent variants of the same program, compiled the same way.
pub type CompareMutants = CompareSsa<(Program, Program)>;

impl CompareMutants {
    /// Generate a random AST and compile it into SSA in two different ways.
    pub fn arb(
        u: &mut Unstructured,
        c: Config,
        f: impl Fn(&mut Unstructured, &Program) -> arbitrary::Result<Program>,
        g: impl Fn(Program) -> SsaProgramArtifact,
    ) -> arbitrary::Result<Self> {
        let program1 = arb_program(u, c)?;
        let program2 = f(u, &program1)?;
        let abi = program_abi(&program1);

        let ssa1 = g(program1.clone());
        let ssa2 = g(program2.clone());

        let input_map = arb_inputs(u, &ssa1.program, &abi)?;

        Ok(Self { program: (program1, program2), abi, input_map, ssa1, ssa2 })
    }
}
