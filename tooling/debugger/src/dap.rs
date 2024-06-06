use std::collections::BTreeMap;
use std::io::{Read, Write};

use acvm::acir::circuit::brillig::BrilligBytecode;
use acvm::acir::circuit::{Circuit, OpcodeLocation};
use acvm::acir::native_types::WitnessMap;
use acvm::{BlackBoxFunctionSolver, FieldElement};

use crate::context::DebugCommandResult;
use crate::context::DebugContext;
use crate::foreign_calls::DefaultDebugForeignCallExecutor;

use dap::errors::ServerError;
use dap::events::StoppedEventBody;
use dap::prelude::Event;
use dap::requests::{Command, Request, SetBreakpointsArguments};
use dap::responses::{
    ContinueResponse, DisassembleResponse, ResponseBody, ScopesResponse, SetBreakpointsResponse,
    SetExceptionBreakpointsResponse, SetInstructionBreakpointsResponse, StackTraceResponse,
    ThreadsResponse, VariablesResponse,
};
use dap::server::Server;
use dap::types::{
    Breakpoint, DisassembledInstruction, Scope, Source, StackFrame, SteppingGranularity,
    StoppedEventReason, Thread, Variable,
};
use nargo::artifacts::debug::DebugArtifact;

use fm::FileId;
use noirc_driver::CompiledProgram;

type BreakpointId = i64;

pub struct DapSession<'a, R: Read, W: Write, B: BlackBoxFunctionSolver<FieldElement>> {
    server: Server<R, W>,
    context: DebugContext<'a, B>,
    debug_artifact: &'a DebugArtifact,
    running: bool,
    next_breakpoint_id: BreakpointId,
    instruction_breakpoints: Vec<(OpcodeLocation, BreakpointId)>,
    source_breakpoints: BTreeMap<FileId, Vec<(OpcodeLocation, BreakpointId)>>,
}

enum ScopeReferences {
    Locals = 1,
    WitnessMap = 2,
    InvalidScope = 0,
}

impl From<i64> for ScopeReferences {
    fn from(value: i64) -> Self {
        match value {
            1 => Self::Locals,
            2 => Self::WitnessMap,
            _ => Self::InvalidScope,
        }
    }
}

impl<'a, R: Read, W: Write, B: BlackBoxFunctionSolver<FieldElement>> DapSession<'a, R, W, B> {
    pub fn new(
        server: Server<R, W>,
        solver: &'a B,
        circuit: &'a Circuit<FieldElement>,
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap<FieldElement>,
        unconstrained_functions: &'a [BrilligBytecode<FieldElement>],
    ) -> Self {
        let context = DebugContext::new(
            solver,
            circuit,
            debug_artifact,
            initial_witness,
            Box::new(DefaultDebugForeignCallExecutor::from_artifact(true, debug_artifact)),
            unconstrained_functions,
        );
        Self {
            server,
            context,
            debug_artifact,
            running: false,
            next_breakpoint_id: 1,
            instruction_breakpoints: vec![],
            source_breakpoints: BTreeMap::new(),
        }
    }

    fn send_stopped_event(&mut self, reason: StoppedEventReason) -> Result<(), ServerError> {
        let description = format!("{:?}", &reason);
        self.server.send_event(Event::Stopped(StoppedEventBody {
            reason,
            description: Some(description),
            thread_id: Some(0),
            preserve_focus_hint: Some(false),
            text: None,
            all_threads_stopped: Some(false),
            hit_breakpoint_ids: None,
        }))?;
        Ok(())
    }

