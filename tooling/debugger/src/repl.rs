use crate::context::{DebugCommandResult, DebugContext};

use acvm::acir::circuit::{Circuit, Opcode, OpcodeLocation};
use acvm::acir::native_types::{Witness, WitnessMap};
use acvm::{BlackBoxFunctionSolver, FieldElement};

use nargo::{artifacts::debug::DebugArtifact, ops::DefaultForeignCallExecutor, NargoError};

use easy_repl::{command, CommandStatus, Repl};
use std::cell::RefCell;

use crate::source_code_printer::print_source_code_location;

pub struct ReplDebugger<'a, B: BlackBoxFunctionSolver> {
    context: DebugContext<'a, B>,
    blackbox_solver: &'a B,
    circuit: &'a Circuit,
    debug_artifact: &'a DebugArtifact,
    initial_witness: WitnessMap,
    last_result: DebugCommandResult,
}

impl<'a, B: BlackBoxFunctionSolver> ReplDebugger<'a, B> {
    pub fn new(
        blackbox_solver: &'a B,
        circuit: &'a Circuit,
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap,
    ) -> Self {
        let context = DebugContext::new(
            blackbox_solver,
            circuit,
            debug_artifact,
            initial_witness.clone(),
            Box::new(DefaultForeignCallExecutor::new(true, None)),
        );
        Self {
            context,
            blackbox_solver,
            circuit,
            debug_artifact,
            initial_witness,
            last_result: DebugCommandResult::Ok,
        }
    }

    pub fn show_current_vm_status(&self) {
        let location = self.context.get_current_opcode_location();
        let opcodes = self.context.get_opcodes();

        match location {
            None => println!("Finished execution"),
            Some(location) => {
                match location {
                    OpcodeLocation::Acir(ip) => {
                        // Default Brillig display is too bloated for this context,
                        // so we limit it to denoting it's the start of a Brillig
                        // block. The user can still use the `opcodes` command to
                        // take a look at the whole block.
                        let opcode_summary = match opcodes[ip] {
                            Opcode::Brillig(..) => "BRILLIG: ...".into(),
                            _ => format!("{}", opcodes[ip]),
                        };
                        println!("At opcode {}: {}", ip, opcode_summary);
                    }
                    OpcodeLocation::Brillig { acir_index, brillig_index } => {
                        let Opcode::Brillig(ref brillig) = opcodes[acir_index] else {
                            unreachable!("Brillig location does not contain a Brillig block");
                        };
                        println!(
                            "At opcode {}.{}: {:?}",
                            acir_index, brillig_index, brillig.bytecode[brillig_index]
                        );
                    }
                }

                print_source_code_location(self.debug_artifact, &location);
            }
        }
    }

