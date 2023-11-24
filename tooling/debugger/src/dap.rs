use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::str::FromStr;

use acvm::acir::circuit::{Circuit, OpcodeLocation};
use acvm::acir::native_types::WitnessMap;
use acvm::BlackBoxFunctionSolver;
use codespan_reporting::files::{Files, SimpleFile};

use crate::context::DebugCommandResult;
use crate::context::DebugContext;

use dap::errors::ServerError;
use dap::events::StoppedEventBody;
use dap::prelude::Event;
use dap::requests::{Command, Request};
use dap::responses::{
    ContinueResponse, DisassembleResponse, ResponseBody, ScopesResponse, SetBreakpointsResponse,
    SetExceptionBreakpointsResponse, SetInstructionBreakpointsResponse, StackTraceResponse,
    ThreadsResponse,
};
use dap::server::Server;
use dap::types::{
    Breakpoint, DisassembledInstruction, Source, StackFrame, StoppedEventReason, Thread,
};
use nargo::artifacts::debug::DebugArtifact;
use nargo::ops::DefaultForeignCallExecutor;

use fm::FileId;
use noirc_driver::CompiledProgram;

pub struct DapSession<'a, R: Read, W: Write, B: BlackBoxFunctionSolver> {
    server: Server<R, W>,
    context: DebugContext<'a, B>,
    debug_artifact: &'a DebugArtifact,
    running: bool,
    source_to_opcodes: BTreeMap<FileId, Vec<(usize, OpcodeLocation)>>,
}

// BTreeMap<FileId, Vec<(usize, OpcodeLocation)>

impl<'a, R: Read, W: Write, B: BlackBoxFunctionSolver> DapSession<'a, R, W, B> {
    pub fn new(
        server: Server<R, W>,
        solver: &'a B,
        circuit: &'a Circuit,
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap,
    ) -> Self {
        let source_to_opcodes = Self::build_source_to_opcode_debug_mappings(debug_artifact);
        let context = DebugContext::new(
            solver,
            circuit,
            debug_artifact,
            initial_witness,
            Box::new(DefaultForeignCallExecutor::new(true)),
        );
        Self { server, context, debug_artifact, source_to_opcodes, running: false }
    }

