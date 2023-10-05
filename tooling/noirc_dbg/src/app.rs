use dap::base_message::Sendable;
use dap::events::*;
use dap::requests::*;
use dap::responses::*;
use dap::types::{
    Capabilities, Scope, ScopePresentationhint, SourceBreakpoint, StoppedEventReason, Thread, Variable
};
use serde_json::Value;

use crate::{compile, dap_server, vm};

use anyhow::{Context, Result};

use acvm::brillig_vm::VM;
#[allow(deprecated)]
use barretenberg_blackbox_solver::BarretenbergSolver;

#[derive(Clone, Debug)]
struct Breakpoint {
    instruction: usize,
}

impl Breakpoint {
    pub(crate) fn new(breakpoint: &SourceBreakpoint) -> Option<Self> {
        Some(Breakpoint { instruction: breakpoint.line as usize })
    }
}

pub(crate) enum State {
    Uninitialized(UninitializedState),
    Running(RunningState),
    Exit,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct UninitializedState;

impl UninitializedState {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn run(&mut self) -> Result<Option<State>> {
        let request = match dap_server::read() {
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

                dap_server::write(Sendable::Response(rsp));
            }

            Command::Launch(ref arguments) => {
                let mut path = None;
                let additional_data = arguments.additional_data.clone();
                let resp = if let Some(Value::Object(data)) = &additional_data {
                    path = data
                        .get("src_path")
                        .context("Missing source file")?
                        .as_str()
                        .context("Source file is not a string")
                        .ok();
                    dap_server::write(Sendable::Event(Event::Initialized));
                    request.ack()?
                } else {
                    request.error("Source file is not provided")
                };

                dap_server::write(Sendable::Response(resp));

                if let Some(src_path) = path {
                    let mut running_state = RunningState::new(src_path)?;
                    running_state.init()?;
                    return Ok(Some(State::Running(running_state)));
                }
            }

            _ => panic!("Invalid request"),
        }

        Ok(None)
    }
}
#[allow(deprecated)]
pub(crate) struct RunningState {
    breakpoints: Vec<Breakpoint>,
    running: bool,
    vm: VM<'static, BarretenbergSolver>,
}

impl RunningState {
    pub(crate) fn new(src_path: &str) -> Result<Self> {
        let program =
            compile(std::env::current_dir().unwrap().join(src_path).as_path().to_str().unwrap())
                .unwrap();

        #[allow(deprecated)]
        let solver = Box::leak(Box::new(BarretenbergSolver::new()));
        let vm = vm::new(program, solver);
        Ok(RunningState { breakpoints: Vec::new(), running: false, vm })
    }

    pub(crate) fn init(&mut self) -> Result<()> {
        self.stop(StoppedEventReason::Entry)
    }

    fn clear_breakpoints(&mut self) {
        self.breakpoints = vec![];
    }

    fn stop(&mut self, reason: StoppedEventReason) -> Result<()> {
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

        dap_server::write(Sendable::Event(stop_event));
        self.running = false;

        Ok(())
    }

    fn get_current_instruction(&self) -> usize {
        0
        // self.vm.program_counter()
    }

    pub(crate) fn run(&mut self) -> Result<Option<State>> {
        if self.running {
            let current_instruction = self.get_current_instruction();
            if self.breakpoints.iter().any(|b| b.instruction == current_instruction) {
                self.stop(StoppedEventReason::Breakpoint)?;
            }
        }
        let request = match dap_server::read() {
            Some(req) => req,
            None => return Ok(None),
        };

        match request.command {
            Command::Next(_) | Command::StepIn(_) | Command::StepOut(_) => {
                self.vm.process_opcode();
                dap_server::write(Sendable::Response(request.ack()?));
                self.stop(StoppedEventReason::Step)?;
            }
            Command::Pause(_) => {
                self.running = false;
                dap_server::write(Sendable::Response(request.ack()?));
                self.stop(StoppedEventReason::Pause)?;
            }
            Command::Continue(_) => {
                self.running = true;
                dap_server::write(Sendable::Response(request.success(ResponseBody::Continue(
                    ContinueResponse { all_threads_continued: Some(true) },
                ))));
            }
            Command::Threads => {
                dap_server::write(Sendable::Response(request.success(ResponseBody::Threads(
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
                    dap_server::write(Sendable::Response(
                        request
                            .success(ResponseBody::Scopes(ScopesResponse { scopes: vec![scope] })),
                    ));
                } else {
                    dap_server::write(Sendable::Response(
                        request
                            .success(ResponseBody::Scopes(ScopesResponse { scopes: Vec::new() })),
                    ));
                }
            }
            Command::Variables(_) => {
                let registers = self.vm.get_registers();
                let variables = registers.inner.iter().enumerate().map(|(i, r)|{
                    Variable {
                        name: format!("Register {i}"),
                        value: format!("{}", r.to_u128()),
                        ..Default::default()
                    }
                }).collect::<Vec<_>>();

                dap_server::write(Sendable::Response(request.success(
                    ResponseBody::Variables(VariablesResponse { variables }),
                )));
            }
            Command::ReadMemory(_) => {
                let memory = self.vm.get_memory();
                let memory = memory.iter().map(|v|{
                    format!("{}", v.to_u128())
                }).collect::<Vec<_>>();
                let memory_string = memory.join(".");

                dap_server::write(Sendable::Response(request.success(
                    ResponseBody::ReadMemory(
                        ReadMemoryResponse {
                            address: "Memory".to_string(),
                            unreadable_bytes: None,
                            data: Some(memory_string),
                        })
                )));
            }
            Command::SetBreakpoints(ref args) => {
                self.clear_breakpoints();
                if let Some(new_breakpoints) = &args.breakpoints {
                    let breakpoints = new_breakpoints.iter().filter_map(|b| Breakpoint::new(b));
                    self.breakpoints.extend(breakpoints);
                }
            }
            Command::Disconnect(_) => {
                dap_server::write(Sendable::Response(request.ack()?));
                return Ok(Some(State::Exit));
            }

            Command::SetExceptionBreakpoints(_) => {}
            _ => panic!("not supported"),
        }
        Ok(None)
    }
}

pub(crate) struct App {
    pub(crate) state: State,
}

impl App {
    pub(crate) fn initialize() -> Self {
        App { state: State::Uninitialized(UninitializedState::new()) }
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        let res = match self.state {
            State::Uninitialized(ref mut s) => s.run()?,
            State::Running(ref mut s) => s.run()?,
            State::Exit => return Ok(()),
        };

        if let Some(state) = res {
            self.state = state;
        }

        Ok(())
    }
}
