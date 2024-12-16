use crate::context::{DebugCommandResult, DebugContext, DebugLocation};

use acvm::acir::brillig::BitSize;
use acvm::acir::circuit::brillig::{BrilligBytecode, BrilligFunctionId};
use acvm::acir::circuit::{Circuit, Opcode, OpcodeLocation};
use acvm::acir::native_types::{Witness, WitnessMap, WitnessStack};
use acvm::brillig_vm::brillig::Opcode as BrilligOpcode;
use acvm::brillig_vm::MemoryValue;
use acvm::AcirField;
use acvm::{BlackBoxFunctionSolver, FieldElement};
use nargo::{NargoError, PrintOutput};
use noirc_driver::CompiledProgram;

use crate::foreign_calls::DefaultDebugForeignCallExecutor;
use noirc_artifacts::debug::DebugArtifact;

use easy_repl::{command, CommandStatus, Repl};
use noirc_printable_type::PrintableValueDisplay;
use std::cell::RefCell;

use crate::source_code_printer::print_source_code_location;

pub struct ReplDebugger<'a, B: BlackBoxFunctionSolver<FieldElement>> {
    context: DebugContext<'a, B>,
    blackbox_solver: &'a B,
    debug_artifact: &'a DebugArtifact,
    initial_witness: WitnessMap<FieldElement>,
    last_result: DebugCommandResult,

    // ACIR functions to debug
    circuits: &'a [Circuit<FieldElement>],

    // Brillig functions referenced from the ACIR circuits above
    unconstrained_functions: &'a [BrilligBytecode<FieldElement>],
}

impl<'a, B: BlackBoxFunctionSolver<FieldElement>> ReplDebugger<'a, B> {
    pub fn new(
        blackbox_solver: &'a B,
        circuits: &'a [Circuit<FieldElement>],
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap<FieldElement>,
        unconstrained_functions: &'a [BrilligBytecode<FieldElement>],
    ) -> Self {
        let foreign_call_executor = Box::new(DefaultDebugForeignCallExecutor::from_artifact(
            PrintOutput::Stdout,
            debug_artifact,
        ));
        let context = DebugContext::new(
            blackbox_solver,
            circuits,
            debug_artifact,
            initial_witness.clone(),
            foreign_call_executor,
            unconstrained_functions,
        );
        let last_result = if context.get_current_debug_location().is_none() {
            // handle circuit with no opcodes
            DebugCommandResult::Done
        } else {
            DebugCommandResult::Ok
        };
        Self {
            context,
            blackbox_solver,
            circuits,
            debug_artifact,
            initial_witness,
            last_result,
            unconstrained_functions,
        }
    }

    pub fn show_current_vm_status(&self) {
        let location = self.context.get_current_debug_location();

        match location {
            None => println!("Finished execution"),
            Some(location) => {
                let circuit_id = location.circuit_id;
                let opcodes = self.context.get_opcodes_of_circuit(circuit_id);
                match &location.opcode_location {
                    OpcodeLocation::Acir(ip) => {
                        println!("At opcode {} :: {}", location, opcodes[*ip]);
                    }
                    OpcodeLocation::Brillig { acir_index, brillig_index } => {
                        let brillig_bytecode =
                            if let Opcode::BrilligCall { id, .. } = opcodes[*acir_index] {
                                &self.unconstrained_functions[id.as_usize()].bytecode
                            } else {
                                unreachable!("Brillig location does not contain Brillig opcodes");
                            };
                        println!(
                            "At opcode {} :: {:?}",
                            location, brillig_bytecode[*brillig_index]
                        );
                    }
                }
                let locations = self.context.get_source_location_for_debug_location(&location);
                print_source_code_location(self.debug_artifact, &locations);
            }
        }
    }

    fn show_stack_frame(&self, index: usize, debug_location: &DebugLocation) {
        let opcodes = self.context.get_opcodes();
        match &debug_location.opcode_location {
            OpcodeLocation::Acir(instruction_pointer) => {
                println!(
                    "Frame #{index}, opcode {} :: {}",
                    debug_location, opcodes[*instruction_pointer]
                )
            }
            OpcodeLocation::Brillig { acir_index, brillig_index } => {
                let brillig_bytecode = if let Opcode::BrilligCall { id, .. } = opcodes[*acir_index]
                {
                    &self.unconstrained_functions[id.as_usize()].bytecode
                } else {
                    unreachable!("Brillig location does not contain Brillig opcodes");
                };
                println!(
                    "Frame #{index}, opcode {} :: {:?}",
                    debug_location, brillig_bytecode[*brillig_index]
                );
            }
        }
        let locations = self.context.get_source_location_for_debug_location(debug_location);
        print_source_code_location(self.debug_artifact, &locations);
    }

