//! Compare an arbitrary AST compiled into SSA and executed with the
//! SSA interpreter at some stage of the SSA pipeline.

use std::collections::BTreeMap;

use arbitrary::Unstructured;
use color_eyre::eyre;
use noirc_abi::{Abi, InputMap};
use noirc_evaluator::ssa::{self, ssa_gen::Ssa};
use noirc_frontend::monomorphization::ast::Program;

use crate::{Config, arb_program, program_abi};

use super::{CompareError, CompareOptions, CompareResult, ExecOutput};

/// The state of the SSA after a particular pass in the pipeline.
pub struct ComparePass {
    /// The overall position of this pass in the pipeline.
    ///
    /// The Initial SSA is considered step 0.
    pub step: usize,
    /// The message (without the counter) of the pass.
    pub msg: String,
    /// The state of the SSA after the pass.
    pub ssa: Ssa,
}

type InterpretResult =
    Result<Vec<ssa::interpreter::value::Value>, ssa::interpreter::errors::InterpreterError>;

/// The result of the SSA interpreter execution.
pub type CompareInterpretedResult =
    CompareResult<Vec<ssa::interpreter::value::Value>, ssa::interpreter::errors::InterpreterError>;

/// Compare the interpretation of two SSA states of an arbitrary program.
pub struct CompareInterpreted {
    pub program: Program,
    pub abi: Abi,
    // TODO: Figure out how to map ABI to SSA input values.
    pub input_map: InputMap,
    /// Options that influence the pipeline, common to both passes.
    pub options: CompareOptions,
    pub ssa1: ComparePass,
    pub ssa2: ComparePass,
}

impl CompareInterpreted {
    /// 1. Generate an arbitrary AST
    /// 2. Stop the compilation at two arbitrary SSA passes
    /// 3. Generate input for the main function of the SSA
    pub fn arb(
        u: &mut Unstructured,
        c: Config,
        f: impl FnOnce(
            &mut Unstructured,
            Program,
        ) -> arbitrary::Result<(CompareOptions, ComparePass, ComparePass)>,
    ) -> arbitrary::Result<Self> {
        let program = arb_program(u, c)?;
        let abi = program_abi(&program);
        let (options, ssa1, ssa2) = f(u, program.clone())?;

        // TODO: Figure out how to create random input from the SSA itself.
        let input_map = BTreeMap::default();

        Ok(Self { program, abi, input_map, options, ssa1, ssa2 })
    }

    pub fn exec(&self) -> eyre::Result<CompareInterpretedResult> {
        let inputs = todo!();
        let res1 = self.ssa1.ssa.interpret(inputs);
        let res2 = self.ssa2.ssa.interpret(inputs);
        Ok(CompareInterpretedResult::new(res1, res2))
    }
}

impl CompareInterpretedResult {
    pub fn new(res1: InterpretResult, res2: InterpretResult) -> Self {
        let out = |ret| ExecOutput { return_value: Some(ret), print_output: Default::default() };
        match (res1, res2) {
            (Ok(r1), Ok(e2)) => Self::BothPassed(out(r1), out(e2)),
            (Ok(r1), Err(e2)) => Self::RightFailed(out(r1), e2),
            (Err(e1), Ok(r2)) => Self::LeftFailed(e1, out(r2)),
            (Err(e1), Err(e2)) => Self::BothFailed(e1, e2),
        }
    }
}

impl CompareError for ssa::interpreter::errors::InterpreterError {
    fn equivalent(e1: &Self, e2: &Self) -> bool {
        e1 == e2
    }
}
