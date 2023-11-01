//! Main module, provides access to debugger functionality through the dap server implementation.
use dap::base_message::Sendable;
use dap::events::*;
use dap::requests::*;
use dap::responses::*;
use dap::types::{
    Capabilities, DisassembledInstruction, Scope, ScopePresentationhint, SourceBreakpoint,
    StoppedEventReason, Thread, Variable,
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_evaluator::brillig::brillig_ir::artifact::GeneratedBrillig;
use serde_json::Value;

use crate::{compile, dap_server::Server, error::DebuggingError, vm, vm::VMType};

use acvm::brillig_vm::VMStatus;
#[allow(deprecated)]
use barretenberg_blackbox_solver::BarretenbergSolver;

/// Breakpoint for program. Sets opcode position to stop.
#[derive(Clone, Debug)]
struct Breakpoint {
    instruction: usize,
}

impl Breakpoint {
    /// Create breakpoint from request.
    pub(crate) fn new(breakpoint: &SourceBreakpoint) -> Option<Self> {
        Some(Breakpoint { instruction: breakpoint.line as usize })
    }
}

/// App state.
#[derive(Debug)]
pub enum State {
    /// The startup state. The state handle only launch command.
    Uninitialized(UninitializedState),
    /// Main loop state. Handles debugging commands.
    Running(RunningState),
    /// Indicates end of work.
    Exit,
}

/// Indicates uninitialized state of app. Handles Initialize and Launch commands.
#[derive(Clone, Debug, Default)]
pub struct UninitializedState;

impl UninitializedState {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn run<T: Server>(
        &mut self,
        server: &mut T,
    ) -> Result<Option<State>, DebuggingError> {
        let request = match server.read() {
            Some(req) => req,
            None => return Ok(None),
        };

        match request.command {
            Command::Initialize(_) => {
                let rsp = request.success(ResponseBody::Initialize(Capabilities {
                    supports_step_back: Some(false),
                    supports_restart_request: Some(false),
                    ..Default::default()
                }));

                server.write(Sendable::Response(rsp));
            }

            Command::Launch(ref arguments) => {
                let mut path = None;
                let mut vm_type = None;
                let additional_data = arguments.additional_data.clone();
                let resp = if let Some(Value::Object(data)) = &additional_data {
                    path = data
                        .get("src_path")
                        .ok_or(DebuggingError::CustomError("Missing source file".to_owned()))
                        .map(|v| {
                            v.as_str().ok_or(DebuggingError::CustomError(
                                "Source file is not a string".to_owned(),
                            ))
                        })?
                        .ok();
                    vm_type = data
                        .get("vm")
                        .ok_or(DebuggingError::CustomError("Missing source file".to_owned()))
                        .map(|v| {
                            v.as_str().ok_or(DebuggingError::CustomError(
                                "Source file is not a string".to_owned(),
                            ))
                        })?
                        .ok();
                    server.write(Sendable::Event(Event::Initialized));
                    request.ack()?
                } else {
                    request.error("Source file is not provided")
                };

                server.write(Sendable::Response(resp));

                if let Some(src_path) = path {
                    let mut running_state = RunningState::new(src_path, vm_type)?;
                    running_state.init(server)?;
                    return Ok(Some(State::Running(running_state)));
                }
            }
            Command::Disconnect(_) => {
                server.write(Sendable::Response(request.ack()?));
                return Ok(Some(State::Exit));
            }
            _ => {
                return Err(DebuggingError::CustomError(
                    "Invalid request. You need to initialize and launch program.".to_string(),
                ))
            }
        }

        Ok(None)
    }
}

/// Implements main debugging functionality.
#[derive(Debug)]
pub struct RunningState {
    /// Breakpoints to interrupt execution.
    breakpoints: Vec<Breakpoint>,
    /// Indicate machine state.
    running: bool,
    /// Keep compiled program to provide access to bytecode and service information.
    program: GeneratedBrillig,
    /// Virtual machine that handles program.
    vm: VMType,
}

impl RunningState {
    /// Create instance using path to file and virtual machine type.
    pub(crate) fn new(src_path: &str, _vm_type: Option<&str>) -> Result<Self, DebuggingError> {
        let toml_path = get_package_manifest(std::path::Path::new(src_path)).unwrap();
        let workspace =
            resolve_workspace_from_toml(&toml_path, PackageSelection::DefaultOrAll).unwrap();

        let Some(package) = workspace.into_iter().find(|p| p.is_binary()) else {
            return Err(DebuggingError::CustomError("No matching binary packages found in workspace. Only binary packages can be debugged.".into()));
        };
        let program = compile::compile_brillig(package.entry_path.to_str().unwrap()).unwrap();

        let (registers, memory) = compile::get_input_registers_and_memory(
            package,
            &program.1,
            &format!("{}.toml", nargo::constants::PROVER_INPUT_FILE),
        );

        #[allow(deprecated)]
        let solver = Box::leak(Box::new(BarretenbergSolver::new()));
        let bytecode = Box::leak(Box::new(program.0.byte_code.clone()));
        let vm = vm::brillig_new(bytecode, registers, memory, solver);
        Ok(RunningState {
            breakpoints: Vec::new(),
            running: false,
            program: program.0,
            vm: VMType::Brillig(vm),
        })
    }

    /// Enter to debugging mode in the state.
    pub(crate) fn init<T: Server>(&mut self, server: &T) -> Result<(), DebuggingError> {
        self.stop(server, StoppedEventReason::Entry)
    }

    /// Clear the defined breakpoints.
    fn clear_breakpoints(&mut self) {
        self.breakpoints = vec![];
    }

    /// Stop machine and provide response with reason.
    fn stop<T: Server>(
        &mut self,
        server: &T,
        reason: StoppedEventReason,
    ) -> Result<(), DebuggingError> {
        let description = format!("{:?}", &reason);
        let stop_event = Event::Stopped(StoppedEventBody {
            reason,
            description: Some(description),
            thread_id: Some(0),
            preserve_focus_hint: Some(false),
            text: None,
            all_threads_stopped: Some(false),
            hit_breakpoint_ids: None,
        });

        server.write(Sendable::Event(stop_event));
        self.running = false;

        Ok(())
    }

    /// Getter for program counter.
    fn get_current_instruction(&self) -> usize {
        self.vm.program_counter()
    }

    /// Perform command from a request.
    pub(crate) fn run<T: Server>(
        &mut self,
        server: &mut T,
    ) -> Result<Option<State>, DebuggingError> {
        if self.running {
            let current_instruction = self.get_current_instruction();
            if self.breakpoints.iter().any(|b| b.instruction == current_instruction) {
                self.stop(server, StoppedEventReason::Breakpoint)?;
            }
        }
        let request = match server.read() {
            Some(req) => req,
            None => return Ok(None),
        };

        match request.command {
            Command::Next(_) | Command::StepIn(_) | Command::StepOut(_) => {
                match self.vm.process_opcode() {
                    VMStatus::InProgress => {
                        server.write(Sendable::Response(request.ack()?));
                        let pc = self.vm.program_counter();
                        println!(
                            "Stopped at position {}\nPerformed opcode: {:?}\nNext opcode: {:?}",
                            pc,
                            self.program.byte_code[pc - 1],
                            self.program.byte_code[pc]
                        );
                        self.stop(server, StoppedEventReason::Step)?;
                    }
                    // TODO: improve
                    VMStatus::Finished => {
                        server.write(Sendable::Response(Response {
                            request_seq: request.seq,
                            body: Some(ResponseBody::Terminate),
                            success: true,
                            message: None,
                            error: None,
                        }));
                        return Ok(Some(State::Exit));
                    }
                    VMStatus::Failure { message, call_stack: _ } => {
                        return Err(DebuggingError::CustomError(message));
                    }
                    _ => {
                        server.write(Sendable::Response(request.ack()?));
                        return Ok(Some(State::Exit));
                    }
                }
            }
            Command::Disassemble(_) => {
                let pos = self.vm.program_counter();
                let instructions = self
                    .program
                    .byte_code
                    .iter()
                    .enumerate()
                    .map(|(i, opcode)| {
                        let instruction = if pos == i {
                            // special formatting for current opcode
                            format!("==>\n<<<<<\n{:#?}\n>>>>>", opcode)
                        } else {
                            format!("{:#?}", opcode)
                        };
                        DisassembledInstruction {
                            address: i.to_string(),
                            instruction,
                            ..Default::default()
                        }
                    })
                    .collect::<Vec<_>>();

                server.write(Sendable::Response(Response {
                    request_seq: request.seq,
                    body: Some(ResponseBody::Disassemble(DisassembleResponse { instructions })),
                    success: true,
                    message: None,
                    error: None,
                }));
            }
            Command::Pause(_) => {
                self.running = false;
                server.write(Sendable::Response(request.ack()?));
                self.stop(server, StoppedEventReason::Pause)?;
            }
            Command::Continue(_) => {
                self.running = true;
                let seq = request.seq;
                server.write(Sendable::Response(request.success(ResponseBody::Continue(
                    ContinueResponse { all_threads_continued: Some(true) },
                ))));
                loop {
                    match self.vm.process_opcode() {
                        VMStatus::InProgress | VMStatus::ForeignCallWait { .. } => {}
                        VMStatus::Finished => {
                            server.write(Sendable::Response(Response {
                                request_seq: self.get_current_instruction() as i64 + seq,
                                body: Some(ResponseBody::Terminate),
                                success: true,
                                message: None,
                                error: None,
                            }));
                            return Ok(Some(State::Exit));
                        }
                        VMStatus::Failure { message, call_stack: _ } => {
                            return Err(DebuggingError::CustomError(message));
                        }
                    }
                    let current_instruction = self.get_current_instruction();
                    if self.breakpoints.iter().any(|b| b.instruction == current_instruction) {
                        self.stop(server, StoppedEventReason::Breakpoint)?;
                        break;
                    }
                }
            }
            Command::Threads => {
                server.write(Sendable::Response(request.success(ResponseBody::Threads(
                    ThreadsResponse { threads: vec![Thread { id: 0, name: "main".to_string() }] },
                ))));
            }
            Command::Scopes(ref args) => {
                if args.frame_id == 0 {
                    let scope = Scope {
                        name: "Locals".to_string(),
                        presentation_hint: Some(ScopePresentationhint::Locals),
                        variables_reference: 133,
                        named_variables: None,
                        indexed_variables: None,
                        line: Some(self.get_current_instruction() as i64),
                        ..Default::default()
                    };
                    server
                        .write(Sendable::Response(request.success(ResponseBody::Scopes(
                            ScopesResponse { scopes: vec![scope] },
                        ))));
                } else {
                    server.write(Sendable::Response(
                        request
                            .success(ResponseBody::Scopes(ScopesResponse { scopes: Vec::new() })),
                    ));
                }
            }
            Command::Variables(_) => {
                let registers = self.vm.get_registers();
                let variables = registers
                    .iter()
                    .enumerate()
                    .map(|(i, r)| Variable {
                        name: format!("Register {i}"),
                        value: format!("{}", r.to_u128()),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>();

                server.write(Sendable::Response(
                    request.success(ResponseBody::Variables(VariablesResponse { variables })),
                ));
            }
            Command::ReadMemory(_) => {
                let memory = self.vm.get_memory();
                let memory = memory.iter().map(|v| format!("{}", v.to_u128())).collect::<Vec<_>>();
                let memory_string = memory.join(", ");

                server.write(Sendable::Response(request.success(ResponseBody::ReadMemory(
                    ReadMemoryResponse {
                        address: "Memory".to_string(),
                        unreadable_bytes: None,
                        data: Some(memory_string),
                    },
                ))));
            }
            Command::SetBreakpoints(ref args) => {
                self.clear_breakpoints();
                if let Some(new_breakpoints) = &args.breakpoints {
                    let breakpoints = new_breakpoints.iter().filter_map(Breakpoint::new);
                    self.breakpoints.extend(breakpoints);
                }
            }
            Command::Disconnect(_) => {
                server.write(Sendable::Response(request.ack()?));
                return Ok(Some(State::Exit));
            }

            Command::SetExceptionBreakpoints(_) => {}
            _ => {
                return Err(DebuggingError::CustomError(
                    "Unsupported command. Please, use the other commands to continue process."
                        .to_string(),
                ))
            }
        }
        Ok(None)
    }
}

/// Application struct. Handles server operations and reports its state.
#[derive(Debug)]
pub struct App<T: Server> {
    pub state: State,
    pub server: T,
}

impl<T: Server> App<T> {
    /// Initialize app using any instance that implements trait Server.
    pub fn initialize(server: T) -> Self {
        App { state: State::Uninitialized(UninitializedState::new()), server }
    }

    /// Perform single run for app.
    pub fn run(&mut self) -> Result<(), DebuggingError> {
        let res = match self.state {
            State::Uninitialized(ref mut s) => s.run(&mut self.server)?,
            State::Running(ref mut s) => s.run(&mut self.server)?,
            State::Exit => return Ok(()),
        };

        if let Some(state) = res {
            self.state = state;
        }

        Ok(())
    }
}
