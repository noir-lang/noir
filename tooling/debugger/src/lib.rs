use acvm::acir::circuit::OpcodeLocation;
use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use nargo::artifacts::debug::DebugArtifact;
use nargo::errors::{ExecutionError, Location};
use nargo::NargoError;

use nargo::ops::ForeignCallExecutor;

use easy_repl::{command, CommandStatus, Critical, Repl};
use std::{
    cell::{Cell, RefCell},
    ops::Range,
};

use owo_colors::OwoColorize;

use codespan_reporting::files::Files;

enum SolveResult {
    Done,
    Ok,
}

struct DebugContext<'backend, B: BlackBoxFunctionSolver> {
    acvm: ACVM<'backend, B>,
    debug_artifact: DebugArtifact,
    foreign_call_executor: ForeignCallExecutor,
    circuit: &'backend Circuit,
    show_output: bool,
}

impl<'backend, B: BlackBoxFunctionSolver> DebugContext<'backend, B> {
    fn step_opcode(&mut self) -> Result<SolveResult, NargoError> {
        let solver_status = self.acvm.solve_opcode();

        match solver_status {
            ACVMStatus::Solved => Ok(SolveResult::Done),
            ACVMStatus::InProgress => Ok(SolveResult::Ok),
            ACVMStatus::Failure(error) => {
                let call_stack = match &error {
                    OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_location: ErrorLocation::Resolved(opcode_location),
                    } => Some(vec![*opcode_location]),
                    OpcodeResolutionError::BrilligFunctionFailed { call_stack, .. } => {
                        Some(call_stack.clone())
                    }
                    _ => None,
                };

                Err(NargoError::ExecutionError(match call_stack {
                    Some(call_stack) => {
                        if let Some(assert_message) = self.circuit.get_assert_message(
                            *call_stack.last().expect("Call stacks should not be empty"),
                        ) {
                            ExecutionError::AssertionFailed(assert_message.to_owned(), call_stack)
                        } else {
                            ExecutionError::SolvingError(error)
                        }
                    }
                    None => ExecutionError::SolvingError(error),
                }))
            }
            ACVMStatus::RequiresForeignCall(foreign_call) => {
                let foreign_call_result =
                    self.foreign_call_executor.execute(&foreign_call, self.show_output)?;
                self.acvm.resolve_pending_foreign_call(foreign_call_result);
                Ok(SolveResult::Ok)
            }
        }
    }

    fn show_current_vm_status(&self) {
        let ip = self.acvm.instruction_pointer();
        let opcodes = self.acvm.opcodes();
        if ip >= opcodes.len() {
            println!("Finished execution");
        } else {
            println!("Stopped at opcode {}: {}", ip, opcodes[ip]);
            self.show_source_code_location(&OpcodeLocation::Acir(ip), &self.debug_artifact);
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

    fn show_source_code_location(&self, location: &OpcodeLocation, debug_artifact: &DebugArtifact) {
        let locations = debug_artifact.debug_symbols[0].opcode_location(location);
        let Some(locations) = locations else { return };
        for loc in locations {
            self.print_location_path(loc);

            let loc_line_index = debug_artifact.location_line_index(loc).unwrap();

            // How many lines before or after the location's line we
            // print
            let context_lines = 5;

            let first_line_to_print =
                if loc_line_index < context_lines { 0 } else { loc_line_index - context_lines };

            let last_line_index = debug_artifact.last_line_index(loc).unwrap();
            let last_line_to_print = std::cmp::min(loc_line_index + context_lines, last_line_index);

            let source = debug_artifact.location_source_code(loc).unwrap();
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
                        debug_artifact.location_in_line(loc).unwrap();
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

    fn cont(&mut self) -> Result<SolveResult, NargoError> {
        loop {
            match self.step_opcode()? {
                SolveResult::Done => break,
                SolveResult::Ok => {}
            }
        }
        Ok(SolveResult::Done)
    }

    fn finalize(self) -> WitnessMap {
        self.acvm.finalize()
    }
}

fn print_line_of_ellipsis(line_number: usize) {
    println!("{}", format!("{:>3} {}", line_number, "...").dimmed());
}

fn print_dimmed_line(line_number: usize, line: &str) {
    println!("{}", format!("{:>3} {:2} {}", line_number, "", line).dimmed());
}

fn map_command_status(result: SolveResult) -> CommandStatus {
    match result {
        SolveResult::Ok => CommandStatus::Done,
        SolveResult::Done => CommandStatus::Quit,
    }
}

pub fn debug_circuit<B: BlackBoxFunctionSolver>(
    blackbox_solver: &B,
    circuit: &Circuit,
    debug_artifact: DebugArtifact,
    initial_witness: WitnessMap,
    show_output: bool,
) -> Result<Option<WitnessMap>, NargoError> {
    let context = RefCell::new(DebugContext {
        acvm: ACVM::new(blackbox_solver, &circuit.opcodes, initial_witness),
        foreign_call_executor: ForeignCallExecutor::default(),
        circuit,
        debug_artifact,
        show_output,
    });
    let ref_step = &context;
    let ref_cont = &context;

    let solved = Cell::new(false);

    context.borrow().show_current_vm_status();

    let handle_result = |result| {
        solved.set(matches!(result, SolveResult::Done));
        Ok(map_command_status(result))
    };

    let mut repl = Repl::builder()
        .add(
            "s",
            command! {
                "step to the next opcode",
                () => || {
                    let result = ref_step.borrow_mut().step_opcode().into_critical()?;
                    ref_step.borrow().show_current_vm_status();
                    handle_result(result)
                }
            },
        )
        .add(
            "c",
            command! {
                "continue execution until the end of the program",
                () => || {
                    println!("(Continuing execution...)");
                    let result = ref_cont.borrow_mut().cont().into_critical()?;
                    handle_result(result)
                }
            },
        )
        .build()
        .expect("Failed to initialize debugger repl");

    repl.run().expect("Debugger error");

    // REPL execution has finished.
    // Drop it so that we can move fields out from `context` again.
    drop(repl);

    if solved.get() {
        let solved_witness = context.into_inner().finalize();
        Ok(Some(solved_witness))
    } else {
        Ok(None)
    }
}
