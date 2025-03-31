use acir::{FieldElement, native_types::WitnessStack};
use arbitrary::Unstructured;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use color_eyre::eyre::{self, WrapErr};
use nargo::{NargoError, PrintOutput, foreign_calls::DefaultForeignCallBuilder};
use noirc_abi::{Abi, InputMap, input_parser::InputValue};
use noirc_evaluator::ssa::SsaProgramArtifact;
use noirc_frontend::monomorphization::ast::Program;

use crate::{Config, arb_inputs, arb_program};

/// Comparison result of the execution.
pub enum CompareResult {
    BothFailed(NargoError<FieldElement>, NargoError<FieldElement>),
    LeftFailed(NargoError<FieldElement>, Option<InputValue>),
    RightFailed(Option<InputValue>, NargoError<FieldElement>),
    Disagree(Option<InputValue>, Option<InputValue>),
    Agree(Option<InputValue>),
}

impl CompareResult {
    fn new(
        abi: &Abi,
        res1: Result<WitnessStack<FieldElement>, NargoError<FieldElement>>,
        res2: Result<WitnessStack<FieldElement>, NargoError<FieldElement>>,
    ) -> eyre::Result<Self> {
        let decode = |ws: WitnessStack<FieldElement>| -> eyre::Result<Option<InputValue>> {
            let wm = &ws.peek().expect("there should be a main witness").witness;
            let (_, r) = abi.decode(wm).wrap_err("abi::decode")?;
            Ok(r)
        };

        match (res1, res2) {
            (Err(e1), Err(e2)) => Ok(CompareResult::BothFailed(e1, e2)),
            (Err(e), Ok(ws)) => Ok(CompareResult::LeftFailed(e, decode(ws)?)),
            (Ok(ws), Err(e)) => Ok(CompareResult::RightFailed(decode(ws)?, e)),
            (Ok(ws1), Ok(ws2)) => {
                let r1 = decode(ws1)?;
                let r2 = decode(ws2)?;
                if r1 == r2 {
                    Ok(CompareResult::Agree(r1))
                } else {
                    Ok(CompareResult::Disagree(r1, r2))
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
        let mut foreign_call_executor = DefaultForeignCallBuilder::default()
            .with_mocks(false)
            .with_output(PrintOutput::None)
            .build();

        let initial_witness = self.abi.encode(&self.input_map, None).wrap_err("abi::encode")?;

        let res1 = nargo::ops::execute_program(
            &self.ssa1.program,
            initial_witness.clone(),
            &blackbox_solver,
            &mut foreign_call_executor,
        );

        let res2 = nargo::ops::execute_program(
            &self.ssa2.program,
            initial_witness,
            &blackbox_solver,
            &mut foreign_call_executor,
        );

        CompareResult::new(&self.abi, res1, res2)
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
        let (program, abi) = arb_program(u, c)?;

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
        let (program1, abi) = arb_program(u, c)?;
        let program2 = f(u, &program1)?;

        let ssa1 = g(program1.clone());
        let ssa2 = g(program2.clone());

        let input_map = arb_inputs(u, &ssa1.program, &abi)?;

        Ok(Self { program: (program1, program2), abi, input_map, ssa1, ssa2 })
    }
}
