use acvm::acir::circuit::{Circuit, Opcode, OpcodeLocation};
use acvm::acir::native_types::{Witness, WitnessMap};
use acvm::brillig_vm::{brillig::Value, Registers};
use acvm::pwg::{
    ACVMStatus, BrilligSolver, BrilligSolverStatus, ForeignCallWaitInfo, StepResult, ACVM,
};
use acvm::{BlackBoxFunctionSolver, FieldElement};

use nargo::artifacts::debug::DebugArtifact;
use nargo::errors::{ExecutionError, Location};
use nargo::ops::ForeignCallExecutor;
use nargo::NargoError;

use std::collections::{hash_set::Iter, HashSet};

#[derive(Debug)]
pub(super) enum DebugCommandResult {
    Done,
    Ok,
    BreakpointReached(OpcodeLocation),
    Error(NargoError),
}

pub(super) struct DebugContext<'a, B: BlackBoxFunctionSolver> {
    acvm: ACVM<'a, B>,
    brillig_solver: Option<BrilligSolver<'a, B>>,
    foreign_call_executor: Box<dyn ForeignCallExecutor + 'a>,
    debug_artifact: &'a DebugArtifact,
    breakpoints: HashSet<OpcodeLocation>,
}

impl<'a, B: BlackBoxFunctionSolver> DebugContext<'a, B> {
    pub(super) fn new(
        blackbox_solver: &'a B,
        circuit: &'a Circuit,
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap,
        foreign_call_executor: Box<dyn ForeignCallExecutor + 'a>,
    ) -> Self {
        Self {
            acvm: ACVM::new(blackbox_solver, &circuit.opcodes, initial_witness),
            brillig_solver: None,
            foreign_call_executor,
            debug_artifact,
            breakpoints: HashSet::new(),
        }
    }

    pub(super) fn get_opcodes(&self) -> &[Opcode] {
        self.acvm.opcodes()
    }

    pub(super) fn get_witness_map(&self) -> &WitnessMap {
        self.acvm.witness_map()
    }

    pub(super) fn overwrite_witness(
        &mut self,
        witness: Witness,
        value: FieldElement,
    ) -> Option<FieldElement> {
        self.acvm.overwrite_witness(witness, value)
    }

    pub(super) fn get_current_opcode_location(&self) -> Option<OpcodeLocation> {
        let ip = self.acvm.instruction_pointer();
        if ip >= self.get_opcodes().len() {
            None
        } else if let Some(ref solver) = self.brillig_solver {
            Some(OpcodeLocation::Brillig {
                acir_index: ip,
                brillig_index: solver.program_counter(),
            })
        } else {
            Some(OpcodeLocation::Acir(ip))
        }
    }

    /// Returns the callstack in source code locations for the currently
    /// executing opcode. This can be `None` if the execution finished (and
    /// `get_current_opcode_location()` returns `None`) or if the opcode is not
    /// mapped to a specific source location in the debug artifact (which can
    /// happen for certain opcodes inserted synthetically by the compiler)
    pub(super) fn get_current_source_location(&self) -> Option<Vec<Location>> {
        self.get_current_opcode_location()
            .as_ref()
            .and_then(|location| self.debug_artifact.debug_symbols[0].opcode_location(location))
    }

    fn get_opcodes_sizes(&self) -> Vec<usize> {
        self.get_opcodes()
            .iter()
            .map(|opcode| match opcode {
                Opcode::Brillig(brillig_block) => brillig_block.bytecode.len(),
                _ => 1,
            })
            .collect()
    }

    /// Offsets the given location by the given number of opcodes (including
    /// Brillig opcodes). If the offset would move the location outside of a
    /// valid circuit location, returns None and the number of remaining
    /// opcodes/instructions left which span outside the valid range in the
    /// second element of the returned tuple.
    pub(super) fn offset_opcode_location(
        &self,
        location: &Option<OpcodeLocation>,
        mut offset: i64,
    ) -> (Option<OpcodeLocation>, i64) {
        if offset == 0 {
            return (*location, 0);
        }
        let Some(location) = location else {
            return (None, offset);
        };

        let (mut acir_index, mut brillig_index) = match location {
            OpcodeLocation::Acir(acir_index) => (*acir_index, 0),
            OpcodeLocation::Brillig { acir_index, brillig_index } => (*acir_index, *brillig_index),
        };
        let opcode_sizes = self.get_opcodes_sizes();
        if offset > 0 {
            while offset > 0 {
                let opcode_size = opcode_sizes[acir_index] as i64 - brillig_index as i64;
                if offset >= opcode_size {
                    acir_index += 1;
                    offset -= opcode_size;
                    brillig_index = 0;
                } else {
                    brillig_index += offset as usize;
                    offset = 0;
                }
                if acir_index >= opcode_sizes.len() {
                    return (None, offset);
                }
            }
        } else {
            while offset < 0 {
                if brillig_index > 0 {
                    if brillig_index > (-offset) as usize {
                        brillig_index -= (-offset) as usize;
                        offset = 0;
                    } else {
                        offset += brillig_index as i64;
                        brillig_index = 0;
                    }
                } else {
                    if acir_index == 0 {
                        return (None, offset);
                    }
                    acir_index -= 1;
                    let opcode_size = opcode_sizes[acir_index] as i64;
                    if opcode_size <= -offset {
                        offset += opcode_size;
                    } else {
                        brillig_index = (opcode_size + offset) as usize;
                        offset = 0;
                    }
                }
            }
        }
        if brillig_index > 0 {
            (Some(OpcodeLocation::Brillig { acir_index, brillig_index }), 0)
        } else {
            (Some(OpcodeLocation::Acir(acir_index)), 0)
        }
    }