    pub fn run_loop(&mut self) -> Result<(), ServerError> {
        self.running = self.context.get_current_opcode_location().is_some();

        if self.running && self.context.get_current_source_location().is_none() {
            // TODO: remove this? This is to ensure that the tool has a proper
            // source location to show when first starting the debugger, but
            // maybe the default behavior should be to start executing until the
            // first breakpoint set.
            _ = self.context.next_into();
        }

        self.server.send_event(Event::Initialized)?;
        self.send_stopped_event(StoppedEventReason::Entry)?;

        while self.running {
            let req = match self.server.poll_request()? {
                Some(req) => req,
                None => break,
            };
            match req.command {
                Command::Disconnect(_) => {
                    eprintln!("INFO: ending debugging session");
                    self.server.respond(req.ack()?)?;
                    break;
                }
                Command::SetBreakpoints(_) => {
                    self.handle_set_source_breakpoints(req)?;
                }
                Command::SetExceptionBreakpoints(_) => {
                    self.server.respond(req.success(ResponseBody::SetExceptionBreakpoints(
                        SetExceptionBreakpointsResponse { breakpoints: None },
                    )))?;
                }
                Command::SetInstructionBreakpoints(_) => {
                    self.handle_set_instruction_breakpoints(req)?;
                }
                Command::Threads => {
                    self.server.respond(req.success(ResponseBody::Threads(ThreadsResponse {
                        threads: vec![Thread { id: 0, name: "main".to_string() }],
                    })))?;
                }
                Command::StackTrace(_) => {
                    self.handle_stack_trace(req)?;
                }
                Command::Disassemble(_) => {
                    self.handle_disassemble(req)?;
                }
                Command::StepIn(ref args) => {
                    let granularity =
                        args.granularity.as_ref().unwrap_or(&SteppingGranularity::Statement);
                    match granularity {
                        SteppingGranularity::Instruction => self.handle_step(req)?,
                        _ => self.handle_next_into(req)?,
                    }
                }
                Command::StepOut(ref args) => {
                    let granularity =
                        args.granularity.as_ref().unwrap_or(&SteppingGranularity::Statement);
                    match granularity {
                        SteppingGranularity::Instruction => self.handle_step(req)?,
                        _ => self.handle_next_out(req)?,
                    }
                }
                Command::Next(ref args) => {
                    let granularity =
                        args.granularity.as_ref().unwrap_or(&SteppingGranularity::Statement);
                    match granularity {
                        SteppingGranularity::Instruction => self.handle_step(req)?,
                        _ => self.handle_next_over(req)?,
                    }
                }
                Command::Continue(_) => {
                    self.handle_continue(req)?;
                }
                Command::Scopes(_) => {
                    self.handle_scopes(req)?;
                }
                Command::Variables(ref _args) => {
                    self.handle_variables(req)?;
                }
                _ => {
                    eprintln!("ERROR: unhandled command: {:?}", req.command);
                }
            }
        }
        Ok(())
    }

    fn build_stack_trace(&self) -> Vec<StackFrame> {
        let stack_frames = self.context.get_variables();

        self.context
            .get_source_call_stack()
            .iter()
            .enumerate()
            .map(|(index, (opcode_location, source_location))| {
                let line_number =
                    self.debug_artifact.location_line_number(*source_location).unwrap();
                let column_number =
                    self.debug_artifact.location_column_number(*source_location).unwrap();

                let name = match stack_frames.get(index) {
                    Some(frame) => format!("{} {}", frame.function_name, index),
                    None => format!("frame #{index}"),
                };
                let address = self.context.opcode_location_to_address(opcode_location);

                StackFrame {
                    id: index as i64,
                    name,
                    source: Some(Source {
                        path: self.debug_artifact.file_map[&source_location.file]
                            .path
                            .to_str()
                            .map(String::from),
                        ..Source::default()
                    }),
                    line: line_number as i64,
                    column: column_number as i64,
                    instruction_pointer_reference: Some(address.to_string()),
                    ..StackFrame::default()
                }
            })
            .rev()
            .collect()
    }

    fn handle_stack_trace(&mut self, req: Request) -> Result<(), ServerError> {
        let frames = self.build_stack_trace();
        let total_frames = Some(frames.len() as i64);
        self.server.respond(req.success(ResponseBody::StackTrace(StackTraceResponse {
            stack_frames: frames,
            total_frames,
        })))?;
        Ok(())
    }

