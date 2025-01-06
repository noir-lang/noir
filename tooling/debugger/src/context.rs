use crate::foreign_calls::DebugForeignCallExecutor;
use acvm::acir::brillig::BitSize;
use acvm::acir::circuit::brillig::{BrilligBytecode, BrilligFunctionId};
use acvm::acir::circuit::{Circuit, Opcode, OpcodeLocation};
use acvm::acir::native_types::{Witness, WitnessMap, WitnessStack};
use acvm::brillig_vm::MemoryValue;
use acvm::pwg::{
    ACVMStatus, AcirCallWaitInfo, BrilligSolver, BrilligSolverStatus, ForeignCallWaitInfo,
    OpcodeNotSolvable, StepResult, ACVM,
};
use acvm::{BlackBoxFunctionSolver, FieldElement};

use codespan_reporting::files::{Files, SimpleFile};
use fm::FileId;
use nargo::errors::{ExecutionError, Location};
use nargo::NargoError;
use noirc_artifacts::debug::{DebugArtifact, StackFrame};
use noirc_driver::DebugFile;

use thiserror::Error;

use std::collections::BTreeMap;
use std::collections::{hash_set::Iter, HashSet};

/// A Noir program is composed by
/// `n` ACIR circuits
///       |_ `m` ACIR opcodes
///                |_ Acir call
///                |_ Acir Brillig function invocation
///                           |_ `p` Brillig opcodes
///
/// The purpose of this structure is to map the opcode locations in ACIR circuits into
/// a flat contiguous address space to be able to expose them to the DAP interface.
/// In this address space, the ACIR circuits are laid out one after the other, and
/// Brillig functions called from such circuits are expanded inline, replacing
/// the `BrilligCall` ACIR opcode.
///
/// `addresses: Vec<Vec<usize>>`
///  * The outer vec is `n` sized - one element per ACIR circuit
///  * Each nested vec is `m` sized - one element per ACIR opcode in circuit
///    * Each element is the "virtual address" of such opcode
///
/// For flattening we map each ACIR circuit and ACIR opcode with a sequential address number
/// We start by assigning 0 to the very first ACIR opcode and then start accumulating by
/// traversing by depth-first
///
/// Even if the address space is continuous, the `addresses` tree only
/// keeps track of the ACIR opcodes, since the Brillig opcode addresses can be
/// calculated from the initial opcode address.
/// As a result the flattened indexed addresses list may have "holes".
///
/// If between two consequent `addresses` nodes there is a "hole" (an address jump),
/// this means that the first one is actually a ACIR Brillig call
/// which has as many brillig opcodes as `second_address - first_address`
///
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct AddressMap {
    addresses: Vec<Vec<usize>>,

    /// Virtual address of the last opcode of the program
    last_valid_address: usize,

    /// Maps the "holes" in the `addresses` nodes to the Brillig function ID
    /// associated with that address space.
    brillig_addresses: Vec<BrilligAddressSpace>,
}

/// Associates a BrilligFunctionId with the address space.
/// A BrilligFunctionId is found by checking whether an address is between
/// the `start_address` and `end_address`
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
struct BrilligAddressSpace {
    /// The start of the Brillig call address space
    start_address: usize,
    /// The end of the Brillig address space
    end_address: usize,
    /// The Brillig function id associated with this address space
    brillig_function_id: BrilligFunctionId,
}

impl AddressMap {
    pub(super) fn new(
        circuits: &[Circuit<FieldElement>],
        unconstrained_functions: &[BrilligBytecode<FieldElement>],
    ) -> Self {
        let opcode_address_size = |opcode: &Opcode<FieldElement>| {
            if let Opcode::BrilligCall { id, .. } = opcode {
                (unconstrained_functions[id.as_usize()].bytecode.len(), Some(*id))
            } else {
                (1, None)
            }
        };

        let mut addresses = Vec::with_capacity(circuits.len());
        let mut next_address = 0usize;
        let mut brillig_addresses = Vec::new();

        for circuit in circuits {
            let mut circuit_addresses = Vec::with_capacity(circuit.opcodes.len());
            for opcode in &circuit.opcodes {
                circuit_addresses.push(next_address);
                let (address_size, brillig_function_id) = opcode_address_size(opcode);
                if let Some(brillig_function_id) = brillig_function_id {
                    let brillig_address_space = BrilligAddressSpace {
                        start_address: next_address,
                        end_address: next_address + address_size,
                        brillig_function_id,
                    };
                    brillig_addresses.push(brillig_address_space);
                }
                next_address += address_size;
            }
            addresses.push(circuit_addresses);
        }

        Self { addresses, last_valid_address: next_address - 1, brillig_addresses }
    }

    /// Returns the absolute address of the opcode at the given location.
    /// Absolute here means accounting for nested Brillig opcodes in BrilligCall
    /// opcodes.
    pub fn debug_location_to_address(&self, location: &DebugLocation) -> usize {
        let circuit_addresses = &self.addresses[location.circuit_id as usize];
        match &location.opcode_location {
            OpcodeLocation::Acir(acir_index) => circuit_addresses[*acir_index],
            OpcodeLocation::Brillig { acir_index, brillig_index } => {
                circuit_addresses[*acir_index] + *brillig_index
            }
        }
    }