    pub(super) fn render_opcode_at_location(&self, location: &Option<OpcodeLocation>) -> String {
        let opcodes = self.get_opcodes();
        match location {
            None => String::from("invalid"),
            Some(OpcodeLocation::Acir(acir_index)) => {
                let opcode = &opcodes[*acir_index];
                if let Opcode::Brillig(ref brillig) = opcode {
                    let first_opcode = &brillig.bytecode[0];
                    format!("BRILLIG {first_opcode:?}")
                } else {
                    format!("{opcode:?}")
                }
            }
            Some(OpcodeLocation::Brillig { acir_index, brillig_index }) => {
                if let Opcode::Brillig(ref brillig) = opcodes[*acir_index] {
                    let opcode = &brillig.bytecode[*brillig_index];
                    format!("      | {opcode:?}")
                } else {
                    String::from("      | invalid")
                }
            }
        }
    }

    fn step_brillig_opcode(&mut self) -> DebugCommandResult {
        let Some(mut solver) = self.brillig_solver.take() else {
            unreachable!("Missing Brillig solver");
        };
        match solver.step() {
            Ok(BrilligSolverStatus::InProgress) => {
                self.brillig_solver = Some(solver);
                if self.breakpoint_reached() {
                    DebugCommandResult::BreakpointReached(
                        self.get_current_opcode_location()
                            .expect("Breakpoint reached but we have no location"),
                    )
                } else {
                    DebugCommandResult::Ok
                }
            }
            Ok(BrilligSolverStatus::Finished) => {
                let status = self.acvm.finish_brillig_with_solver(solver);
                self.handle_acvm_status(status)
            }
            Ok(BrilligSolverStatus::ForeignCallWait(foreign_call)) => {
                self.brillig_solver = Some(solver);
                self.handle_foreign_call(foreign_call)
            }
            Err(err) => DebugCommandResult::Error(NargoError::ExecutionError(
                ExecutionError::SolvingError(err),
            )),
        }
    }

    fn handle_foreign_call(&mut self, foreign_call: ForeignCallWaitInfo) -> DebugCommandResult {
        let foreign_call_result = self.foreign_call_executor.execute(&foreign_call);
        match foreign_call_result {
            Ok(foreign_call_result) => {
                if let Some(mut solver) = self.brillig_solver.take() {
                    solver.resolve_pending_foreign_call(foreign_call_result);
                    self.brillig_solver = Some(solver);
                } else {
                    self.acvm.resolve_pending_foreign_call(foreign_call_result);
                }
                // TODO: should we retry executing the opcode somehow in this case?
                DebugCommandResult::Ok
            }
            Err(error) => DebugCommandResult::Error(error.into()),
        }
    }

    fn handle_acvm_status(&mut self, status: ACVMStatus) -> DebugCommandResult {
        if let ACVMStatus::RequiresForeignCall(foreign_call) = status {
            return self.handle_foreign_call(foreign_call);
        }

        match status {
            ACVMStatus::Solved => DebugCommandResult::Done,
            ACVMStatus::InProgress => {
                if self.breakpoint_reached() {
                    DebugCommandResult::BreakpointReached(
                        self.get_current_opcode_location()
                            .expect("Breakpoint reached but we have no location"),
                    )
                } else {
                    DebugCommandResult::Ok
                }
            }
            ACVMStatus::Failure(error) => DebugCommandResult::Error(NargoError::ExecutionError(
                ExecutionError::SolvingError(error),
            )),
            ACVMStatus::RequiresForeignCall(_) => {
                unreachable!("Unexpected pending foreign call resolution");
            }
        }
    }