    pub fn show_current_call_stack(&self) {
        let call_stack = self.context.get_call_stack();
        if call_stack.is_empty() {
            println!("Finished execution. Call stack empty.");
            return;
        }

        for (i, frame_location) in call_stack.iter().enumerate() {
            self.show_stack_frame(i, frame_location);
        }
    }

    fn display_opcodes(&self) {
        for i in 0..self.circuits.len() {
            self.display_opcodes_of_circuit(i as u32);
        }
    }

    fn display_opcodes_of_circuit(&self, circuit_id: u32) {
        let current_opcode_location =
            self.context.get_current_debug_location().and_then(|debug_location| {
                if debug_location.circuit_id == circuit_id {
                    Some(debug_location.opcode_location)
                } else {
                    None
                }
            });
        let opcodes = self.context.get_opcodes_of_circuit(circuit_id);
        let current_acir_index = match current_opcode_location {
            Some(OpcodeLocation::Acir(ip)) => Some(ip),
            Some(OpcodeLocation::Brillig { acir_index, .. }) => Some(acir_index),
            None => None,
        };
        let current_brillig_index = match current_opcode_location {
            Some(OpcodeLocation::Brillig { brillig_index, .. }) => brillig_index,
            _ => 0,
        };
        let outer_marker = |acir_index| {
            if current_acir_index == Some(acir_index) {
                "->"
            } else if self.context.is_breakpoint_set(&DebugLocation {
                circuit_id,
                opcode_location: OpcodeLocation::Acir(acir_index),
                brillig_function_id: None,
            }) {
                " *"
            } else {
                ""
            }
        };
        let brillig_marker = |acir_index, brillig_index, brillig_function_id| {
            if current_acir_index == Some(acir_index) && brillig_index == current_brillig_index {
                "->"
            } else if self.context.is_breakpoint_set(&DebugLocation {
                circuit_id,
                opcode_location: OpcodeLocation::Brillig { acir_index, brillig_index },
                brillig_function_id: Some(brillig_function_id),
            }) {
                " *"
            } else {
                ""
            }
        };
        let print_brillig_bytecode =
            |acir_index,
             bytecode: &[BrilligOpcode<FieldElement>],
             brillig_function_id: BrilligFunctionId| {
                for (brillig_index, brillig_opcode) in bytecode.iter().enumerate() {
                    println!(
                        "{:>2}:{:>3}.{:<2} |{:2} {:?}",
                        circuit_id,
                        acir_index,
                        brillig_index,
                        brillig_marker(acir_index, brillig_index, brillig_function_id),
                        brillig_opcode
                    );
                }
            };
        for (acir_index, opcode) in opcodes.iter().enumerate() {
            let marker = outer_marker(acir_index);
            match &opcode {
                Opcode::BrilligCall { id, inputs, outputs, .. } => {
                    println!(
                        "{:>2}:{:>3} {:2} BRILLIG CALL id={} inputs={:?}",
                        circuit_id, acir_index, marker, id, inputs
                    );
                    println!("          |       outputs={:?}", outputs);
                    let bytecode = &self.unconstrained_functions[id.as_usize()].bytecode;
                    print_brillig_bytecode(acir_index, bytecode, *id);
                }
                _ => println!("{:>2}:{:>3} {:2} {:?}", circuit_id, acir_index, marker, opcode),
            }
        }
    }

    fn add_breakpoint_at(&mut self, location: DebugLocation) {
        if !self.context.is_valid_debug_location(&location) {
            println!("Invalid location {location}");
        } else if self.context.add_breakpoint(location) {
            println!("Added breakpoint at {location}");
        } else {
            println!("Breakpoint at {location} already set");
        }
    }

    fn delete_breakpoint_at(&mut self, location: DebugLocation) {
        if self.context.delete_breakpoint(&location) {
            println!("Breakpoint at {location} deleted");
        } else {
            println!("Breakpoint at {location} not set");
        }
    }

