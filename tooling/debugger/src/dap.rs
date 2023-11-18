use std::io::{Read, Write};
use std::str::FromStr;

use acvm::acir::circuit::{Circuit, OpcodeLocation};
use acvm::acir::native_types::WitnessMap;
use acvm::BlackBoxFunctionSolver;

use crate::context::DebugCommandResult;
use crate::context::DebugContext;

use dap::errors::ServerError;
use dap::events::StoppedEventBody;
use dap::prelude::Event;
use dap::requests::{Command, Request};
use dap::responses::{
    DisassembleResponse, ResponseBody, ScopesResponse, SetBreakpointsResponse,
    SetExceptionBreakpointsResponse, SetInstructionBreakpointsResponse, StackTraceResponse,
    ThreadsResponse,
};
use dap::server::Server;
use dap::types::{DisassembledInstruction, Source, StackFrame, StoppedEventReason, Thread};
use nargo::artifacts::debug::DebugArtifact;
use nargo::ops::DefaultForeignCallExecutor;

use noirc_driver::CompiledProgram;

pub struct DapSession<'a, R: Read, W: Write, B: BlackBoxFunctionSolver> {
    server: Server<R, W>,
    context: DebugContext<'a, B>,
    debug_artifact: &'a DebugArtifact,
    running: bool,
}