    pub(super) fn step_into_opcode(&mut self) -> DebugCommandResult {
        if self.brillig_solver.is_some() {
            return self.step_brillig_opcode();
        }

        match self.acvm.step_into_brillig_opcode() {
            StepResult::IntoBrillig(solver) => {
                self.brillig_solver = Some(solver);
                self.step_brillig_opcode()
            }
            StepResult::Status(status) => self.handle_acvm_status(status),
        }
    }

    fn currently_executing_brillig(&self) -> bool {
        if self.brillig_solver.is_some() {
            return true;
        }

        match self.get_current_opcode_location() {
            Some(OpcodeLocation::Brillig { .. }) => true,
            Some(OpcodeLocation::Acir(acir_index)) => {
                matches!(self.get_opcodes()[acir_index], Opcode::Brillig(_))
            }
            _ => false,
        }
    }

    fn get_current_acir_index(&self) -> Option<usize> {
        self.get_current_opcode_location().map(|opcode_location| match opcode_location {
            OpcodeLocation::Acir(acir_index) => acir_index,
            OpcodeLocation::Brillig { acir_index, .. } => acir_index,
        })
    }

    fn step_out_of_brillig_opcode(&mut self) -> DebugCommandResult {
        let Some(start_acir_index) = self.get_current_acir_index() else {
            return DebugCommandResult::Done;
        };
        loop {
            let result = self.step_into_opcode();
            if !matches!(result, DebugCommandResult::Ok) {
                return result;
            }
            let new_acir_index = self.get_current_acir_index().unwrap();
            if new_acir_index != start_acir_index {
                return DebugCommandResult::Ok;
            }
        }
    }

    pub(super) fn step_acir_opcode(&mut self) -> DebugCommandResult {
        if self.currently_executing_brillig() {
            self.step_out_of_brillig_opcode()
        } else {
            let status = self.acvm.solve_opcode();
            self.handle_acvm_status(status)
        }
    }

    pub(super) fn next(&mut self) -> DebugCommandResult {
        let start_location = self.get_current_source_location();
        loop {
            let result = self.step_into_opcode();
            if !matches!(result, DebugCommandResult::Ok) {
                return result;
            }
            let new_location = self.get_current_source_location();
            if new_location.is_some() && new_location != start_location {
                return DebugCommandResult::Ok;
            }
        }
    }

    pub(super) fn cont(&mut self) -> DebugCommandResult {
        loop {
            let result = self.step_into_opcode();
            if !matches!(result, DebugCommandResult::Ok) {
                return result;
            }
        }
    }

    pub(super) fn is_executing_brillig(&self) -> bool {
        let opcodes = self.get_opcodes();
        let acir_index = self.acvm.instruction_pointer();
        acir_index < opcodes.len() && matches!(opcodes[acir_index], Opcode::Brillig(..))
    }

    pub(super) fn get_brillig_registers(&self) -> Option<&Registers> {
        self.brillig_solver.as_ref().map(|solver| solver.get_registers())
    }

    pub(super) fn set_brillig_register(&mut self, register_index: usize, value: FieldElement) {
        if let Some(solver) = self.brillig_solver.as_mut() {
            solver.set_register(register_index, value.into());
        }
    }

    pub(super) fn get_brillig_memory(&self) -> Option<&[Value]> {
        self.brillig_solver.as_ref().map(|solver| solver.get_memory())
    }

    pub(super) fn write_brillig_memory(&mut self, ptr: usize, value: FieldElement) {
        if let Some(solver) = self.brillig_solver.as_mut() {
            solver.write_memory_at(ptr, value.into());
        }
    }

    fn breakpoint_reached(&self) -> bool {
        if let Some(location) = self.get_current_opcode_location() {
            self.breakpoints.contains(&location)
        } else {
            false
        }
    }

    pub(super) fn is_valid_opcode_location(&self, location: &OpcodeLocation) -> bool {
        let opcodes = self.get_opcodes();
        match *location {
            OpcodeLocation::Acir(acir_index) => acir_index < opcodes.len(),
            OpcodeLocation::Brillig { acir_index, brillig_index } => {
                acir_index < opcodes.len()
                    && matches!(opcodes[acir_index], Opcode::Brillig(..))
                    && {
                        if let Opcode::Brillig(ref brillig) = opcodes[acir_index] {
                            brillig_index < brillig.bytecode.len()
                        } else {
                            false
                        }
                    }
            }
        }
    }

    pub(super) fn is_breakpoint_set(&self, location: &OpcodeLocation) -> bool {
        self.breakpoints.contains(location)
    }