    fn handle_disassemble(&mut self, req: Request) -> Result<(), ServerError> {
        let Command::Disassemble(ref args) = req.command else {
            unreachable!("handle_disassemble called on a non disassemble request");
        };

        // we assume memory references are unsigned integers
        let starting_address = args.memory_reference.parse::<i64>().unwrap_or(0);
        let instruction_offset = args.instruction_offset.unwrap_or(0);

        let mut address = starting_address + instruction_offset;
        let mut count = args.instruction_count;

        let mut instructions: Vec<DisassembledInstruction> = vec![];

        while count > 0 {
            let opcode_location = if address >= 0 {
                self.context.address_to_opcode_location(address as usize)
            } else {
                None
            };

            if let Some(opcode_location) = opcode_location {
                instructions.push(DisassembledInstruction {
                    address: address.to_string(),
                    // we'll use the instruction_bytes field to render the OpcodeLocation
                    instruction_bytes: Some(opcode_location.to_string()),
                    instruction: self.context.render_opcode_at_location(&opcode_location),
                    ..DisassembledInstruction::default()
                });
            } else {
                // entry for invalid location to fill up the request
                instructions.push(DisassembledInstruction {
                    address: "---".to_owned(),
                    instruction: "---".to_owned(),
                    ..DisassembledInstruction::default()
                });
            }
            count -= 1;
            address += 1;
        }

        self.server.respond(
            req.success(ResponseBody::Disassemble(DisassembleResponse { instructions })),
        )?;
        Ok(())
    }

    fn handle_step(&mut self, req: Request) -> Result<(), ServerError> {
        let result = self.context.step_into_opcode();
        eprintln!("INFO: stepped by instruction with result {result:?}");
        self.server.respond(req.ack()?)?;
        self.handle_execution_result(result)
    }

    fn handle_next_into(&mut self, req: Request) -> Result<(), ServerError> {
        let result = self.context.next_into();
        eprintln!("INFO: stepped into by statement with result {result:?}");
        self.server.respond(req.ack()?)?;
        self.handle_execution_result(result)
    }

    fn handle_next_out(&mut self, req: Request) -> Result<(), ServerError> {
        let result = self.context.next_out();
        eprintln!("INFO: stepped out by statement with result {result:?}");
        self.server.respond(req.ack()?)?;
        self.handle_execution_result(result)
    }

    fn handle_next_over(&mut self, req: Request) -> Result<(), ServerError> {
        let result = self.context.next_over();
        eprintln!("INFO: stepped over by statement with result {result:?}");
        self.server.respond(req.ack()?)?;
        self.handle_execution_result(result)
    }

    fn handle_continue(&mut self, req: Request) -> Result<(), ServerError> {
        let result = self.context.cont();
        eprintln!("INFO: continue with result {result:?}");
        self.server.respond(req.success(ResponseBody::Continue(ContinueResponse {
            all_threads_continued: Some(true),
        })))?;
        self.handle_execution_result(result)
    }

    fn find_breakpoints_at_location(&self, opcode_location: &OpcodeLocation) -> Vec<i64> {
        let mut result = vec![];
        for (location, id) in &self.instruction_breakpoints {
            if opcode_location == location {
                result.push(*id);
            }
        }
        for breakpoints in self.source_breakpoints.values() {
            for (location, id) in breakpoints {
                if opcode_location == location {
                    result.push(*id);
                }
            }
        }
        result
    }

    fn handle_execution_result(&mut self, result: DebugCommandResult) -> Result<(), ServerError> {
        match result {
            DebugCommandResult::Done => {
                self.running = false;
            }
            DebugCommandResult::Ok => {
                self.server.send_event(Event::Stopped(StoppedEventBody {
                    reason: StoppedEventReason::Pause,
                    description: None,
                    thread_id: Some(0),
                    preserve_focus_hint: Some(false),
                    text: None,
                    all_threads_stopped: Some(false),
                    hit_breakpoint_ids: None,
                }))?;
            }
            DebugCommandResult::BreakpointReached(location) => {
                let breakpoint_ids = self.find_breakpoints_at_location(&location);
                self.server.send_event(Event::Stopped(StoppedEventBody {
                    reason: StoppedEventReason::Breakpoint,
                    description: Some(String::from("Paused at breakpoint")),
                    thread_id: Some(0),
                    preserve_focus_hint: Some(false),
                    text: None,
                    all_threads_stopped: Some(false),
                    hit_breakpoint_ids: Some(breakpoint_ids),
                }))?;
            }
            DebugCommandResult::Error(err) => {
                self.server.send_event(Event::Stopped(StoppedEventBody {
                    reason: StoppedEventReason::Exception,
                    description: Some(format!("{err:?}")),
                    thread_id: Some(0),
                    preserve_focus_hint: Some(false),
                    text: None,
                    all_threads_stopped: Some(false),
                    hit_breakpoint_ids: None,
                }))?;
            }
        }
        Ok(())
    }

