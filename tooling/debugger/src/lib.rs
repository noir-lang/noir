use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use acvm::acir::circuit::OpcodeLocation;

use nargo::artifacts::debug::DebugArtifact;
use nargo::errors::ExecutionError;
use nargo::NargoError;

use nargo::ops::ForeignCallExecutor;

use thiserror::Error;

use easy_repl::{Repl, CommandStatus, command};

// use reedline_repl_rs::clap::{
//     ArgMatches as ReplArgMatches,
//     Command as ReplCommand,
// };
// use reedline_repl_rs::Repl;

enum SolveResult {
    Done,
    Ok,
}

#[derive(Debug, Error)]
enum DebuggingError {
    /// ACIR circuit execution error
    #[error(transparent)]
    ExecutionError(#[from] nargo::errors::ExecutionError),

    /// Oracle handling error
    #[error(transparent)]
    ForeignCallError(#[from] noirc_printable_type::ForeignCallError),
}

struct ReplContext<'backend, B: BlackBoxFunctionSolver> {
    acvm: Option<ACVM<'backend, B>>,
    debug_artifact: DebugArtifact,
    foreign_call_executor: ForeignCallExecutor,
    circuit: Circuit,
    show_output: bool,
}

fn step_opcode<'backend, B: BlackBoxFunctionSolver>(acvm: &mut ACVM<'backend, B>, circuit: Circuit, foreign_call_executor: &mut ForeignCallExecutor, show_output: bool) -> Result<SolveResult, DebuggingError> {
    // Assert messages are not a map due to https://github.com/noir-lang/acvm/issues/522
    let assert_messages = circuit.assert_messages;
    let get_assert_message = |opcode_location| {
        assert_messages
            .iter()
            .find(|(loc, _)| loc == opcode_location)
            .map(|(_, message)| message.clone())
    };

    // let acvm = self.acvm.as_mut().unwrap();
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

            Err(DebuggingError::ExecutionError(match call_stack {
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
            let foreign_call_result = foreign_call_executor.execute(&foreign_call, show_output)?;
            acvm.resolve_pending_foreign_call(foreign_call_result);
            Ok(SolveResult::Ok)
        }
    }
}

fn show_current_vm_status<'backend, B: BlackBoxFunctionSolver>(acvm: &mut ACVM<'backend, B>, debug_artifact: DebugArtifact) {
    let ip = acvm.instruction_pointer();
    println!("Stopped at opcode {}: {}", ip, acvm.opcodes()[ip]);
    show_source_code_location(&OpcodeLocation::Acir(ip), debug_artifact);
}

fn show_source_code_location(location: &OpcodeLocation, debug_artifact: DebugArtifact) {
    let locations = debug_artifact.debug_symbols[0].opcode_location(&location);
    match locations {
        Some(locations) => {
            for loc in locations {
                let file = &debug_artifact.file_map[&loc.file];
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


impl From<reedline_repl_rs::Error> for DebuggingError {
    fn from(_e: reedline_repl_rs::Error) -> Self {
        DebuggingError::ExecutionError(ExecutionError::Halted)
    }
}

impl From<nargo::errors::NargoError> for DebuggingError {
    fn from(e: nargo::errors::NargoError) -> Self {
        match e {
            NargoError::ForeignCallError(e1) => DebuggingError::ForeignCallError(e1),
            _ => DebuggingError::ExecutionError(ExecutionError::Halted),
        }
    }
}

pub fn debug_circuit<B: BlackBoxFunctionSolver>(
    blackbox_solver: &B,
    circuit: Circuit,
    debug_artifact: DebugArtifact,
    initial_witness: WitnessMap,
    show_output: bool,
) -> Result<WitnessMap, NargoError> {
    let opcodes = circuit.opcodes.clone();
    let mut acvm = ACVM::new(blackbox_solver, opcodes, initial_witness);
    let mut foreign_call_executor = ForeignCallExecutor::default();

    let mut repl = Repl::builder()
        .add("s", command! {
            "step to the next opcode",
            () => || {
                step_command(&mut acvm, debug_artifact, circuit, &mut foreign_call_executor, show_output);
                Ok(CommandStatus::Done)
            } 
        })
        .add("c", command! {
            "continue execution until the end of the program",
            () => || {
                continue_command(&mut acvm, debug_artifact, circuit, &mut foreign_call_executor, show_output);
                Ok(CommandStatus::Quit)
            }
        })
        .add("q", command! {
            "quit the debugger",
            () => || {
                quit_command(&mut acvm, debug_artifact);
                Ok(CommandStatus::Quit)
            }
        }).build().expect("Failed to initialize debugger repl");
        repl.run().expect("Critical debugger error");

    // // let mut repl = Repl::new(repl_context.clone())
    // //     .with_name("debug")
    // //     .with_version(env!["CARGO_PKG_VERSION"])
    // //     .with_command(
    // //         ReplCommand::new("s")
    // //             .about("step to the next opcode"),
    // //         step_command,
    // //     )
    // //     .with_command(
    // //         ReplCommand::new("c")
    // //             .about("continue execution until the end of the program"),
    // //         continue_command,
    // //     )
    // //     .with_command(
    // //         ReplCommand::new("q")
    // //             .about("quit the debugger"),
    // //         quit_command,
    // //     );
    // // repl.run().unwrap();
    
    let solved_witness = acvm.finalize();
    Ok(solved_witness)
}

fn step_command<'backend, B: BlackBoxFunctionSolver>(acvm: &mut ACVM<'backend, B>, debug_artifact: DebugArtifact, circuit: Circuit, foreign_call_executor: &mut ForeignCallExecutor, show_output: bool) -> Result<Option<String>, DebuggingError> {
    show_current_vm_status(acvm, debug_artifact);
    match step_opcode(acvm, circuit, foreign_call_executor, show_output)? {
        SolveResult::Done => Ok(Some("Done".to_string())),
        SolveResult::Ok => Ok(Some("Ok".to_string())),
    }
}

fn continue_command<'backend, B: BlackBoxFunctionSolver>(acvm: &mut ACVM<'backend, B>, debug_artifact: DebugArtifact, circuit: Circuit, foreign_call_executor: &mut ForeignCallExecutor, show_output: bool) -> Result<Option<String>, DebuggingError> {
    show_current_vm_status(acvm, debug_artifact);
    println!("(Continuing execution...)");
    loop {
        match step_opcode(acvm, circuit, foreign_call_executor, show_output)? {
            SolveResult::Done => break,
            SolveResult::Ok => {},
        }
    }
    Ok(Some("Ok".to_string()))
}

fn quit_command<'backend, B: BlackBoxFunctionSolver>(acvm: &mut ACVM<'backend, B>, debug_artifact: DebugArtifact) -> Result<Option<String>, DebuggingError> {
    show_current_vm_status(acvm, debug_artifact);
    Err(DebuggingError::ExecutionError(ExecutionError::Halted))
}
