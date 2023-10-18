use acvm::acir::circuit::OpcodeLocation;
use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use nargo::artifacts::debug::DebugArtifact;
use nargo::errors::ExecutionError;
use nargo::NargoError;

use nargo::ops::ForeignCallExecutor;

use easy_repl::{command, CommandStatus, Critical, Repl};
use std::cell::{Cell, RefCell};

use owo_colors::OwoColorize;

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
            Self::show_source_code_location(&OpcodeLocation::Acir(ip), &self.debug_artifact);
        }
    }

    fn show_source_code_location(location: &OpcodeLocation, debug_artifact: &DebugArtifact) {
        let locations = debug_artifact.debug_symbols[0].opcode_location(location);
        if let Some(locations) = locations {
            for loc in locations {
                let file = &debug_artifact.file_map[&loc.file];
                let source = &file.source.as_str();
                let start = loc.span.start() as usize;
                let end = loc.span.end() as usize;
                println!("At {}:{start}-{end}", file.path.as_path().display());
                println!(
                    "\n{}{}{}\n",
                    &source[0..start].to_string().dimmed(),
                    &source[start..end],
                    &source[end..].to_string().dimmed(),
                );
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