    fn get_next_breakpoint_id(&mut self) -> BreakpointId {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;
        id
    }

    fn reinstall_breakpoints(&mut self) {
        self.context.clear_breakpoints();
        for (location, _) in &self.instruction_breakpoints {
            self.context.add_breakpoint(*location);
        }
        for breakpoints in self.source_breakpoints.values() {
            for (location, _) in breakpoints {
                self.context.add_breakpoint(*location);
            }
        }
    }

    fn handle_set_instruction_breakpoints(&mut self, req: Request) -> Result<(), ServerError> {
        let Command::SetInstructionBreakpoints(ref args) = req.command else {
            unreachable!("handle_set_instruction_breakpoints called on a different request");
        };

        // compute breakpoints to set and return
        let mut breakpoints_to_set: Vec<(OpcodeLocation, i64)> = vec![];
        let breakpoints: Vec<Breakpoint> = args
            .breakpoints
            .iter()
            .map(|breakpoint| {
                let offset = breakpoint.offset.unwrap_or(0);
                let address = breakpoint.instruction_reference.parse::<i64>().unwrap_or(0) + offset;
                let Ok(address): Result<usize, _> = address.try_into() else {
                    return Breakpoint {
                        verified: false,
                        message: Some(String::from("Invalid instruction reference/offset")),
                        ..Breakpoint::default()
                    };
                };
                let Some(location) = self
                    .context
                    .address_to_opcode_location(address)
                    .filter(|location| self.context.is_valid_opcode_location(location))
                else {
                    return Breakpoint {
                        verified: false,
                        message: Some(String::from("Invalid opcode location")),
                        ..Breakpoint::default()
                    };
                };
                let id = self.get_next_breakpoint_id();
                breakpoints_to_set.push((location, id));
                Breakpoint {
                    id: Some(id),
                    verified: true,
                    offset: Some(0),
                    instruction_reference: Some(address.to_string()),
                    ..Breakpoint::default()
                }
            })
            .collect();

        // actually set the computed breakpoints
        self.instruction_breakpoints = breakpoints_to_set;
        self.reinstall_breakpoints();

        // response to request
        self.server.respond(req.success(ResponseBody::SetInstructionBreakpoints(
            SetInstructionBreakpointsResponse { breakpoints },
        )))?;
        Ok(())
    }

    fn find_file_id(&self, source_path: &str) -> Option<FileId> {
        let file_map = &self.debug_artifact.file_map;
        let found = file_map.iter().find(|(_, debug_file)| match debug_file.path.to_str() {
            Some(debug_file_path) => debug_file_path == source_path,
            None => false,
        });
        found.map(|iter| *iter.0)
    }

