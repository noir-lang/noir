use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use acvm::acir::circuit::OpcodeLocation;

use crate::artifacts::debug::DebugArtifact;
use crate::errors::ExecutionError;
use crate::NargoError;

use super::foreign_calls::ForeignCallExecutor;

use std::rc::Rc;
use std::sync::Mutex;

use reedline_repl_rs::clap::{
    ArgMatches as ReplArgMatches,
    Command as ReplCommand,
};
use reedline_repl_rs::Repl;

enum SolveResult {
    Done,
    Ok,
}

struct ReplContext<'backend, B: BlackBoxFunctionSolver> {
    acvm: Option<ACVM<'backend, B>>,
    debug_artifact: DebugArtifact,
    foreign_call_executor: ForeignCallExecutor,
    circuit: Circuit,
    show_output: bool,
}

impl<'backend, B> ReplContext<'backend, B> where B: BlackBoxFunctionSolver {
    fn step_opcode(&mut self) -> Result<SolveResult, NargoError> {
        // Assert messages are not a map due to https://github.com/noir-lang/acvm/issues/522
        let assert_messages = &self.circuit.assert_messages;
        let get_assert_message = |opcode_location| {
            assert_messages
                .iter()
                .find(|(loc, _)| loc == opcode_location)
                .map(|(_, message)| message.clone())
        };

        let acvm = self.acvm.as_mut().unwrap();
        let solver_status = acvm.solve_opcode();

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
                        if let Some(assert_message) = get_assert_message(
                            call_stack.last().expect("Call stacks should not be empty"),
                        ) {
                            ExecutionError::AssertionFailed(assert_message, call_stack)
                        } else {
                            ExecutionError::SolvingError(error)
                        }
                    }
                    None => ExecutionError::SolvingError(error),
                }))
            }
            ACVMStatus::RequiresForeignCall(foreign_call) => {
                let foreign_call_result = self.foreign_call_executor.execute(&foreign_call, self.show_output)?;
                acvm.resolve_pending_foreign_call(foreign_call_result);
                Ok(SolveResult::Ok)
            }
        }
    }

    fn show_current_vm_status(&self) {
        let acvm = self.acvm.as_ref().unwrap();
        let ip = acvm.instruction_pointer();
        println!("Stopped at opcode {}: {}", ip, acvm.opcodes()[ip]);
        self.show_source_code_location(&OpcodeLocation::Acir(ip));
    }

    fn show_source_code_location(&self, location: &OpcodeLocation) {
        let locations = self.debug_artifact.debug_symbols[0].opcode_location(&location);
        match locations {
            Some(locations) => {
                for loc in locations {
                    let file = &self.debug_artifact.file_map[&loc.file];
                    let source = &file.source.as_str();
                    let start = loc.span.start() as usize;
                    let end = loc.span.end() as usize;
                    println!("At {}:{start}-{end}", file.path.as_path().display());
                    println!("\n{}\n", &source[start..end]);
                }
            },
            None => {}
        }
    }

    fn finalize(&mut self) -> WitnessMap {
        self.acvm.take().unwrap().finalize()
    }
}

impl From<reedline_repl_rs::Error> for NargoError {
    fn from(_e: reedline_repl_rs::Error) -> Self {
        NargoError::CompilationError
    }
}


// impl fmt::Display for NargoError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             NargoError::CompilationError => write!(f, "Compilation Error"),
//             NargoError::ExecutionError(e) => write!(f, "Execution Error: {}", e),
//             NargoError::ForeignCallError(e) => write!(f, "Foreign call Error: {}", e),
//         }
//     }
// }

pub fn debug_circuit<B: BlackBoxFunctionSolver>(
    blackbox_solver: &B,
    circuit: Circuit,
    debug_artifact: DebugArtifact,
    initial_witness: WitnessMap,
    show_output: bool,
) -> Result<WitnessMap, NargoError> {
    let opcodes = circuit.opcodes.clone();
    let acvm = ACVM::new(blackbox_solver, opcodes, initial_witness);
    let foreign_call_executor = ForeignCallExecutor::default();

    let repl_context = Rc::new(Mutex::new(ReplContext {
        acvm: Some(acvm),
        debug_artifact,
        foreign_call_executor,
        circuit,
        show_output,
    }));
    let mut repl = Repl::new(repl_context.clone())
        .with_name("debug")
        .with_version(env!["CARGO_PKG_VERSION"])
        .with_command(
            ReplCommand::new("s")
                .about("step to the next opcode"),
            step_command,
        )
        .with_command(
            ReplCommand::new("c")
                .about("continue execution until the end of the program"),
            continue_command,
        )
        .with_command(
            ReplCommand::new("q")
                .about("quit the debugger"),
            quit_command,
        );
    repl.run().unwrap();
    let solved_witness = repl_context.lock().unwrap().finalize();
    Ok(solved_witness)
}

fn step_command<B: BlackBoxFunctionSolver>(_args: ReplArgMatches, context: &mut Rc<Mutex<ReplContext<B>>>) -> Result<Option<String>, NargoError> {
    let mut c = context.lock().unwrap();
    c.show_current_vm_status();
    match c.step_opcode()? {
        SolveResult::Done => Ok(Some("Done".to_string())),
        SolveResult::Ok => Ok(Some("Ok".to_string())),
    }
}

fn continue_command<B: BlackBoxFunctionSolver>(_args: ReplArgMatches, context: &mut Rc<Mutex<ReplContext<B>>>) -> Result<Option<String>, NargoError> {
    let mut c = context.lock().unwrap();
    c.show_current_vm_status();
    println!("(Continuing execution...)");
    loop {
        match c.step_opcode()? {
            SolveResult::Done => break,
            SolveResult::Ok => {},
        }
    }
    Ok(Some("Ok".to_string()))
}

fn quit_command<B: BlackBoxFunctionSolver>(_args: ReplArgMatches, context: &mut Rc<Mutex<ReplContext<B>>>) -> Result<Option<String>, NargoError> {
    context.lock().unwrap().show_current_vm_status();
    Err(NargoError::ExecutionError(ExecutionError::Halted))
}