    fn validate_in_progress(&self) -> bool {
        match self.last_result {
            DebugCommandResult::Ok | DebugCommandResult::BreakpointReached(..) => true,
            DebugCommandResult::Done => {
                println!("Execution finished");
                false
            }
            DebugCommandResult::Error(ref error) => {
                println!("ERROR: {}", error);
                self.show_current_vm_status();
                false
            }
        }
    }

    fn handle_debug_command_result(&mut self, result: DebugCommandResult) {
        match &result {
            DebugCommandResult::BreakpointReached(location) => {
                println!("Stopped at breakpoint in opcode {}", location);
            }
            DebugCommandResult::Error(error) => {
                println!("ERROR: {}", error);
            }
            _ => (),
        }
        self.last_result = result;
        self.show_current_vm_status();
    }

    fn step_acir_opcode(&mut self) {
        if self.validate_in_progress() {
            let result = self.context.step_acir_opcode();
            self.handle_debug_command_result(result);
        }
    }

    fn step_into_opcode(&mut self) {
        if self.validate_in_progress() {
            let result = self.context.step_into_opcode();
            self.handle_debug_command_result(result);
        }
    }

    fn next_into(&mut self) {
        if self.validate_in_progress() {
            let result = self.context.next_into();
            self.handle_debug_command_result(result);
        }
    }

    fn next_over(&mut self) {
        if self.validate_in_progress() {
            let result = self.context.next_over();
            self.handle_debug_command_result(result);
        }
    }

    fn next_out(&mut self) {
        if self.validate_in_progress() {
            let result = self.context.next_out();
            self.handle_debug_command_result(result);
        }
    }

    fn cont(&mut self) {
        if self.validate_in_progress() {
            println!("(Continuing execution...)");
            let result = self.context.cont();
            self.handle_debug_command_result(result);
        }
    }

    fn restart_session(&mut self) {
        let breakpoints: Vec<DebugLocation> = self.context.iterate_breakpoints().copied().collect();
        let foreign_call_executor = Box::new(DefaultDebugForeignCallExecutor::from_artifact(
            PrintOutput::Stdout,
            self.debug_artifact,
        ));
        self.context = DebugContext::new(
            self.blackbox_solver,
            self.circuits,
            self.debug_artifact,
            self.initial_witness.clone(),
            foreign_call_executor,
            self.unconstrained_functions,
        );
        for debug_location in breakpoints {
            self.context.add_breakpoint(debug_location);
        }
        self.last_result = DebugCommandResult::Ok;
        println!("Restarted debugging session.");
        self.show_current_vm_status();
    }

    pub fn show_witness_map(&self) {
        let witness_map = self.context.get_witness_map();
        // NOTE: we need to clone() here to get the iterator
        for (witness, value) in witness_map.clone().into_iter() {
            println!("_{} = {value}", witness.witness_index());
        }
    }

    pub fn show_witness(&self, index: u32) {
        if let Some(value) = self.context.get_witness_map().get_index(index) {
            println!("_{} = {value}", index);
        }
    }

    pub fn update_witness(&mut self, index: u32, value: String) {
        let Some(field_value) = FieldElement::try_from_str(&value) else {
            println!("Invalid witness value: {value}");
            return;
        };

        let witness = Witness::from(index);
        _ = self.context.overwrite_witness(witness, field_value);
        println!("_{} = {value}", index);
    }

    pub fn show_brillig_memory(&self) {
        if !self.context.is_executing_brillig() {
            println!("Not executing a Brillig block");
            return;
        }

        let Some(memory) = self.context.get_brillig_memory() else {
            // this can happen when just entering the Brillig block since ACVM
            // would have not initialized the Brillig VM yet; in fact, the
            // Brillig code may be skipped altogether
            println!("Brillig VM memory not available");
            return;
        };

        for (index, value) in memory.iter().enumerate() {
            // Zero field is the default value, we omit it when printing memory
            if let MemoryValue::Field(field) = value {
                if field == &FieldElement::zero() {
                    continue;
                }
            }
            println!("{index} = {}", value);
        }
    }

    pub fn write_brillig_memory(&mut self, index: usize, value: String, bit_size: u32) {
        let Some(field_value) = FieldElement::try_from_str(&value) else {
            println!("Invalid value: {value}");
            return;
        };

        let Ok(bit_size) = BitSize::try_from_u32::<FieldElement>(bit_size) else {
            println!("Invalid bit size: {bit_size}");
            return;
        };

        if !self.context.is_executing_brillig() {
            println!("Not executing a Brillig block");
            return;
        }
        self.context.write_brillig_memory(index, field_value, bit_size);
    }