    fn map_source_breakpoints(&mut self, args: &SetBreakpointsArguments) -> Vec<Breakpoint> {
        let Some(ref source) = &args.source.path else {
            return vec![];
        };
        let Some(file_id) = self.find_file_id(source) else {
            eprintln!("WARN: file ID for source {source} not found");
            return vec![];
        };
        let Some(ref breakpoints) = &args.breakpoints else {
            return vec![];
        };
        let mut breakpoints_to_set: Vec<(OpcodeLocation, i64)> = vec![];
        let breakpoints = breakpoints
            .iter()
            .map(|breakpoint| {
                let line = breakpoint.line;
                let Some(location) = self.context.find_opcode_for_source_location(&file_id, line)
                else {
                    return Breakpoint {
                        verified: false,
                        message: Some(String::from(
                            "Source location cannot be matched to opcode location",
                        )),
                        ..Breakpoint::default()
                    };
                };
                // TODO: line will not necessarily be the one requested; we
                // should do the reverse mapping and retrieve the actual source
                // code line number
                if !self.context.is_valid_opcode_location(&location) {
                    return Breakpoint {
                        verified: false,
                        message: Some(String::from("Invalid opcode location")),
                        ..Breakpoint::default()
                    };
                }
                let breakpoint_address = self.context.opcode_location_to_address(&location);
                let instruction_reference = format!("{}", breakpoint_address);
                let breakpoint_id = self.get_next_breakpoint_id();
                breakpoints_to_set.push((location, breakpoint_id));
                Breakpoint {
                    id: Some(breakpoint_id),
                    verified: true,
                    source: Some(args.source.clone()),
                    line: Some(line),
                    instruction_reference: Some(instruction_reference),
                    offset: Some(0),
                    ..Breakpoint::default()
                }
            })
            .collect();

        self.source_breakpoints.insert(file_id, breakpoints_to_set);

        breakpoints
    }

    fn handle_set_source_breakpoints(&mut self, req: Request) -> Result<(), ServerError> {
        let Command::SetBreakpoints(ref args) = req.command else {
            unreachable!("handle_set_source_breakpoints called on a different request");
        };
        let breakpoints = self.map_source_breakpoints(args);
        self.reinstall_breakpoints();
        self.server.respond(
            req.success(ResponseBody::SetBreakpoints(SetBreakpointsResponse { breakpoints })),
        )?;
        Ok(())
    }

    fn handle_scopes(&mut self, req: Request) -> Result<(), ServerError> {
        self.server.respond(req.success(ResponseBody::Scopes(ScopesResponse {
            scopes: vec![
                Scope {
                    name: String::from("Locals"),
                    variables_reference: ScopeReferences::Locals as i64,
                    ..Scope::default()
                },
                Scope {
                    name: String::from("Witness Map"),
                    variables_reference: ScopeReferences::WitnessMap as i64,
                    ..Scope::default()
                },
            ],
        })))?;
        Ok(())
    }

    fn build_local_variables(&self) -> Vec<Variable> {
        let Some(current_stack_frame) = self.context.current_stack_frame() else {
            return vec![];
        };

        let mut variables = current_stack_frame
            .variables
            .iter()
            .map(|(name, value, _var_type)| Variable {
                name: String::from(*name),
                value: format!("{:?}", *value),
                ..Variable::default()
            })
            .collect::<Vec<Variable>>();

        variables.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        variables
    }

    fn build_witness_map(&self) -> Vec<Variable> {
        self.context
            .get_witness_map()
            .clone()
            .into_iter()
            .map(|(witness, value)| Variable {
                name: format!("_{}", witness.witness_index()),
                value: format!("{value:?}"),
                ..Variable::default()
            })
            .collect()
    }

    fn handle_variables(&mut self, req: Request) -> Result<(), ServerError> {
        let Command::Variables(ref args) = req.command else {
            unreachable!("handle_variables called on a different request");
        };
        let scope: ScopeReferences = args.variables_reference.into();
        let variables: Vec<_> = match scope {
            ScopeReferences::Locals => self.build_local_variables(),
            ScopeReferences::WitnessMap => self.build_witness_map(),
            _ => {
                eprintln!(
                    "handle_variables with an unknown variables_reference {}",
                    args.variables_reference
                );
                vec![]
            }
        };
        self.server
            .respond(req.success(ResponseBody::Variables(VariablesResponse { variables })))?;
        Ok(())
    }
}

pub fn run_session<R: Read, W: Write, B: BlackBoxFunctionSolver<FieldElement>>(
    server: Server<R, W>,
    solver: &B,
    program: CompiledProgram,
    initial_witness: WitnessMap<FieldElement>,
) -> Result<(), ServerError> {
    let debug_artifact = DebugArtifact { debug_symbols: program.debug, file_map: program.file_map };
    let mut session = DapSession::new(
        server,
        solver,
        &program.program.functions[0],
        &debug_artifact,
        initial_witness,
        &program.program.unconstrained_functions,
    );

    session.run_loop()
}