impl<'a, R: Read, W: Write, B: BlackBoxFunctionSolver> DapSession<'a, R, W, B> {
    pub fn new(
        server: Server<R, W>,
        solver: &'a B,
        circuit: &'a Circuit,
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap,
    ) -> Self {
        let context = DebugContext::new(
            solver,
            circuit,
            debug_artifact,
            initial_witness,
            Box::new(DefaultForeignCallExecutor::new(true)),
        );
        Self { server, context, debug_artifact, running: false }
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
        self.running = true;

        if matches!(self.context.get_current_source_location(), None) {
            // FIXME: remove this?
            _ = self.context.next();
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
                Command::SetBreakpoints(ref args) => {
                    eprintln!("INFO: Received SetBreakpoints {:?}", args);
                    // FIXME: set and return the breakpoints actually set
                    self.server.respond(req.success(ResponseBody::SetBreakpoints(
                        SetBreakpointsResponse { breakpoints: vec![] },
                    )))?;
                }
                Command::SetExceptionBreakpoints(_) => {
                    self.server.respond(req.success(ResponseBody::SetExceptionBreakpoints(
                        SetExceptionBreakpointsResponse { breakpoints: None },
                    )))?;
                }
                Command::SetInstructionBreakpoints(ref args) => {
                    eprintln!("INFO: Received SetInstructionBreakpoints {:?}", args);
                    // FIXME: set and return the breakpoints actually set
                    self.server.respond(req.success(ResponseBody::SetInstructionBreakpoints(
                        SetInstructionBreakpointsResponse { breakpoints: vec![] },
                    )))?;
                }
                Command::Threads => {
                    self.server.respond(req.success(ResponseBody::Threads(ThreadsResponse {
                        threads: vec![Thread { id: 0, name: "main".to_string() }],
                    })))?;
                }
                Command::StackTrace(_) => {
                    let opcode_location = self.context.get_current_opcode_location();
                    let source_location = self.context.get_current_source_location();
                    let frames = match source_location {
                        None => vec![],
                        Some(locations) => locations
                            .iter()
                            .enumerate()
                            .map(|(index, location)| {
                                let line_number =
                                    self.debug_artifact.location_line_number(*location).unwrap();
                                let column_number =
                                    self.debug_artifact.location_column_number(*location).unwrap();
                                let ip_reference =
                                    opcode_location.map(|location| location.to_string());
                                StackFrame {
                                    id: index as i64,
                                    name: format!("frame #{index}"),
                                    source: Some(Source {
                                        path: self.debug_artifact.file_map[&location.file]
                                            .path
                                            .to_str()
                                            .map(String::from),
                                        ..Source::default()
                                    }),
                                    line: line_number as i64,
                                    column: column_number as i64,
                                    instruction_pointer_reference: ip_reference,
                                    ..StackFrame::default()
                                }
                            })
                            .collect(),
                    };
                    let total_frames = Some(frames.len() as i64);
                    self.server.respond(req.success(ResponseBody::StackTrace(
                        StackTraceResponse { stack_frames: frames, total_frames },
                    )))?;
                }
                Command::Disassemble(ref args) => {
                    let starting_ip = OpcodeLocation::from_str(args.memory_reference.as_str()).ok();
                    let instruction_offset = args.instruction_offset.unwrap_or(0);
                    let (mut opcode_location, mut invalid_count) =
                        self.context.offset_opcode_location(&starting_ip, instruction_offset);
                    let mut count = args.instruction_count;

                    let mut instructions: Vec<DisassembledInstruction> = vec![];

                    // leading invalid locations (when the request goes back
                    // beyond the start of the program)
                    if invalid_count < 0 {
                        while invalid_count < 0 {
                            instructions.push(DisassembledInstruction {
                                address: String::from("---"),
                                instruction: String::from("---"),
                                ..DisassembledInstruction::default()
                            });
                            invalid_count += 1;
                            count -= 1;
                        }
                        if count > 0 {
                            opcode_location = Some(OpcodeLocation::Acir(0));
                        }
                    }
                    // the actual opcodes
                    while count > 0 && !matches!(opcode_location, None) {
                        instructions.push(DisassembledInstruction {
                            address: format!("{}", opcode_location.unwrap()),
                            instruction: self.context.render_opcode_at_location(&opcode_location),
                            ..DisassembledInstruction::default()
                        });
                        (opcode_location, _) =
                            self.context.offset_opcode_location(&opcode_location, 1);
                        count -= 1;
                    }
                    // any remaining instruction count is beyond the valid opcode
                    // vector so return invalid placeholders
                    while count > 0 {
                        instructions.push(DisassembledInstruction {
                            address: String::from("---"),
                            instruction: String::from("---"),
                            ..DisassembledInstruction::default()
                        });
                        invalid_count -= 1;
                        count -= 1;
                    }

                    self.server.respond(req.success(ResponseBody::Disassemble(
                        DisassembleResponse { instructions },
                    )))?;
                }
                Command::Next(_) | Command::StepIn(_) | Command::StepOut(_) => {
                    self.handle_next(req)?;
                }
                Command::Continue(_) => {
                    self.handle_continue(req)?;
                }
                Command::Scopes(_) => {
                    // FIXME
                    self.server.respond(
                        req.success(ResponseBody::Scopes(ScopesResponse { scopes: vec![] })),
                    )?;
                }
                _ => {
                    eprintln!("ERROR: unhandled command: {:?}", req.command);
                }
            }
        }
        Ok(())
    }

    fn handle_next(&mut self, req: Request) -> Result<(), ServerError> {
        let result = self.context.next();
        eprintln!("INFO: stepped with result {result:?}");
        self.handle_execution_result(req, result)
    }

    fn handle_continue(&mut self, req: Request) -> Result<(), ServerError> {
        let result = self.context.cont();
        eprintln!("INFO: continue with result {result:?}");
        self.handle_execution_result(req, result)
    }

    fn handle_execution_result(
        &mut self,
        req: Request,
        result: DebugCommandResult,
    ) -> Result<(), ServerError> {
        match result {
            DebugCommandResult::Done => {
                self.server.respond(req.success(ResponseBody::Terminate))?;
                self.running = false;
            }
            _ => {
                self.server.respond(req.ack()?)?;
                self.send_stopped_event(StoppedEventReason::Pause)?;
            }
        }
        Ok(())
    }
}

pub fn run_session<R: Read, W: Write, B: BlackBoxFunctionSolver>(
    server: Server<R, W>,
    solver: &B,
    program: CompiledProgram,
    initial_witness: WitnessMap,
) -> Result<(), ServerError> {
    let debug_artifact = DebugArtifact {
        debug_symbols: vec![program.debug.clone()],
        file_map: program.file_map.clone(),
        warnings: program.warnings.clone(),
    };
    let mut session =
        DapSession::new(server, solver, &program.circuit, &debug_artifact, initial_witness);

    session.run_loop()
}
