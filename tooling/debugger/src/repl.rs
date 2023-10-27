use crate::context::{DebugCommandResult, DebugContext};

use acvm::acir::circuit::{Opcode, OpcodeLocation};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use nargo::artifacts::debug::DebugArtifact;
use nargo::NargoError;

use easy_repl::{command, CommandStatus, Repl};
use std::cell::RefCell;

use codespan_reporting::files::Files;
use noirc_errors::Location;

use owo_colors::OwoColorize;

use std::ops::Range;

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
        let context =
            DebugContext::new(blackbox_solver, circuit, debug_artifact, initial_witness.clone());
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
                        println!("At opcode {}: {}", ip, opcodes[ip])
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
                self.show_source_code_location(&location);
            }
        }
    }

    fn print_location_path(&self, loc: Location) {
        let line_number = self.debug_artifact.location_line_number(loc).unwrap();
        let column_number = self.debug_artifact.location_column_number(loc).unwrap();

        println!(
            "At {}:{line_number}:{column_number}",
            self.debug_artifact.name(loc.file).unwrap()
        );
    }

    fn show_source_code_location(&self, location: &OpcodeLocation) {
        let locations = self.debug_artifact.debug_symbols[0].opcode_location(location);
        let Some(locations) = locations else { return };
        for loc in locations {
            self.print_location_path(loc);

            let loc_line_index = self.debug_artifact.location_line_index(loc).unwrap();

            // How many lines before or after the location's line we
            // print
            let context_lines = 5;

            let first_line_to_print =
                if loc_line_index < context_lines { 0 } else { loc_line_index - context_lines };

            let last_line_index = self.debug_artifact.last_line_index(loc).unwrap();
            let last_line_to_print = std::cmp::min(loc_line_index + context_lines, last_line_index);

            let source = self.debug_artifact.location_source_code(loc).unwrap();
            for (current_line_index, line) in source.lines().enumerate() {
                let current_line_number = current_line_index + 1;

                if current_line_index < first_line_to_print {
                    // Ignore lines before range starts
                    continue;
                } else if current_line_index == first_line_to_print && current_line_index > 0 {
                    // Denote that there's more lines before but we're not showing them
                    print_line_of_ellipsis(current_line_index);
                }

                if current_line_index > last_line_to_print {
                    // Denote that there's more lines after but we're not showing them,
                    // and stop printing
                    print_line_of_ellipsis(current_line_number);
                    break;
                }

                if current_line_index == loc_line_index {
                    // Highlight current location
                    let Range { start: loc_start, end: loc_end } =
                        self.debug_artifact.location_in_line(loc).unwrap();
                    println!(
                        "{:>3} {:2} {}{}{}",
                        current_line_number,
                        "->",
                        &line[0..loc_start].to_string().dimmed(),
                        &line[loc_start..loc_end],
                        &line[loc_end..].to_string().dimmed()
                    );
                } else {
                    print_dimmed_line(current_line_number, line);
                }
            }
        }
    }

    fn display_opcodes(&self) {
        let opcodes = self.context.get_opcodes();
        let current_opcode_location = self.context.get_current_opcode_location();
        let current_ip = match current_opcode_location {
            Some(OpcodeLocation::Acir(ip)) => Some(ip),
            Some(OpcodeLocation::Brillig { acir_index, .. }) => Some(acir_index),
            None => None,
        };
        let current_brillig_pc = match current_opcode_location {
            Some(OpcodeLocation::Brillig { brillig_index, .. }) => brillig_index,
            _ => 0,
        };
        for (ip, opcode) in opcodes.iter().enumerate() {
            let marker = if current_ip == Some(ip) {
                "->"
            } else if self.context.is_breakpoint_set(&OpcodeLocation::Acir(ip)) {
                " *"
            } else {
                ""
            };
            if let Opcode::Brillig(brillig) = opcode {
                println!("{:>3} {:2} BRILLIG inputs={:?}", ip, marker, brillig.inputs);
                println!("       |       outputs={:?}", brillig.outputs);
                for (pc, brillig_opcode) in brillig.bytecode.iter().enumerate() {
                    println!(
                        "{:>3}.{:<2} |{:2} {:?}",
                        ip,
                        pc,
                        if pc == current_brillig_pc {
                            marker
                        } else if self.context.is_breakpoint_set(&OpcodeLocation::Brillig {
                            acir_index: ip,
                            brillig_index: pc,
                        }) {
                            " *"
                        } else {
                            ""
                        },
                        brillig_opcode
                    );
                }
            } else {
                println!("{:>3} {:2} {:?}", ip, marker, opcode);
            }
        }
    }

    fn add_breakpoint_at(&mut self, location: OpcodeLocation) {
        if self.context.is_breakpoint_set(&location) {
            println!("Breakpoint at opcode {location} already set");
        } else if !self.context.is_valid_location(&location) {
            println!("Invalid opcode location {location}");
        } else {
            self.context.add_breakpoint(location);
            println!("Added breakpoint at opcode {location}");
        }
    }

    fn delete_breakpoint_at(&mut self, location: OpcodeLocation) {
        if self.context.is_breakpoint_set(&location) {
            self.context.delete_breakpoint(&location);
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
        self.context = DebugContext::new(
            self.blackbox_solver,
            self.circuit,
            self.debug_artifact,
            self.initial_witness.clone(),
        );
        self.last_result = DebugCommandResult::Ok;
        println!("Restarted debugging session.");
        self.show_current_vm_status();
    }

    fn is_solved(&self) -> bool {
        self.context.is_solved()
    }

    fn finalize(self) -> WitnessMap {
        self.context.finalize()
    }
}

fn print_line_of_ellipsis(line_number: usize) {
    println!("{}", format!("{:>3} {}", line_number, "...").dimmed());
}

fn print_dimmed_line(line_number: usize, line: &str) {
    println!("{}", format!("{:>3} {:2} {}", line_number, "", line).dimmed());
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