    pub(super) fn add_breakpoint(&mut self, location: OpcodeLocation) -> bool {
        self.breakpoints.insert(location)
    }

    pub(super) fn delete_breakpoint(&mut self, location: &OpcodeLocation) -> bool {
        self.breakpoints.remove(location)
    }

    pub(super) fn iterate_breakpoints(&self) -> Iter<'_, OpcodeLocation> {
        self.breakpoints.iter()
    }

    pub(super) fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    pub(super) fn is_solved(&self) -> bool {
        matches!(self.acvm.get_status(), ACVMStatus::Solved)
    }

    pub fn finalize(self) -> WitnessMap {
        self.acvm.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{DebugCommandResult, DebugContext};

    use acvm::{
        acir::{
            circuit::{
                brillig::{Brillig, BrilligInputs, BrilligOutputs},
                opcodes::BlockId,
            },
            native_types::Expression,
        },
        blackbox_solver::StubbedBlackBoxSolver,
        brillig_vm::brillig::{
            BinaryFieldOp, Opcode as BrilligOpcode, RegisterIndex, RegisterOrMemory,
        },
    };
    use nargo::{artifacts::debug::DebugArtifact, ops::DefaultForeignCallExecutor};
    use std::collections::BTreeMap;

    #[test]
    fn test_resolve_foreign_calls_stepping_into_brillig() {
        let fe_0 = FieldElement::zero();
        let fe_1 = FieldElement::one();
        let w_x = Witness(1);

        let brillig_opcodes = Brillig {
            inputs: vec![BrilligInputs::Single(Expression {
                linear_combinations: vec![(fe_1, w_x)],
                ..Expression::default()
            })],
            outputs: vec![],
            bytecode: vec![
                BrilligOpcode::Const {
                    destination: RegisterIndex::from(1),
                    value: Value::from(fe_0),
                },
                BrilligOpcode::ForeignCall {
                    function: "clear_mock".into(),
                    destinations: vec![],
                    inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
                },
                BrilligOpcode::Stop,
            ],
            predicate: None,
        };
        let opcodes = vec![Opcode::Brillig(brillig_opcodes)];
        let current_witness_index = 2;
        let circuit = &Circuit { current_witness_index, opcodes, ..Circuit::default() };

        let debug_symbols = vec![];
        let file_map = BTreeMap::new();
        let warnings = vec![];
        let debug_artifact = &DebugArtifact { debug_symbols, file_map, warnings };

        let initial_witness = BTreeMap::from([(Witness(1), fe_1)]).into();

        let mut context = DebugContext::new(
            &StubbedBlackBoxSolver,
            circuit,
            debug_artifact,
            initial_witness,
            Box::new(DefaultForeignCallExecutor::new(true, None)),
        );

        assert_eq!(context.get_current_opcode_location(), Some(OpcodeLocation::Acir(0)));

        // execute the first Brillig opcode (const)
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_opcode_location(),
            Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 1 })
        );

        // try to execute the second Brillig opcode (and resolve the foreign call)
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_opcode_location(),
            Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 1 })
        );

        // retry the second Brillig opcode (foreign call should be finished)
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_opcode_location(),
            Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 2 })
        );

        // last Brillig opcode
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Done));
        assert_eq!(context.get_current_opcode_location(), None);
    }

    #[test]
    fn test_break_brillig_block_while_stepping_acir_opcodes() {
        let fe_0 = FieldElement::zero();
        let fe_1 = FieldElement::one();
        let w_x = Witness(1);
        let w_y = Witness(2);
        let w_z = Witness(3);

        // This Brillig block is equivalent to: z = x + y
        let brillig_opcodes = Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    linear_combinations: vec![(fe_1, w_x)],
                    ..Expression::default()
                }),
                BrilligInputs::Single(Expression {
                    linear_combinations: vec![(fe_1, w_y)],
                    ..Expression::default()
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(w_z)],
            bytecode: vec![
                BrilligOpcode::BinaryFieldOp {
                    destination: RegisterIndex::from(0),
                    op: BinaryFieldOp::Add,
                    lhs: RegisterIndex::from(0),
                    rhs: RegisterIndex::from(1),
                },
                BrilligOpcode::Stop,
            ],
            predicate: None,
        };
        let opcodes = vec![
            // z = x + y
            Opcode::Brillig(brillig_opcodes),
            // x + y - z = 0
            Opcode::AssertZero(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
                q_c: fe_0,
            }),
        ];
        let current_witness_index = 3;
        let circuit = &Circuit { current_witness_index, opcodes, ..Circuit::default() };

        let debug_symbols = vec![];
        let file_map = BTreeMap::new();
        let warnings = vec![];
        let debug_artifact = &DebugArtifact { debug_symbols, file_map, warnings };

        let initial_witness = BTreeMap::from([(Witness(1), fe_1), (Witness(2), fe_1)]).into();

        let mut context = DebugContext::new(
            &StubbedBlackBoxSolver,
            circuit,
            debug_artifact,
            initial_witness,
            Box::new(DefaultForeignCallExecutor::new(true, None)),
        );

        // set breakpoint
        let breakpoint_location = OpcodeLocation::Brillig { acir_index: 0, brillig_index: 1 };
        assert!(context.add_breakpoint(breakpoint_location));

        // execute the first ACIR opcode (Brillig block) -> should reach the breakpoint instead
        let result = context.step_acir_opcode();
        assert!(matches!(result, DebugCommandResult::BreakpointReached(_)));
        assert_eq!(context.get_current_opcode_location(), Some(breakpoint_location));

        // continue execution to the next ACIR opcode
        let result = context.step_acir_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(context.get_current_opcode_location(), Some(OpcodeLocation::Acir(1)));

        // last ACIR opcode
        let result = context.step_acir_opcode();
        assert!(matches!(result, DebugCommandResult::Done));
        assert_eq!(context.get_current_opcode_location(), None);
    }

    #[test]
    fn test_offset_opcode_location() {
        let opcodes = vec![
            Opcode::Brillig(Brillig {
                inputs: vec![],
                outputs: vec![],
                bytecode: vec![BrilligOpcode::Stop, BrilligOpcode::Stop, BrilligOpcode::Stop],
                predicate: None,
            }),
            Opcode::MemoryInit { block_id: BlockId(0), init: vec![] },
            Opcode::Brillig(Brillig {
                inputs: vec![],
                outputs: vec![],
                bytecode: vec![BrilligOpcode::Stop, BrilligOpcode::Stop, BrilligOpcode::Stop],
                predicate: None,
            }),
            Opcode::AssertZero(Expression::default()),
        ];
        let circuit = Circuit { opcodes, ..Circuit::default() };
        let debug_artifact =
            DebugArtifact { debug_symbols: vec![], file_map: BTreeMap::new(), warnings: vec![] };
        let context = DebugContext::new(
            &StubbedBlackBoxSolver,
            &circuit,
            &debug_artifact,
            WitnessMap::new(),
            Box::new(DefaultForeignCallExecutor::new(true, None)),
        );

        assert_eq!(context.offset_opcode_location(&None, 0), (None, 0));
        assert_eq!(context.offset_opcode_location(&None, 2), (None, 2));
        assert_eq!(context.offset_opcode_location(&None, -2), (None, -2));
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), 0),
            (Some(OpcodeLocation::Acir(0)), 0)
        );
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), 1),
            (Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 1 }), 0)
        );
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), 2),
            (Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 2 }), 0)
        );
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), 3),
            (Some(OpcodeLocation::Acir(1)), 0)
        );
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), 4),
            (Some(OpcodeLocation::Acir(2)), 0)
        );
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), 5),
            (Some(OpcodeLocation::Brillig { acir_index: 2, brillig_index: 1 }), 0)
        );
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), 7),
            (Some(OpcodeLocation::Acir(3)), 0)
        );
        assert_eq!(context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), 8), (None, 0));
        assert_eq!(context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), 20), (None, 12));
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(1)), 2),
            (Some(OpcodeLocation::Brillig { acir_index: 2, brillig_index: 1 }), 0)
        );
        assert_eq!(context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), -1), (None, -1));
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(0)), -10),
            (None, -10)
        );

        assert_eq!(
            context.offset_opcode_location(
                &Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 1 }),
                -1
            ),
            (Some(OpcodeLocation::Acir(0)), 0)
        );
        assert_eq!(
            context.offset_opcode_location(
                &Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 2 }),
                -2
            ),
            (Some(OpcodeLocation::Acir(0)), 0)
        );
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(1)), -3),
            (Some(OpcodeLocation::Acir(0)), 0)
        );
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(2)), -4),
            (Some(OpcodeLocation::Acir(0)), 0)
        );
        assert_eq!(
            context.offset_opcode_location(
                &Some(OpcodeLocation::Brillig { acir_index: 2, brillig_index: 1 }),
                -5
            ),
            (Some(OpcodeLocation::Acir(0)), 0)
        );
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(3)), -7),
            (Some(OpcodeLocation::Acir(0)), 0)
        );
        assert_eq!(
            context.offset_opcode_location(&Some(OpcodeLocation::Acir(2)), -2),
            (Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 2 }), 0)
        );
    }
}
