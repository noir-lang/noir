use crate::foreign_calls::DebugForeignCallExecutor;
use acvm::acir::circuit::brillig::BrilligBytecode;
use acvm::acir::circuit::{Circuit, Opcode, OpcodeLocation};
use acvm::acir::native_types::{Witness, WitnessMap};
use acvm::brillig_vm::MemoryValue;
use acvm::pwg::{
    ACVMStatus, BrilligSolver, BrilligSolverStatus, ForeignCallWaitInfo, StepResult, ACVM,
};
use acvm::{BlackBoxFunctionSolver, FieldElement};

use codespan_reporting::files::{Files, SimpleFile};
use fm::FileId;
use nargo::artifacts::debug::{DebugArtifact, StackFrame};
use nargo::errors::{ExecutionError, Location};
use nargo::NargoError;
use noirc_driver::DebugFile;

use std::collections::BTreeMap;
use std::collections::{hash_set::Iter, HashSet};

#[derive(Debug)]
pub(super) enum DebugCommandResult {
    Done,
    Ok,
    BreakpointReached(OpcodeLocation),
    Error(NargoError),
}

pub(super) struct DebugContext<'a, B: BlackBoxFunctionSolver<FieldElement>> {
    acvm: ACVM<'a, FieldElement, B>,
    brillig_solver: Option<BrilligSolver<'a, FieldElement, B>>,
    foreign_call_executor: Box<dyn DebugForeignCallExecutor + 'a>,
    debug_artifact: &'a DebugArtifact,
    breakpoints: HashSet<OpcodeLocation>,
    source_to_opcodes: BTreeMap<FileId, Vec<(usize, OpcodeLocation)>>,
    unconstrained_functions: &'a [BrilligBytecode<FieldElement>],

    // Absolute (in terms of all the opcodes ACIR+Brillig) addresses of the ACIR
    // opcodes with one additional entry for to indicate the last valid address.
    acir_opcode_addresses: Vec<usize>,
}

impl<'a, B: BlackBoxFunctionSolver<FieldElement>> DebugContext<'a, B> {
    pub(super) fn new(
        blackbox_solver: &'a B,
        circuit: &'a Circuit<FieldElement>,
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap<FieldElement>,
        foreign_call_executor: Box<dyn DebugForeignCallExecutor + 'a>,
        unconstrained_functions: &'a [BrilligBytecode<FieldElement>],
    ) -> Self {
        let source_to_opcodes = build_source_to_opcode_debug_mappings(debug_artifact);
        let acir_opcode_addresses = build_acir_opcode_offsets(circuit, unconstrained_functions);
        Self {
            // TODO: need to handle brillig pointer in the debugger
            acvm: ACVM::new(
                blackbox_solver,
                &circuit.opcodes,
                initial_witness,
                unconstrained_functions,
                &circuit.assert_messages,
            ),
            brillig_solver: None,
            foreign_call_executor,
            debug_artifact,
            breakpoints: HashSet::new(),
            source_to_opcodes,
            unconstrained_functions,
            acir_opcode_addresses,
        }
    }

    pub(super) fn get_opcodes(&self) -> &[Opcode<FieldElement>] {
        self.acvm.opcodes()
    }