    fn display_opcodes(&self) {
        let opcodes = self.context.get_opcodes();
        let current_opcode_location = self.context.get_current_opcode_location();
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
            } else if self.context.is_breakpoint_set(&OpcodeLocation::Acir(acir_index)) {
                " *"
            } else {
                ""
            }
        };
        let brillig_marker = |acir_index, brillig_index| {
            if current_acir_index == Some(acir_index) && brillig_index == current_brillig_index {
                "->"
            } else if self
                .context
                .is_breakpoint_set(&OpcodeLocation::Brillig { acir_index, brillig_index })
            {
                " *"
            } else {
                ""
            }
        };
        for (acir_index, opcode) in opcodes.iter().enumerate() {
            let marker = outer_marker(acir_index);
            if let Opcode::Brillig(brillig) = opcode {
                println!("{:>3} {:2} BRILLIG inputs={:?}", acir_index, marker, brillig.inputs);
                println!("       |       outputs={:?}", brillig.outputs);
                for (brillig_index, brillig_opcode) in brillig.bytecode.iter().enumerate() {
                    println!(
                        "{:>3}.{:<2} |{:2} {:?}",
                        acir_index,
                        brillig_index,
                        brillig_marker(acir_index, brillig_index),
                        brillig_opcode
                    );
                }
            } else {
                println!("{:>3} {:2} {:?}", acir_index, marker, opcode);
            }
        }
    }

    fn add_breakpoint_at(&mut self, location: OpcodeLocation) {
        if !self.context.is_valid_opcode_location(&location) {
            println!("Invalid opcode location {location}");
        } else if self.context.add_breakpoint(location) {
            println!("Added breakpoint at opcode {location}");
        } else {
            println!("Breakpoint at opcode {location} already set");
        }
    }

    fn delete_breakpoint_at(&mut self, location: OpcodeLocation) {
        if self.context.delete_breakpoint(&location) {
            println!("Breakpoint at opcode {location} deleted");
        } else {
            println!("Breakpoint at opcode {location} not set");
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

    fn next(&mut self) {
        if self.validate_in_progress() {
            let result = self.context.next();
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
        let breakpoints: Vec<OpcodeLocation> =
            self.context.iterate_breakpoints().copied().collect();
        self.context = DebugContext::new(
            self.blackbox_solver,
            self.circuit,
            self.debug_artifact,
            self.initial_witness.clone(),
            Box::new(DefaultForeignCallExecutor::new(true, None)),
        );
        for opcode_location in breakpoints {
            self.context.add_breakpoint(opcode_location);
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

    pub fn show_brillig_registers(&self) {
        if !self.context.is_executing_brillig() {
            println!("Not executing a Brillig block");
            return;
        }

        let Some(registers) = self.context.get_brillig_registers() else {
            // this can happen when just entering the Brillig block since ACVM
            // would have not initialized the Brillig VM yet; in fact, the
            // Brillig code may be skipped altogether
            println!("Brillig VM registers not available");
            return;
        };

        for (index, value) in registers.inner.iter().enumerate() {
            println!("{index} = {}", value.to_field());
        }
    }

    pub fn set_brillig_register(&mut self, index: usize, value: String) {
        let Some(field_value) = FieldElement::try_from_str(&value) else {
            println!("Invalid value: {value}");
            return;
        };
        if !self.context.is_executing_brillig() {
            println!("Not executing a Brillig block");
            return;
        }
        self.context.set_brillig_register(index, field_value);
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
            println!("{index} = {}", value.to_field());
        }
    }

    pub fn write_brillig_memory(&mut self, index: usize, value: String) {
        let Some(field_value) = FieldElement::try_from_str(&value) else {
            println!("Invalid value: {value}");
            return;
        };
        if !self.context.is_executing_brillig() {
            println!("Not executing a Brillig block");
            return;
        }
        self.context.write_brillig_memory(index, field_value);
    }

    fn is_solved(&self) -> bool {
        self.context.is_solved()
    }

    fn finalize(self) -> WitnessMap {
        self.context.finalize()
    }
}

pub fn run<B: BlackBoxFunctionSolver>(
    blackbox_solver: &B,
    circuit: &Circuit,
    debug_artifact: &DebugArtifact,
    initial_witness: WitnessMap,
) -> Result<Option<WitnessMap>, NargoError> {
    let context =
        RefCell::new(ReplDebugger::new(blackbox_solver, circuit, debug_artifact, initial_witness));
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
                    ref_context.borrow_mut().next();
                    Ok(CommandStatus::Done)
                }
            },
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
                (LOCATION:OpcodeLocation) => |location| {
                    ref_context.borrow_mut().add_breakpoint_at(location);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "delete",
            command! {
                "delete breakpoint at an opcode location",
                (LOCATION:OpcodeLocation) => |location| {
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
            "registers",
            command! {
                "show Brillig registers (valid when executing a Brillig block)",
                () => || {
                    ref_context.borrow().show_brillig_registers();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "regset",
            command! {
                "update a Brillig register with the given value",
                (index: usize, value: String) => |index, value| {
                    ref_context.borrow_mut().set_brillig_register(index, value);
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
                (index: usize, value: String) => |index, value| {
                    ref_context.borrow_mut().write_brillig_memory(index, value);
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
        let solved_witness = context.into_inner().finalize();
        Ok(Some(solved_witness))
    } else {
        Ok(None)
    }
}
