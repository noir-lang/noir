//! Compare an arbitrary AST compiled into bytecode and executed with the VM.
use arbitrary::Unstructured;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use color_eyre::eyre::{self, WrapErr};
use nargo::foreign_calls::DefaultForeignCallBuilder;
use noirc_abi::{Abi, InputMap};
use noirc_evaluator::ssa::SsaProgramArtifact;
use noirc_frontend::monomorphization::ast::Program;

use crate::{Config, arb_inputs, arb_program, program_abi};

use super::{CompareOptions, CompareResult, HasPrograms};

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