    /// Builds a map from FileId to an ordered vector of tuples with line
    /// numbers and opcode locations correspoding to those line numbers
    fn build_source_to_opcode_debug_mappings(
        debug_artifact: &'a DebugArtifact,
    ) -> BTreeMap<FileId, Vec<(usize, OpcodeLocation)>> {
        let mut result = BTreeMap::new();
        if debug_artifact.debug_symbols.is_empty() {
            return result;
        }
        let locations = &debug_artifact.debug_symbols[0].locations;
        let mut simple_files = BTreeMap::new();
        debug_artifact.file_map.iter().for_each(|(file_id, debug_file)| {
            simple_files.insert(
                file_id,
                SimpleFile::new(debug_file.path.to_str().unwrap(), debug_file.source.as_str()),
            );
        });

        locations.iter().for_each(|(opcode_location, source_locations)| {
            if source_locations.is_empty() {
                return;
            }
            let source_location = source_locations[0];
            let span = source_location.span;
            let file_id = source_location.file;
            let Ok(line_index) = &simple_files[&file_id].line_index((), span.start() as usize) else {
                return;
            };
            let line_number = line_index + 1;

            if result.contains_key(&file_id) {
                result.get_mut(&file_id).unwrap().push((line_number, *opcode_location));
            } else {
                result.insert(file_id, vec![(line_number, *opcode_location)]);
            }
        });
        result.iter_mut().for_each(|(_, file_locations)| file_locations.sort_by_key(|x| x.0));
        result
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

    fn handle_stack_trace(&mut self, req: Request) -> Result<(), ServerError> {
        let opcode_location = self.context.get_current_opcode_location();
        let source_location = self.context.get_current_source_location();
        let frames = match source_location {
            None => vec![],
            Some(locations) => locations
                .iter()
                .enumerate()
                .map(|(index, location)| {
                    let line_number = self.debug_artifact.location_line_number(*location).unwrap();
                    let column_number =
                        self.debug_artifact.location_column_number(*location).unwrap();
                    let ip_reference = opcode_location.map(|location| location.to_string());
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
            (opcode_location, _) = self.context.offset_opcode_location(&opcode_location, 1);
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

        self.server.respond(
            req.success(ResponseBody::Disassemble(DisassembleResponse { instructions })),
        )?;
        Ok(())
    }

    fn handle_next(&mut self, req: Request) -> Result<(), ServerError> {
        let result = self.context.next();
        eprintln!("INFO: stepped with result {result:?}");
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

    fn handle_execution_result(&mut self, result: DebugCommandResult) -> Result<(), ServerError> {
        match result {
            DebugCommandResult::Done => {
                self.running = false;
            }
            _ => {
                self.send_stopped_event(StoppedEventReason::Pause)?;
            }
        }
        Ok(())
    }

    fn handle_set_instruction_breakpoints(&mut self, req: Request) -> Result<(), ServerError> {
        let Command::SetInstructionBreakpoints(ref args) = req.command else {
            unreachable!("handle_set_instruction_breakpoints called on a different request");
        };
        // FIXME: clear previous instruction breakpoints
        let breakpoints = args.breakpoints.iter().filter_map(|breakpoint| {
            let Ok(location) = OpcodeLocation::from_str(breakpoint.instruction_reference.as_str()) else {
                return None;
            };
            if !self.context.is_valid_opcode_location(&location) {
                return None;
            }
            if self.context.add_breakpoint(location) {
                Some(
                    Breakpoint {
                        verified: true,
                        instruction_reference: Some(breakpoint.instruction_reference.clone()),
                        ..Breakpoint::default()
                    }
                )
            } else {
                None
            }
        }).collect();
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
        if let Some(iter) = found {
            Some(*iter.0)
        } else {
            None
        }
    }

    fn find_opcode_for_source_location(&self, source: &str, line: i64) -> Option<OpcodeLocation> {
        let line = line as usize;
        let Some(file_id) = self.find_file_id(source) else {
            return None;
        };
        if self.debug_artifact.debug_symbols.is_empty() {
            return None;
        }
        let Some(line_to_opcodes) = self.source_to_opcodes.get(&file_id) else {
            return None;
        };
        let found_index = match line_to_opcodes.binary_search_by(|x| x.0.cmp(&line)) {
            Ok(index) => line_to_opcodes[index].1,
            Err(index) => line_to_opcodes[index].1,
        };
        Some(found_index)
    }

    fn handle_set_source_breakpoints(&mut self, req: Request) -> Result<(), ServerError> {
        let Command::SetBreakpoints(ref args) = req.command else {
            unreachable!("handle_set_source_breakpoints called on a different request");
        };
        let Some(ref source) = &args.source.path else {
            self.server.respond(
                req.success(ResponseBody::SetBreakpoints(SetBreakpointsResponse { breakpoints: vec![] })),
            )?;
            return Ok(());
        };
        let Some(ref breakpoints) = &args.breakpoints else {
            self.server.respond(
                req.success(ResponseBody::SetBreakpoints(SetBreakpointsResponse { breakpoints: vec![] })),
            )?;
            return Ok(());
        };
        // FIXME: clear previous source breakpoints on this source
        let breakpoints = breakpoints
            .iter()
            .filter_map(|breakpoint| {
                let line = breakpoint.line;
                let Some(location) = self.find_opcode_for_source_location(source, line) else {
                    return None;
                };
                if !self.context.is_valid_opcode_location(&location) {
                    return None;
                }
                if self.context.add_breakpoint(location) {
                    let instruction_reference = format!("{}", location);
                    Some(Breakpoint {
                        verified: true,
                        source: Some(args.source.clone()),
                        instruction_reference: Some(instruction_reference),
                        line: Some(line),
                        ..Breakpoint::default()
                    })
                } else {
                    None
                }
            })
            .collect();

        self.server.respond(
            req.success(ResponseBody::SetBreakpoints(SetBreakpointsResponse { breakpoints })),
        )?;
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
