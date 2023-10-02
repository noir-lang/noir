use acvm::pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use acvm::acir::circuit::OpcodeLocation;

use crate::artifacts::debug::DebugArtifact;
use crate::errors::ExecutionError;
use crate::NargoError;

use super::foreign_calls::ForeignCall;

use std::io::{self, Write};

enum SolveResult {
    Done,
    Ok,
}

enum Command {
    Step,
    Continue,
    Stop,
}

pub fn debug_circuit<B: BlackBoxFunctionSolver>(
    blackbox_solver: &B,
    circuit: Circuit,
    debug_artifact: DebugArtifact,
    initial_witness: WitnessMap,
    show_output: bool,
) -> Result<WitnessMap, NargoError> {
    let mut acvm = ACVM::new(blackbox_solver, circuit.opcodes, initial_witness);

    'outer: loop {
        show_current_vm_status(&acvm, &debug_artifact);
        let command = match read_command() {
            Ok(cmd) => cmd,
            Err(err) => {
                eprintln!("Error reading command: {}", err);
                return Err(NargoError::ExecutionError(ExecutionError::Halted))
            }
        };
        match command {
            Command::Stop => return Err(NargoError::ExecutionError(ExecutionError::Halted)),
            Command::Step => {
                match step_opcode(&mut acvm, &circuit.assert_messages, show_output)? {
                    SolveResult::Done => break,
                    SolveResult::Ok => {},
                }
            }
            Command::Continue => {
                println!("(Continuing execution...)");
                loop {
                    match step_opcode(&mut acvm, &circuit.assert_messages, show_output)? {
                        SolveResult::Done => break 'outer,
                        SolveResult::Ok => {},
                    }
                }
            },
        }
    }

    let solved_witness = acvm.finalize();
    Ok(solved_witness)
}

fn step_opcode<B: BlackBoxFunctionSolver>(
    acvm: &mut ACVM<B>,
    assert_messages: &Vec<(OpcodeLocation, String)>,
    show_output: bool,
) -> Result<SolveResult, NargoError> {
    // Assert messages are not a map due to https://github.com/noir-lang/acvm/issues/522
    let get_assert_message = |opcode_location| {
        assert_messages
            .iter()
            .find(|(loc, _)| loc == opcode_location)
            .map(|(_, message)| message.clone())
    };

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
            let foreign_call_result = ForeignCall::execute(&foreign_call, show_output)?;
            acvm.resolve_pending_foreign_call(foreign_call_result);
            Ok(SolveResult::Ok)
        }
    }
}

fn show_source_code_location(location: &OpcodeLocation, debug_artifact: &DebugArtifact) {
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

fn show_current_vm_status<B: BlackBoxFunctionSolver> (acvm: &ACVM<B>, debug_artifact: &DebugArtifact) {
    let ip = acvm.instruction_pointer();
    println!("Stopped at opcode {}: {}", ip, acvm.opcodes()[ip]);
    show_source_code_location(&OpcodeLocation::Acir(ip), &debug_artifact);
}

fn read_command() -> Result<Command, io::Error> {
    loop {
        let mut line = String::new();
        print!(">>> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line)?;
        if line.is_empty() {
            return Ok(Command::Stop);
        }
        match line.trim() {
            "s" => return Ok(Command::Step),
            "c" => return Ok(Command::Continue),
            "q" => return Ok(Command::Stop),
            "" => continue,
            _ => println!("ERROR: unknown command")
        }
    }
}