    pub(super) fn get_witness_map(&self) -> &WitnessMap<FieldElement> {
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

    pub(super) fn get_call_stack(&self) -> Vec<OpcodeLocation> {
        let instruction_pointer = self.acvm.instruction_pointer();
        if instruction_pointer >= self.get_opcodes().len() {
            vec![]
        } else if let Some(ref solver) = self.brillig_solver {
            solver
                .get_call_stack()
                .iter()
                .map(|program_counter| OpcodeLocation::Brillig {
                    acir_index: instruction_pointer,
                    brillig_index: *program_counter,
                })
                .collect()
        } else {
            vec![OpcodeLocation::Acir(instruction_pointer)]
        }
    }

    pub(super) fn is_source_location_in_debug_module(&self, location: &Location) -> bool {
        self.debug_artifact
            .file_map
            .get(&location.file)
            .map(is_debug_file_in_debug_crate)
            .unwrap_or(false)
    }

    /// Find an opcode location matching a source code location
    // We apply some heuristics here, and there are four possibilities for the
    // return value of this function:
    // 1. the source location is not found -> None
    // 2. an exact unique location is found (very rare) -> Some(opcode_location)
    // 3. an exact but not unique location is found, ie. a source location may
    //    be mapped to multiple opcodes, and those may be disjoint, for example for
    //    functions called multiple times throughout the program
    //    -> return the first opcode in program order that matches the source location
    // 4. exact location is not found, so an opcode for a nearby source location
    //    is returned (this again could actually be more than one opcodes)
    //    -> return the opcode for the next source line that is mapped
    pub(super) fn find_opcode_for_source_location(
        &self,
        file_id: &FileId,
        line: i64,
    ) -> Option<OpcodeLocation> {
        let line = line as usize;
        let line_to_opcodes = self.source_to_opcodes.get(file_id)?;
        let found_index = match line_to_opcodes.binary_search_by(|x| x.0.cmp(&line)) {
            Ok(index) => {
                // move backwards to find the first opcode which matches the line
                let mut index = index;
                while index > 0 && line_to_opcodes[index - 1].0 == line {
                    index -= 1;
                }
                line_to_opcodes[index].1
            }
            Err(index) => {
                if index >= line_to_opcodes.len() {
                    return None;
                }
                line_to_opcodes[index].1
            }
        };
        Some(found_index)
    }

    /// Returns the callstack in source code locations for the currently
    /// executing opcode. This can be `None` if the execution finished (and
    /// `get_current_opcode_location()` returns `None`) or if the opcode is not
    /// mapped to a specific source location in the debug artifact (which can
    /// happen for certain opcodes inserted synthetically by the compiler).
    /// This function also filters source locations that are determined to be in
    /// the internal debug module.
    pub(super) fn get_current_source_location(&self) -> Option<Vec<Location>> {
        self.get_current_opcode_location()
            .as_ref()
            .map(|opcode_location| self.get_source_location_for_opcode_location(opcode_location))
            .filter(|v: &Vec<Location>| !v.is_empty())
    }

    /// Returns the (possible) stack of source locations corresponding to the
    /// given opcode location. Due to compiler inlining it's possible for this
    /// function to return multiple source locations. An empty vector means that
    /// the given opcode location cannot be mapped back to a source location
    /// (eg. it may be pure debug instrumentation code or other synthetically
    /// produced opcode by the compiler)
    pub(super) fn get_source_location_for_opcode_location(
        &self,
        opcode_location: &OpcodeLocation,
    ) -> Vec<Location> {
        // TODO: this assumes we're debugging a program (ie. the DebugArtifact
        // will contain a single DebugInfo), but this assumption doesn't hold
        // for contracts
        self.debug_artifact.debug_symbols[0]
            .opcode_location(opcode_location)
            .map(|source_locations| {
                source_locations
                    .into_iter()
                    .filter(|source_location| {
                        !self.is_source_location_in_debug_module(source_location)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the current call stack with expanded source locations. In
    /// general, the matching between opcode location and source location is 1
    /// to 1, but due to the compiler inlining functions a single opcode
    /// location may expand to multiple source locations.
    pub(super) fn get_source_call_stack(&self) -> Vec<(OpcodeLocation, Location)> {
        self.get_call_stack()
            .iter()
            .flat_map(|opcode_location| {
                self.get_source_location_for_opcode_location(opcode_location)
                    .into_iter()
                    .map(|source_location| (*opcode_location, source_location))
            })
            .collect()
    }

    /// Returns the absolute address of the opcode at the given location.
    /// Absolute here means accounting for nested Brillig opcodes in BrilligCall
    /// opcodes.
    pub fn opcode_location_to_address(&self, location: &OpcodeLocation) -> usize {
        match location {
            OpcodeLocation::Acir(acir_index) => self.acir_opcode_addresses[*acir_index],
            OpcodeLocation::Brillig { acir_index, brillig_index } => {
                self.acir_opcode_addresses[*acir_index] + *brillig_index
            }
        }
    }

    pub fn address_to_opcode_location(&self, address: usize) -> Option<OpcodeLocation> {
        if address >= *self.acir_opcode_addresses.last().unwrap_or(&0) {
            return None;
        }
        let location = match self.acir_opcode_addresses.binary_search(&address) {
            Ok(found_index) => OpcodeLocation::Acir(found_index),
            Err(insert_index) => {
                let acir_index = insert_index - 1;
                let base_offset = self.acir_opcode_addresses[acir_index];
                let brillig_index = address - base_offset;
                OpcodeLocation::Brillig { acir_index, brillig_index }
            }
        };
        Some(location)
    }

    pub(super) fn render_opcode_at_location(&self, location: &OpcodeLocation) -> String {
        let opcodes = self.get_opcodes();
        match location {
            OpcodeLocation::Acir(acir_index) => {
                let opcode = &opcodes[*acir_index];
                match opcode {
                    Opcode::BrilligCall { id, .. } => {
                        let first_opcode = &self.unconstrained_functions[*id as usize].bytecode[0];
                        format!("BRILLIG {first_opcode:?}")
                    }
                    _ => format!("{opcode:?}"),
                }
            }
            OpcodeLocation::Brillig { acir_index, brillig_index } => match &opcodes[*acir_index] {
                Opcode::BrilligCall { id, .. } => {
                    let bytecode = &self.unconstrained_functions[*id as usize].bytecode;
                    let opcode = &bytecode[*brillig_index];
                    format!("      | {opcode:?}")
                }
                _ => String::from("      | invalid"),
            },
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
                // TODO: debugger does not not handle multiple acir calls
                ExecutionError::SolvingError(err, None),
            )),
        }
    }

    fn handle_foreign_call(
        &mut self,
        foreign_call: ForeignCallWaitInfo<FieldElement>,
    ) -> DebugCommandResult {
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

    fn handle_acvm_status(&mut self, status: ACVMStatus<FieldElement>) -> DebugCommandResult {
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
                // TODO: debugger does not not handle multiple acir calls
                ExecutionError::SolvingError(error, None),
            )),
            ACVMStatus::RequiresForeignCall(_) => {
                unreachable!("Unexpected pending foreign call resolution");
            }
            ACVMStatus::RequiresAcirCall(_) => {
                todo!("Multiple ACIR calls are not supported");
            }
        }
    }

    pub(super) fn step_into_opcode(&mut self) -> DebugCommandResult {
        if self.brillig_solver.is_some() {
            return self.step_brillig_opcode();
        }

        match self.acvm.step_into_brillig() {
            StepResult::IntoBrillig(solver) => {
                self.brillig_solver = Some(solver);
                self.step_brillig_opcode()
            }
            StepResult::Status(status) => self.handle_acvm_status(status),
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

    pub(super) fn is_executing_brillig(&self) -> bool {
        if self.brillig_solver.is_some() {
            return true;
        }

        match self.get_current_opcode_location() {
            Some(OpcodeLocation::Brillig { .. }) => true,
            Some(OpcodeLocation::Acir(acir_index)) => {
                matches!(self.get_opcodes()[acir_index], Opcode::BrilligCall { .. })
            }
            _ => false,
        }
    }

    pub(super) fn step_acir_opcode(&mut self) -> DebugCommandResult {
        if self.is_executing_brillig() {
            self.step_out_of_brillig_opcode()
        } else {
            let status = self.acvm.solve_opcode();
            self.handle_acvm_status(status)
        }
    }

    /// Steps debugging execution until the next source location
    pub(super) fn next_into(&mut self) -> DebugCommandResult {
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

    /// Steps debugging execution until the next source location at the same (or
    /// less) call stack depth (eg. don't dive into function calls)
    pub(super) fn next_over(&mut self) -> DebugCommandResult {
        let start_call_stack = self.get_source_call_stack();
        loop {
            let result = self.next_into();
            if !matches!(result, DebugCommandResult::Ok) {
                return result;
            }
            let new_call_stack = self.get_source_call_stack();
            if new_call_stack.len() <= start_call_stack.len() {
                return DebugCommandResult::Ok;
            }
        }
    }

    /// Steps debugging execution until the next source location with a smaller
    /// call stack depth (eg. returning from the current function)
    pub(super) fn next_out(&mut self) -> DebugCommandResult {
        let start_call_stack = self.get_source_call_stack();
        loop {
            let result = self.next_into();
            if !matches!(result, DebugCommandResult::Ok) {
                return result;
            }
            let new_call_stack = self.get_source_call_stack();
            if new_call_stack.len() < start_call_stack.len() {
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

    pub(super) fn get_brillig_memory(&self) -> Option<&[MemoryValue<FieldElement>]> {
        self.brillig_solver.as_ref().map(|solver| solver.get_memory())
    }

    pub(super) fn write_brillig_memory(&mut self, ptr: usize, value: FieldElement, bit_size: u32) {
        if let Some(solver) = self.brillig_solver.as_mut() {
            solver.write_memory_at(
                ptr,
                MemoryValue::new_checked(value, bit_size)
                    .expect("Invalid value for the given bit size"),
            );
        }
    }

    pub(super) fn get_variables(&self) -> Vec<StackFrame> {
        return self.foreign_call_executor.get_variables();
    }

    pub(super) fn current_stack_frame(&self) -> Option<StackFrame> {
        return self.foreign_call_executor.current_stack_frame();
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
                if acir_index < opcodes.len() {
                    match &opcodes[acir_index] {
                        Opcode::BrilligCall { id, .. } => {
                            let bytecode = &self.unconstrained_functions[*id as usize].bytecode;
                            brillig_index < bytecode.len()
                        }
                        _ => false,
                    }
                } else {
                    false
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

    pub fn finalize(self) -> WitnessMap<FieldElement> {
        self.acvm.finalize()
    }
}

fn is_debug_file_in_debug_crate(debug_file: &DebugFile) -> bool {
    debug_file.path.starts_with("__debug/")
}

/// Builds a map from FileId to an ordered vector of tuples with line
/// numbers and opcode locations corresponding to those line numbers
fn build_source_to_opcode_debug_mappings(
    debug_artifact: &DebugArtifact,
) -> BTreeMap<FileId, Vec<(usize, OpcodeLocation)>> {
    if debug_artifact.debug_symbols.is_empty() {
        return BTreeMap::new();
    }
    let locations = &debug_artifact.debug_symbols[0].locations;
    let simple_files: BTreeMap<_, _> = debug_artifact
        .file_map
        .iter()
        .filter(|(_, debug_file)| !is_debug_file_in_debug_crate(debug_file))
        .map(|(file_id, debug_file)| {
            (
                file_id,
                SimpleFile::new(debug_file.path.to_str().unwrap(), debug_file.source.as_str()),
            )
        })
        .collect();

    let mut result: BTreeMap<FileId, Vec<(usize, OpcodeLocation)>> = BTreeMap::new();
    locations.iter().for_each(|(opcode_location, source_locations)| {
        source_locations.iter().for_each(|source_location| {
            let span = source_location.span;
            let file_id = source_location.file;
            let Some(file) = simple_files.get(&file_id) else {
                return;
            };
            let Ok(line_index) = file.line_index((), span.start() as usize) else {
                return;
            };
            let line_number = line_index + 1;

            result.entry(file_id).or_default().push((line_number, *opcode_location));
        });
    });
    result.iter_mut().for_each(|(_, file_locations)| file_locations.sort_by_key(|x| (x.0, x.1)));

    result
}

fn build_acir_opcode_offsets(
    circuit: &Circuit<FieldElement>,
    unconstrained_functions: &[BrilligBytecode<FieldElement>],
) -> Vec<usize> {
    let mut result = Vec::with_capacity(circuit.opcodes.len() + 1);
    // address of the first opcode is always 0
    result.push(0);
    circuit.opcodes.iter().fold(0, |acc, opcode| {
        let acc = acc
            + match opcode {
                Opcode::BrilligCall { id, .. } => {
                    unconstrained_functions[*id as usize].bytecode.len()
                }
                _ => 1,
            };
        // push the starting address of the next opcode
        result.push(acc);
        acc
    });
    result
}

// TODO: update all debugger tests to use unconstrained brillig pointers
#[cfg(test)]
mod tests {
    use super::*;

    use crate::foreign_calls::DefaultDebugForeignCallExecutor;
    use acvm::{
        acir::{
            circuit::{
                brillig::{BrilligInputs, BrilligOutputs},
                opcodes::BlockId,
            },
            native_types::Expression,
            AcirField,
        },
        blackbox_solver::StubbedBlackBoxSolver,
        brillig_vm::brillig::{
            BinaryFieldOp, HeapValueType, MemoryAddress, Opcode as BrilligOpcode, ValueOrArray,
        },
    };

    #[test]
    fn test_resolve_foreign_calls_stepping_into_brillig() {
        let fe_0 = FieldElement::zero();
        let fe_1 = FieldElement::one();
        let w_x = Witness(1);

        let brillig_bytecode = BrilligBytecode {
            bytecode: vec![
                BrilligOpcode::CalldataCopy {
                    destination_address: MemoryAddress(0),
                    size: 1,
                    offset: 0,
                },
                BrilligOpcode::Const {
                    destination: MemoryAddress::from(1),
                    value: fe_0,
                    bit_size: 32,
                },
                BrilligOpcode::ForeignCall {
                    function: "clear_mock".into(),
                    destinations: vec![],
                    destination_value_types: vec![],
                    inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(0))],
                    input_value_types: vec![HeapValueType::field()],
                },
                BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 0 },
            ],
        };
        let opcodes = vec![Opcode::BrilligCall {
            id: 0,
            inputs: vec![BrilligInputs::Single(Expression {
                linear_combinations: vec![(fe_1, w_x)],
                ..Expression::default()
            })],
            outputs: vec![],
            predicate: None,
        }];
        let brillig_funcs = &vec![brillig_bytecode];
        let current_witness_index = 2;
        let circuit = &Circuit { current_witness_index, opcodes, ..Circuit::default() };

        let debug_symbols = vec![];
        let file_map = BTreeMap::new();
        let debug_artifact = &DebugArtifact { debug_symbols, file_map };

        let initial_witness = BTreeMap::from([(Witness(1), fe_1)]).into();

        let foreign_call_executor =
            Box::new(DefaultDebugForeignCallExecutor::from_artifact(true, debug_artifact));
        let mut context = DebugContext::new(
            &StubbedBlackBoxSolver,
            circuit,
            debug_artifact,
            initial_witness,
            foreign_call_executor,
            brillig_funcs,
        );

        assert_eq!(context.get_current_opcode_location(), Some(OpcodeLocation::Acir(0)));

        // Execute the first Brillig opcode (calldata copy)
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_opcode_location(),
            Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 1 })
        );

        // execute the second Brillig opcode (const)
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_opcode_location(),
            Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 2 })
        );

        // try to execute the third Brillig opcode (and resolve the foreign call)
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_opcode_location(),
            Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 2 })
        );

        // retry the third Brillig opcode (foreign call should be finished)
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_opcode_location(),
            Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 3 })
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
        let brillig_bytecode = BrilligBytecode {
            bytecode: vec![
                BrilligOpcode::CalldataCopy {
                    destination_address: MemoryAddress(0),
                    size: 2,
                    offset: 0,
                },
                BrilligOpcode::BinaryFieldOp {
                    destination: MemoryAddress::from(0),
                    op: BinaryFieldOp::Add,
                    lhs: MemoryAddress::from(0),
                    rhs: MemoryAddress::from(1),
                },
                BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 1 },
            ],
        };
        let opcodes = vec![
            // z = x + y
            Opcode::BrilligCall {
                id: 0,
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
                predicate: None,
            },
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
        let debug_artifact = &DebugArtifact { debug_symbols, file_map };

        let initial_witness = BTreeMap::from([(Witness(1), fe_1), (Witness(2), fe_1)]).into();

        let foreign_call_executor =
            Box::new(DefaultDebugForeignCallExecutor::from_artifact(true, debug_artifact));
        let brillig_funcs = &vec![brillig_bytecode];
        let mut context = DebugContext::new(
            &StubbedBlackBoxSolver,
            circuit,
            debug_artifact,
            initial_witness,
            foreign_call_executor,
            brillig_funcs,
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
    fn test_address_opcode_location_mapping() {
        let brillig_bytecode = BrilligBytecode {
            bytecode: vec![
                BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 0 },
                BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 0 },
                BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 0 },
            ],
        };

        let opcodes = vec![
            Opcode::BrilligCall { id: 0, inputs: vec![], outputs: vec![], predicate: None },
            Opcode::MemoryInit {
                block_id: BlockId(0),
                init: vec![],
                block_type: acvm::acir::circuit::opcodes::BlockType::Memory,
            },
            Opcode::BrilligCall { id: 0, inputs: vec![], outputs: vec![], predicate: None },
            Opcode::AssertZero(Expression::default()),
        ];
        let circuit = Circuit { opcodes, ..Circuit::default() };
        let debug_artifact = DebugArtifact { debug_symbols: vec![], file_map: BTreeMap::new() };
        let brillig_funcs = &vec![brillig_bytecode];
        let context = DebugContext::new(
            &StubbedBlackBoxSolver,
            &circuit,
            &debug_artifact,
            WitnessMap::new(),
            Box::new(DefaultDebugForeignCallExecutor::new(true)),
            brillig_funcs,
        );

        let locations =
            (0..=7).map(|address| context.address_to_opcode_location(address)).collect::<Vec<_>>();

        // mapping from addresses to opcode locations
        assert_eq!(
            locations,
            vec![
                Some(OpcodeLocation::Acir(0)),
                Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 1 }),
                Some(OpcodeLocation::Brillig { acir_index: 0, brillig_index: 2 }),
                Some(OpcodeLocation::Acir(1)),
                Some(OpcodeLocation::Acir(2)),
                Some(OpcodeLocation::Brillig { acir_index: 2, brillig_index: 1 }),
                Some(OpcodeLocation::Brillig { acir_index: 2, brillig_index: 2 }),
                Some(OpcodeLocation::Acir(3)),
            ]
        );

        let addresses = locations
            .iter()
            .flatten()
            .map(|location| context.opcode_location_to_address(location))
            .collect::<Vec<_>>();

        // and vice-versa
        assert_eq!(addresses, (0..=7).collect::<Vec<_>>());

        // check edge cases
        assert_eq!(None, context.address_to_opcode_location(8));
        assert_eq!(
            0,
            context.opcode_location_to_address(&OpcodeLocation::Brillig {
                acir_index: 0,
                brillig_index: 0
            })
        );
        assert_eq!(
            4,
            context.opcode_location_to_address(&OpcodeLocation::Brillig {
                acir_index: 2,
                brillig_index: 0
            })
        );
    }
}