    pub fn show_vars(&self) {
        for frame in self.context.get_variables() {
            println!("{}({})", frame.function_name, frame.function_params.join(", "));
            for (var_name, value, var_type) in frame.variables.iter() {
                let printable_value =
                    PrintableValueDisplay::Plain((*value).clone(), (*var_type).clone());
                println!("  {var_name}:{var_type:?} = {}", printable_value);
            }
        }
    }

    fn is_solved(&self) -> bool {
        self.context.is_solved()
    }

    fn finalize(self) -> WitnessStack<FieldElement> {
        self.context.finalize()
    }
}

pub fn run<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    program: CompiledProgram,
    initial_witness: WitnessMap<FieldElement>,
) -> Result<Option<WitnessStack<FieldElement>>, NargoError<FieldElement>> {
    let circuits = &program.program.functions;
    let debug_artifact =
        &DebugArtifact { debug_symbols: program.debug, file_map: program.file_map };
    let unconstrained_functions = &program.program.unconstrained_functions;
    let context = RefCell::new(ReplDebugger::new(
        blackbox_solver,
        circuits,
        debug_artifact,
        initial_witness,
        unconstrained_functions,
    ));
    let ref_context = &context;

    ref_context.borrow().show_current_vm_status();

    let mut repl = Repl::builder()
        .add(
            "step",
            command! {
                "step to the next ACIR opcode",
                () => || {
                    ref_context.borrow_mut().step_acir_opcode();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "into",
            command! {
                "step into to the next opcode",
                () => || {
                    ref_context.borrow_mut().step_into_opcode();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "next",
            command! {
                "step until a new source location is reached",
                () => || {
                    ref_context.borrow_mut().next_into();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "over",
            command! {
                "step until a new source location is reached without diving into function calls",
                () => || {
                    ref_context.borrow_mut().next_over();
                    Ok(CommandStatus::Done)
                }
            }
        )
        .add(
            "out",
            command! {
                "step until a new source location is reached and the current stack frame is finished",
                () => || {
                    ref_context.borrow_mut().next_out();
                    Ok(CommandStatus::Done)
                }
            }
        )
        .add(
            "continue",
            command! {
                "continue execution until the end of the program",
                () => || {
                    ref_context.borrow_mut().cont();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "restart",
            command! {
                "restart the debugging session",
                () => || {
                    ref_context.borrow_mut().restart_session();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "opcodes",
            command! {
                "display ACIR opcodes",
                () => || {
                    ref_context.borrow().display_opcodes();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "break",
            command! {
                "add a breakpoint at an opcode location",
                (LOCATION:DebugLocation) => |location| {
                    ref_context.borrow_mut().add_breakpoint_at(location);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "delete",
            command! {
                "delete breakpoint at an opcode location",
                (LOCATION:DebugLocation) => |location| {
                    ref_context.borrow_mut().delete_breakpoint_at(location);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "witness",
            command! {
                "show witness map",
                () => || {
                    ref_context.borrow().show_witness_map();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "witness",
            command! {
                "display a single witness from the witness map",
                (index: u32) => |index| {
                    ref_context.borrow().show_witness(index);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "witness",
            command! {
                "update a witness with the given value",
                (index: u32, value: String) => |index, value| {
                    ref_context.borrow_mut().update_witness(index, value);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "memory",
            command! {
                "show Brillig memory (valid when executing a Brillig block)",
                () => || {
                    ref_context.borrow().show_brillig_memory();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "memset",
            command! {
                "update a Brillig memory cell with the given value",
                (index: usize, value: String, bit_size: u32) => |index, value, bit_size| {
                    ref_context.borrow_mut().write_brillig_memory(index, value, bit_size);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "stacktrace",
            command! {
                "display the current stack trace",
                () => || {
                    ref_context.borrow().show_current_call_stack();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "vars",
            command! {
                "show variables for each function scope available at this point in execution",
                () => || {
                    ref_context.borrow_mut().show_vars();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .build()
        .expect("Failed to initialize debugger repl");

    repl.run().expect("Debugger error");

    // REPL execution has finished.
    // Drop it so that we can move fields out from `context` again.
    drop(repl);

    if context.borrow().is_solved() {
        let solved_witness_stack = context.into_inner().finalize();
        Ok(Some(solved_witness_stack))
    } else {
        Ok(None)
    }
}