    pub fn address_to_debug_location(&self, address: usize) -> Option<DebugLocation> {
        if address > self.last_valid_address {
            return None;
        }
        // We binary search if the given address is the first opcode address of each circuit id
        // if is not, this means that the address itself is "contained" in the previous
        // circuit indicated by `Err(insert_index)`
        let circuit_id =
            match self.addresses.binary_search_by(|addresses| addresses[0].cmp(&address)) {
                Ok(found_index) => found_index,
                // This means that the address is not in `insert_index` circuit
                // because is an `Err`, so it must be included in previous circuit vec of opcodes
                Err(insert_index) => insert_index - 1,
            };

        // We binary search among the selected `circuit_id`` list of opcodes
        // If Err(insert_index) this means that the given address
        // is a Brillig addresses that's contained in previous index ACIR opcode index
        let (opcode_location, brillig_function_id) =
            match self.addresses[circuit_id].binary_search(&address) {
                Ok(found_index) => (OpcodeLocation::Acir(found_index), None),
                Err(insert_index) => {
                    let acir_index = insert_index - 1;
                    let base_offset = self.addresses[circuit_id][acir_index];
                    let brillig_index = address - base_offset;
                    let brillig_function_id = self
                        .brillig_addresses
                        .iter()
                        .find(|brillig_address_space| {
                            address >= brillig_address_space.start_address
                                && address <= brillig_address_space.end_address
                        })
                        .map(|brillig_address_space| brillig_address_space.brillig_function_id);
                    (OpcodeLocation::Brillig { acir_index, brillig_index }, brillig_function_id)
                }
            };

        Some(DebugLocation { circuit_id: circuit_id as u32, opcode_location, brillig_function_id })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct DebugLocation {
    pub circuit_id: u32,
    pub opcode_location: OpcodeLocation,
    pub brillig_function_id: Option<BrilligFunctionId>,
}

impl std::fmt::Display for DebugLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let circuit_id = self.circuit_id;
        match self.opcode_location {
            OpcodeLocation::Acir(index) => write!(f, "{circuit_id}:{index}"),
            OpcodeLocation::Brillig { acir_index, brillig_index } => {
                write!(f, "{circuit_id}:{acir_index}.{brillig_index}")
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum DebugLocationFromStrError {
    #[error("Invalid debug location string: {0}")]
    InvalidDebugLocationString(String),
}

impl std::str::FromStr for DebugLocation {
    type Err = DebugLocationFromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(':').collect();
        let error = Err(DebugLocationFromStrError::InvalidDebugLocationString(s.to_string()));

        match parts.len() {
            1 => OpcodeLocation::from_str(parts[0]).map_or(error, |opcode_location| {
                Ok(DebugLocation { circuit_id: 0, opcode_location, brillig_function_id: None })
            }),
            2 => {
                let first_part = parts[0].parse().ok();
                let second_part = OpcodeLocation::from_str(parts[1]).ok();
                if let (Some(circuit_id), Some(opcode_location)) = (first_part, second_part) {
                    Ok(DebugLocation { circuit_id, opcode_location, brillig_function_id: None })
                } else {
                    error
                }
            }
            _ => error,
        }
    }
}

#[derive(Debug)]
pub(super) enum DebugCommandResult {
    Done,
    Ok,
    BreakpointReached(DebugLocation),
    Error(NargoError<FieldElement>),
}

pub struct ExecutionFrame<'a, B: BlackBoxFunctionSolver<FieldElement>> {
    circuit_id: u32,
    acvm: ACVM<'a, FieldElement, B>,
}

pub(super) struct DebugContext<'a, B: BlackBoxFunctionSolver<FieldElement>> {
    acvm: ACVM<'a, FieldElement, B>,
    current_circuit_id: u32,
    brillig_solver: Option<BrilligSolver<'a, FieldElement, B>>,

    witness_stack: WitnessStack<FieldElement>,
    acvm_stack: Vec<ExecutionFrame<'a, B>>,

    backend: &'a B,
    foreign_call_executor: Box<dyn DebugForeignCallExecutor + 'a>,

    debug_artifact: &'a DebugArtifact,
    breakpoints: HashSet<DebugLocation>,
    source_to_locations: BTreeMap<FileId, Vec<(usize, DebugLocation)>>,

    circuits: &'a [Circuit<FieldElement>],
    unconstrained_functions: &'a [BrilligBytecode<FieldElement>],

    acir_opcode_addresses: AddressMap,
}

impl<'a, B: BlackBoxFunctionSolver<FieldElement>> DebugContext<'a, B> {
    pub(super) fn new(
        blackbox_solver: &'a B,
        circuits: &'a [Circuit<FieldElement>],
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap<FieldElement>,
        foreign_call_executor: Box<dyn DebugForeignCallExecutor + 'a>,
        unconstrained_functions: &'a [BrilligBytecode<FieldElement>],
    ) -> Self {
        let source_to_opcodes = build_source_to_opcode_debug_mappings(debug_artifact);
        let current_circuit_id: u32 = 0;
        let initial_circuit = &circuits[current_circuit_id as usize];
        let acir_opcode_addresses = AddressMap::new(circuits, unconstrained_functions);
        Self {
            acvm: ACVM::new(
                blackbox_solver,
                &initial_circuit.opcodes,
                initial_witness,
                unconstrained_functions,
                &initial_circuit.assert_messages,
            ),
            current_circuit_id,
            brillig_solver: None,
            witness_stack: WitnessStack::default(),
            acvm_stack: vec![],
            backend: blackbox_solver,
            foreign_call_executor,
            debug_artifact,
            breakpoints: HashSet::new(),
            source_to_locations: source_to_opcodes,
            circuits,
            unconstrained_functions,
            acir_opcode_addresses,
        }
    }

    pub(super) fn get_opcodes(&self) -> &[Opcode<FieldElement>] {
        self.acvm.opcodes()
    }

    pub(super) fn get_opcodes_of_circuit(&self, circuit_id: u32) -> &[Opcode<FieldElement>] {
        &self.circuits[circuit_id as usize].opcodes
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

    pub(super) fn get_current_debug_location(&self) -> Option<DebugLocation> {
        let ip = self.acvm.instruction_pointer();
        if ip >= self.get_opcodes().len() {
            None
        } else {
            let (opcode_location, brillig_function_id) =
                if let Some(ref solver) = self.brillig_solver {
                    let function_id = solver.function_id;
                    (
                        OpcodeLocation::Brillig {
                            acir_index: ip,
                            brillig_index: solver.program_counter(),
                        },
                        Some(function_id),
                    )
                } else {
                    (OpcodeLocation::Acir(ip), None)
                };
            Some(DebugLocation {
                circuit_id: self.current_circuit_id,
                brillig_function_id,
                opcode_location,
            })
        }
    }

    pub(super) fn get_call_stack(&self) -> Vec<DebugLocation> {
        // Build the frames from parent ACIR calls
        let mut frames: Vec<_> = self
            .acvm_stack
            .iter()
            .map(|ExecutionFrame { circuit_id, acvm }| DebugLocation {
                circuit_id: *circuit_id,
                opcode_location: OpcodeLocation::Acir(acvm.instruction_pointer()),
                brillig_function_id: None,
            })
            .collect();

        // Now add the frame(s) for the currently executing ACVM
        let instruction_pointer = self.acvm.instruction_pointer();
        let circuit_id = self.current_circuit_id;
        if let Some(ref solver) = self.brillig_solver {
            frames.extend(solver.get_call_stack().iter().map(|program_counter| DebugLocation {
                circuit_id,
                opcode_location: OpcodeLocation::Brillig {
                    acir_index: instruction_pointer,
                    brillig_index: *program_counter,
                },
                brillig_function_id: Some(solver.function_id),
            }));
        } else if instruction_pointer < self.get_opcodes().len() {
            frames.push(DebugLocation {
                circuit_id,
                opcode_location: OpcodeLocation::Acir(instruction_pointer),
                brillig_function_id: None,
            });
        }
        frames
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
    ) -> Option<DebugLocation> {
        let line = line as usize;
        let line_to_opcodes = self.source_to_locations.get(file_id)?;
        let found_location = match line_to_opcodes.binary_search_by(|x| x.0.cmp(&line)) {
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
        Some(found_location)
    }

    /// Returns the callstack in source code locations for the currently
    /// executing opcode. This can be `None` if the execution finished (and
    /// `get_current_opcode_location()` returns `None`) or if the opcode is not
    /// mapped to a specific source location in the debug artifact (which can
    /// happen for certain opcodes inserted synthetically by the compiler).
    /// This function also filters source locations that are determined to be in
    /// the internal debug module.
    pub(super) fn get_current_source_location(&self) -> Option<Vec<Location>> {
        self.get_current_debug_location()
            .as_ref()
            .map(|debug_location| self.get_source_location_for_debug_location(debug_location))
            .filter(|v: &Vec<Location>| !v.is_empty())
    }

    /// Returns the (possible) stack of source locations corresponding to the
    /// given opcode location. Due to compiler inlining it's possible for this
    /// function to return multiple source locations. An empty vector means that
    /// the given opcode location cannot be mapped back to a source location
    /// (eg. it may be pure debug instrumentation code or other synthetically
    /// produced opcode by the compiler)
    pub(super) fn get_source_location_for_debug_location(
        &self,
        debug_location: &DebugLocation,
    ) -> Vec<Location> {
        self.debug_artifact.debug_symbols[debug_location.circuit_id as usize]
            .opcode_location(&debug_location.opcode_location)
            .unwrap_or_else(|| {
                if let (Some(brillig_function_id), Some(brillig_location)) = (
                    debug_location.brillig_function_id,
                    debug_location.opcode_location.to_brillig_location(),
                ) {
                    let brillig_locations = self.debug_artifact.debug_symbols
                        [debug_location.circuit_id as usize]
                        .brillig_locations
                        .get(&brillig_function_id);
                    brillig_locations.unwrap().get(&brillig_location).cloned().unwrap_or_default()
                } else {
                    vec![]
                }
            })
            .into_iter()
            .filter(|source_location| !self.is_source_location_in_debug_module(source_location))
            .collect()
    }

    /// Returns the current call stack with expanded source locations. In
    /// general, the matching between opcode location and source location is 1
    /// to 1, but due to the compiler inlining functions a single opcode
    /// location may expand to multiple source locations.
    pub(super) fn get_source_call_stack(&self) -> Vec<(DebugLocation, Location)> {
        self.get_call_stack()
            .iter()
            .flat_map(|debug_location| {
                self.get_source_location_for_debug_location(debug_location)
                    .into_iter()
                    .map(|source_location| (*debug_location, source_location))
            })
            .collect()
    }

    /// Returns the absolute address of the opcode at the given location.
    pub fn debug_location_to_address(&self, location: &DebugLocation) -> usize {
        self.acir_opcode_addresses.debug_location_to_address(location)
    }

    // Returns the DebugLocation associated to the given address
    pub fn address_to_debug_location(&self, address: usize) -> Option<DebugLocation> {
        self.acir_opcode_addresses.address_to_debug_location(address)
    }

    pub(super) fn render_opcode_at_location(&self, location: &DebugLocation) -> String {
        let opcodes = self.get_opcodes_of_circuit(location.circuit_id);
        match &location.opcode_location {
            OpcodeLocation::Acir(acir_index) => {
                let opcode = &opcodes[*acir_index];
                match opcode {
                    Opcode::BrilligCall { id, .. } => {
                        let first_opcode = &self.unconstrained_functions[id.as_usize()].bytecode[0];
                        format!("BRILLIG {first_opcode:?}")
                    }
                    _ => format!("{opcode:?}"),
                }
            }
            OpcodeLocation::Brillig { acir_index, brillig_index } => match &opcodes[*acir_index] {
                Opcode::BrilligCall { id, .. } => {
                    let bytecode = &self.unconstrained_functions[id.as_usize()].bytecode;
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
                        self.get_current_debug_location()
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
                // TODO: should we retry executing the opcode somehow in this
                // case? Otherwise, executing a foreign call takes two debugging
                // steps.
                DebugCommandResult::Ok
            }
            Err(error) => DebugCommandResult::Error(error.into()),
        }
    }

    fn handle_acir_call(
        &mut self,
        call_info: AcirCallWaitInfo<FieldElement>,
    ) -> DebugCommandResult {
        let callee_circuit = &self.circuits[call_info.id.as_usize()];
        let callee_witness_map = call_info.initial_witness;
        let callee_acvm = ACVM::new(
            self.backend,
            &callee_circuit.opcodes,
            callee_witness_map,
            self.unconstrained_functions,
            &callee_circuit.assert_messages,
        );
        let caller_acvm = std::mem::replace(&mut self.acvm, callee_acvm);
        self.acvm_stack
            .push(ExecutionFrame { circuit_id: self.current_circuit_id, acvm: caller_acvm });
        self.current_circuit_id = call_info.id.0;

        // Explicitly handling the new ACVM status here handles two edge cases:
        // 1. there is a breakpoint set at the beginning of a circuit
        // 2. the called circuit has no opcodes
        self.handle_acvm_status(self.acvm.get_status().clone())
    }

    fn handle_acir_call_finished(&mut self) -> DebugCommandResult {
        let caller_frame = self.acvm_stack.pop().expect("Execution stack should not be empty");
        let caller_acvm = caller_frame.acvm;
        let callee_acvm = std::mem::replace(&mut self.acvm, caller_acvm);
        self.current_circuit_id = caller_frame.circuit_id;
        let call_solved_witness = callee_acvm.finalize();

        let ACVMStatus::RequiresAcirCall(call_info) = self.acvm.get_status() else {
            unreachable!("Resolving an ACIR call, the caller is in an invalid state");
        };
        let acir_to_call = &self.circuits[call_info.id.as_usize()];

        let mut call_resolved_outputs = Vec::new();
        for return_witness_index in acir_to_call.return_values.indices() {
            if let Some(return_value) = call_solved_witness.get_index(return_witness_index) {
                call_resolved_outputs.push(*return_value);
            } else {
                return DebugCommandResult::Error(
                    ExecutionError::SolvingError(
                        OpcodeNotSolvable::MissingAssignment(return_witness_index).into(),
                        None, // Missing assignment errors do not supply user-facing diagnostics so we do not need to attach a call stack
                    )
                    .into(),
                );
            }
        }
        self.acvm.resolve_pending_acir_call(call_resolved_outputs);

        DebugCommandResult::Ok
    }

    fn handle_acvm_status(&mut self, status: ACVMStatus<FieldElement>) -> DebugCommandResult {
        match status {
            ACVMStatus::Solved => {
                if self.acvm_stack.is_empty() {
                    return DebugCommandResult::Done;
                }
                self.handle_acir_call_finished()
            }
            ACVMStatus::InProgress => {
                if self.breakpoint_reached() {
                    DebugCommandResult::BreakpointReached(
                        self.get_current_debug_location()
                            .expect("Breakpoint reached but we have no location"),
                    )
                } else {
                    DebugCommandResult::Ok
                }
            }
            ACVMStatus::Failure(error) => DebugCommandResult::Error(NargoError::ExecutionError(
                ExecutionError::SolvingError(error, None),
            )),
            ACVMStatus::RequiresForeignCall(foreign_call) => self.handle_foreign_call(foreign_call),
            ACVMStatus::RequiresAcirCall(call_info) => self.handle_acir_call(call_info),
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
        self.get_current_debug_location().map(|debug_location| {
            match debug_location.opcode_location {
                OpcodeLocation::Acir(acir_index) | OpcodeLocation::Brillig { acir_index, .. } => {
                    acir_index
                }
            }
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

        match self.get_current_debug_location() {
            Some(DebugLocation { opcode_location: OpcodeLocation::Brillig { .. }, .. }) => true,
            Some(DebugLocation {
                circuit_id,
                opcode_location: OpcodeLocation::Acir(acir_index),
                ..
            }) => {
                matches!(
                    self.get_opcodes_of_circuit(circuit_id)[acir_index],
                    Opcode::BrilligCall { .. }
                )
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

    pub(super) fn write_brillig_memory(
        &mut self,
        ptr: usize,
        value: FieldElement,
        bit_size: BitSize,
    ) {
        if let Some(solver) = self.brillig_solver.as_mut() {
            solver.write_memory_at(
                ptr,
                MemoryValue::new_checked(value, bit_size)
                    .expect("Invalid value for the given bit size"),
            );
        }
    }

    pub(super) fn get_variables(&self) -> Vec<StackFrame<FieldElement>> {
        return self.foreign_call_executor.get_variables();
    }

    pub(super) fn current_stack_frame(&self) -> Option<StackFrame<FieldElement>> {
        return self.foreign_call_executor.current_stack_frame();
    }

    fn breakpoint_reached(&self) -> bool {
        if let Some(location) = self.get_current_debug_location() {
            self.breakpoints.contains(&location)
        } else {
            false
        }
    }

    pub(super) fn is_valid_debug_location(&self, location: &DebugLocation) -> bool {
        if location.circuit_id as usize >= self.circuits.len() {
            return false;
        }
        let opcodes = self.get_opcodes_of_circuit(location.circuit_id);
        match location.opcode_location {
            OpcodeLocation::Acir(acir_index) => acir_index < opcodes.len(),
            OpcodeLocation::Brillig { acir_index, brillig_index } => {
                if acir_index < opcodes.len() {
                    match &opcodes[acir_index] {
                        Opcode::BrilligCall { id, .. } => {
                            let bytecode = &self.unconstrained_functions[id.as_usize()].bytecode;
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

    pub(super) fn is_breakpoint_set(&self, location: &DebugLocation) -> bool {
        self.breakpoints.contains(location)
    }

    pub(super) fn add_breakpoint(&mut self, location: DebugLocation) -> bool {
        self.breakpoints.insert(location)
    }

    pub(super) fn delete_breakpoint(&mut self, location: &DebugLocation) -> bool {
        self.breakpoints.remove(location)
    }

    pub(super) fn iterate_breakpoints(&self) -> Iter<'_, DebugLocation> {
        self.breakpoints.iter()
    }

    pub(super) fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    pub(super) fn is_solved(&self) -> bool {
        matches!(self.acvm.get_status(), ACVMStatus::Solved)
    }

    pub fn finalize(mut self) -> WitnessStack<FieldElement> {
        let last_witness_map = self.acvm.finalize();
        self.witness_stack.push(0, last_witness_map);
        self.witness_stack
    }
}

fn is_debug_file_in_debug_crate(debug_file: &DebugFile) -> bool {
    debug_file.path.starts_with("__debug/")
}

/// Builds a map from FileId to an ordered vector of tuples with line
/// numbers and opcode locations corresponding to those line numbers
fn build_source_to_opcode_debug_mappings(
    debug_artifact: &DebugArtifact,
) -> BTreeMap<FileId, Vec<(usize, DebugLocation)>> {
    if debug_artifact.debug_symbols.is_empty() {
        return BTreeMap::new();
    }
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

    let mut result: BTreeMap<FileId, Vec<(usize, DebugLocation)>> = BTreeMap::new();

    for (circuit_id, debug_symbols) in debug_artifact.debug_symbols.iter().enumerate() {
        add_opcode_locations_map(
            &debug_symbols.locations,
            &mut result,
            &simple_files,
            circuit_id,
            None,
        );

        for (brillig_function_id, brillig_locations_map) in &debug_symbols.brillig_locations {
            let brillig_locations_map = brillig_locations_map
                .iter()
                .map(|(key, val)| {
                    (
                        // TODO: this is a temporary placeholder until the debugger is updated to handle the new brillig debug locations.
                        OpcodeLocation::Brillig { acir_index: 0, brillig_index: key.0 },
                        val.clone(),
                    )
                })
                .collect();

            add_opcode_locations_map(
                &brillig_locations_map,
                &mut result,
                &simple_files,
                circuit_id,
                Some(*brillig_function_id),
            );
        }
    }
    result.iter_mut().for_each(|(_, file_locations)| file_locations.sort_by_key(|x| (x.0, x.1)));

    result
}

fn add_opcode_locations_map(
    opcode_to_locations: &BTreeMap<OpcodeLocation, Vec<Location>>,
    source_to_locations: &mut BTreeMap<FileId, Vec<(usize, DebugLocation)>>,
    simple_files: &BTreeMap<&FileId, SimpleFile<&str, &str>>,
    circuit_id: usize,
    brillig_function_id: Option<BrilligFunctionId>,
) {
    for (opcode_location, source_locations) in opcode_to_locations {
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

            let debug_location = DebugLocation {
                circuit_id: circuit_id as u32,
                opcode_location: *opcode_location,
                brillig_function_id,
            };
            source_to_locations.entry(file_id).or_default().push((line_number, debug_location));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::foreign_calls::DefaultDebugForeignCallExecutor;
    use acvm::{
        acir::{
            brillig::{HeapVector, IntegerBitSize},
            circuit::{
                brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs},
                opcodes::{AcirFunctionId, BlockId, BlockType},
            },
            native_types::Expression,
            AcirField,
        },
        blackbox_solver::StubbedBlackBoxSolver,
        brillig_vm::brillig::{
            BinaryFieldOp, HeapValueType, MemoryAddress, Opcode as BrilligOpcode, ValueOrArray,
        },
    };
    use nargo::PrintOutput;

    #[test]
    fn test_resolve_foreign_calls_stepping_into_brillig() {
        let fe_1 = FieldElement::one();
        let w_x = Witness(1);

        let brillig_bytecode = BrilligBytecode {
            bytecode: vec![
                BrilligOpcode::Const {
                    destination: MemoryAddress::direct(1),
                    bit_size: BitSize::Integer(IntegerBitSize::U32),
                    value: FieldElement::from(1u64),
                },
                BrilligOpcode::Const {
                    destination: MemoryAddress::direct(2),
                    bit_size: BitSize::Integer(IntegerBitSize::U32),
                    value: FieldElement::from(0u64),
                },
                BrilligOpcode::CalldataCopy {
                    destination_address: MemoryAddress::direct(0),
                    size_address: MemoryAddress::direct(1),
                    offset_address: MemoryAddress::direct(2),
                },
                BrilligOpcode::ForeignCall {
                    function: "clear_mock".into(),
                    destinations: vec![],
                    destination_value_types: vec![],
                    inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(0))],
                    input_value_types: vec![HeapValueType::field()],
                },
                BrilligOpcode::Stop {
                    return_data: HeapVector {
                        pointer: MemoryAddress::direct(2),
                        size: MemoryAddress::direct(2),
                    },
                },
            ],
        };
        let opcodes = vec![Opcode::BrilligCall {
            id: BrilligFunctionId(0),
            inputs: vec![BrilligInputs::Single(Expression {
                linear_combinations: vec![(fe_1, w_x)],
                ..Expression::default()
            })],
            outputs: vec![],
            predicate: None,
        }];
        let brillig_funcs = &[brillig_bytecode];
        let current_witness_index = 2;
        let circuit = Circuit { current_witness_index, opcodes, ..Circuit::default() };
        let circuits = &[circuit];

        let debug_symbols = vec![];
        let file_map = BTreeMap::new();
        let debug_artifact = &DebugArtifact { debug_symbols, file_map };

        let initial_witness = BTreeMap::from([(Witness(1), fe_1)]).into();

        let foreign_call_executor = Box::new(DefaultDebugForeignCallExecutor::from_artifact(
            PrintOutput::Stdout,
            debug_artifact,
        ));
        let mut context = DebugContext::new(
            &StubbedBlackBoxSolver,
            circuits,
            debug_artifact,
            initial_witness,
            foreign_call_executor,
            brillig_funcs,
        );

        assert_eq!(
            context.get_current_debug_location(),
            Some(DebugLocation {
                circuit_id: 0,
                opcode_location: OpcodeLocation::Acir(0),
                brillig_function_id: None,
            })
        );

        // Const
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_debug_location(),
            Some(DebugLocation {
                circuit_id: 0,
                opcode_location: OpcodeLocation::Brillig { acir_index: 0, brillig_index: 1 },
                brillig_function_id: Some(BrilligFunctionId(0)),
            })
        );

        // Const
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_debug_location(),
            Some(DebugLocation {
                circuit_id: 0,
                opcode_location: OpcodeLocation::Brillig { acir_index: 0, brillig_index: 2 },
                brillig_function_id: Some(BrilligFunctionId(0)),
            })
        );

        // Calldatacopy
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_debug_location(),
            Some(DebugLocation {
                circuit_id: 0,
                opcode_location: OpcodeLocation::Brillig { acir_index: 0, brillig_index: 3 },
                brillig_function_id: Some(BrilligFunctionId(0)),
            })
        );

        // try to execute the Brillig opcode (and resolve the foreign call)
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_debug_location(),
            Some(DebugLocation {
                circuit_id: 0,
                opcode_location: OpcodeLocation::Brillig { acir_index: 0, brillig_index: 3 },
                brillig_function_id: Some(BrilligFunctionId(0)),
            })
        );

        // retry the Brillig opcode (foreign call should be finished)
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_debug_location(),
            Some(DebugLocation {
                circuit_id: 0,
                opcode_location: OpcodeLocation::Brillig { acir_index: 0, brillig_index: 4 },
                brillig_function_id: Some(BrilligFunctionId(0)),
            })
        );

        // last Brillig opcode
        let result = context.step_into_opcode();
        assert!(matches!(result, DebugCommandResult::Done));
        assert_eq!(context.get_current_debug_location(), None);
    }

    #[test]
    fn test_break_brillig_block_while_stepping_acir_opcodes() {
        let fe_0 = FieldElement::zero();
        let fe_1 = FieldElement::one();
        let w_x = Witness(1);
        let w_y = Witness(2);
        let w_z = Witness(3);

        let zero_usize = MemoryAddress::direct(2);
        let one_usize = MemoryAddress::direct(3);

        // This Brillig block is equivalent to: z = x + y
        let brillig_bytecode = BrilligBytecode {
            bytecode: vec![
                BrilligOpcode::Const {
                    destination: MemoryAddress::direct(0),
                    bit_size: BitSize::Integer(IntegerBitSize::U32),
                    value: FieldElement::from(2u64),
                },
                BrilligOpcode::Const {
                    destination: zero_usize,
                    bit_size: BitSize::Integer(IntegerBitSize::U32),
                    value: FieldElement::from(0u64),
                },
                BrilligOpcode::Const {
                    destination: one_usize,
                    bit_size: BitSize::Integer(IntegerBitSize::U32),
                    value: FieldElement::from(1u64),
                },
                BrilligOpcode::CalldataCopy {
                    destination_address: MemoryAddress::direct(0),
                    size_address: MemoryAddress::direct(0),
                    offset_address: zero_usize,
                },
                BrilligOpcode::BinaryFieldOp {
                    destination: MemoryAddress::direct(0),
                    op: BinaryFieldOp::Add,
                    lhs: MemoryAddress::direct(0),
                    rhs: MemoryAddress::direct(1),
                },
                BrilligOpcode::Stop {
                    return_data: HeapVector { pointer: zero_usize, size: one_usize },
                },
            ],
        };
        let opcodes = vec![
            // z = x + y
            Opcode::BrilligCall {
                id: BrilligFunctionId(0),
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
        let circuit = Circuit { current_witness_index, opcodes, ..Circuit::default() };
        let circuits = &[circuit];

        let debug_symbols = vec![];
        let file_map = BTreeMap::new();
        let debug_artifact = &DebugArtifact { debug_symbols, file_map };

        let initial_witness = BTreeMap::from([(Witness(1), fe_1), (Witness(2), fe_1)]).into();

        let foreign_call_executor = Box::new(DefaultDebugForeignCallExecutor::from_artifact(
            PrintOutput::Stdout,
            debug_artifact,
        ));
        let brillig_funcs = &[brillig_bytecode];
        let mut context = DebugContext::new(
            &StubbedBlackBoxSolver,
            circuits,
            debug_artifact,
            initial_witness,
            foreign_call_executor,
            brillig_funcs,
        );

        // set breakpoint
        let breakpoint_location = DebugLocation {
            circuit_id: 0,
            opcode_location: OpcodeLocation::Brillig { acir_index: 0, brillig_index: 1 },
            brillig_function_id: Some(BrilligFunctionId(0)),
        };
        assert!(context.add_breakpoint(breakpoint_location));

        // execute the first ACIR opcode (Brillig block) -> should reach the breakpoint instead
        let result = context.step_acir_opcode();
        assert!(matches!(result, DebugCommandResult::BreakpointReached(_)));
        assert_eq!(context.get_current_debug_location(), Some(breakpoint_location));

        // continue execution to the next ACIR opcode
        let result = context.step_acir_opcode();
        assert!(matches!(result, DebugCommandResult::Ok));
        assert_eq!(
            context.get_current_debug_location(),
            Some(DebugLocation {
                circuit_id: 0,
                opcode_location: OpcodeLocation::Acir(1),
                brillig_function_id: None
            })
        );

        // last ACIR opcode
        let result = context.step_acir_opcode();
        assert!(matches!(result, DebugCommandResult::Done));
        assert_eq!(context.get_current_debug_location(), None);
    }

    #[test]
    fn test_address_debug_location_mapping() {
        let brillig_one =
            BrilligBytecode { bytecode: vec![BrilligOpcode::Return, BrilligOpcode::Return] };
        let brillig_two = BrilligBytecode {
            bytecode: vec![BrilligOpcode::Return, BrilligOpcode::Return, BrilligOpcode::Return],
        };

        let circuit_one = Circuit {
            opcodes: vec![
                Opcode::MemoryInit {
                    block_id: BlockId(0),
                    init: vec![],
                    block_type: BlockType::Memory,
                },
                Opcode::BrilligCall {
                    id: BrilligFunctionId(0),
                    inputs: vec![],
                    outputs: vec![],
                    predicate: None,
                },
                Opcode::Call {
                    id: AcirFunctionId(1),
                    inputs: vec![],
                    outputs: vec![],
                    predicate: None,
                },
                Opcode::AssertZero(Expression::default()),
            ],
            ..Circuit::default()
        };
        let circuit_two = Circuit {
            opcodes: vec![
                Opcode::BrilligCall {
                    id: BrilligFunctionId(1),
                    inputs: vec![],
                    outputs: vec![],
                    predicate: None,
                },
                Opcode::AssertZero(Expression::default()),
            ],
            ..Circuit::default()
        };
        let circuits = vec![circuit_one, circuit_two];
        let debug_artifact = DebugArtifact { debug_symbols: vec![], file_map: BTreeMap::new() };
        let brillig_funcs = &[brillig_one, brillig_two];

        let context = DebugContext::new(
            &StubbedBlackBoxSolver,
            &circuits,
            &debug_artifact,
            WitnessMap::new(),
            Box::new(DefaultDebugForeignCallExecutor::new(PrintOutput::Stdout)),
            brillig_funcs,
        );

        let locations =
            (0..=8).map(|address| context.address_to_debug_location(address)).collect::<Vec<_>>();

        // mapping from addresses to opcode locations
        assert_eq!(
            locations,
            vec![
                Some(DebugLocation {
                    circuit_id: 0,
                    opcode_location: OpcodeLocation::Acir(0),
                    brillig_function_id: None
                }),
                Some(DebugLocation {
                    circuit_id: 0,
                    opcode_location: OpcodeLocation::Acir(1),
                    brillig_function_id: None
                }),
                Some(DebugLocation {
                    circuit_id: 0,
                    opcode_location: OpcodeLocation::Brillig { acir_index: 1, brillig_index: 1 },
                    brillig_function_id: Some(BrilligFunctionId(0)),
                }),
                Some(DebugLocation {
                    circuit_id: 0,
                    opcode_location: OpcodeLocation::Acir(2),
                    brillig_function_id: None
                }),
                Some(DebugLocation {
                    circuit_id: 0,
                    opcode_location: OpcodeLocation::Acir(3),
                    brillig_function_id: None
                }),
                Some(DebugLocation {
                    circuit_id: 1,
                    opcode_location: OpcodeLocation::Acir(0),
                    brillig_function_id: None
                }),
                Some(DebugLocation {
                    circuit_id: 1,
                    opcode_location: OpcodeLocation::Brillig { acir_index: 0, brillig_index: 1 },
                    brillig_function_id: Some(BrilligFunctionId(1)),
                }),
                Some(DebugLocation {
                    circuit_id: 1,
                    opcode_location: OpcodeLocation::Brillig { acir_index: 0, brillig_index: 2 },
                    brillig_function_id: Some(BrilligFunctionId(1)),
                }),
                Some(DebugLocation {
                    circuit_id: 1,
                    opcode_location: OpcodeLocation::Acir(1),
                    brillig_function_id: None
                }),
            ]
        );

        let addresses = locations
            .iter()
            .flatten()
            .map(|location| context.debug_location_to_address(location))
            .collect::<Vec<_>>();

        // and vice-versa
        assert_eq!(addresses, (0..=8).collect::<Vec<_>>());

        // check edge cases
        assert_eq!(None, context.address_to_debug_location(9));
        assert_eq!(
            1,
            context.debug_location_to_address(&DebugLocation {
                circuit_id: 0,
                opcode_location: OpcodeLocation::Brillig { acir_index: 1, brillig_index: 0 },
                brillig_function_id: Some(BrilligFunctionId(0)),
            })
        );
        assert_eq!(
            5,
            context.debug_location_to_address(&DebugLocation {
                circuit_id: 1,
                opcode_location: OpcodeLocation::Brillig { acir_index: 0, brillig_index: 0 },
                brillig_function_id: Some(BrilligFunctionId(1)),
            })
        );
    }
}
