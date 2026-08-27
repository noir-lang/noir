use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};

use dap::events::StoppedEventBody;
use dap::prelude::Event;
use dap::requests::Command;
use dap::responses::{
    ResponseBody, ScopesResponse, SetBreakpointsResponse, SetExceptionBreakpointsResponse,
    SourceResponse, StackTraceResponse, ThreadsResponse, VariablesResponse,
};
use dap::server::Server;
use dap::types::{Breakpoint, Scope, Source, StackFrame, StoppedEventReason, Thread, Variable};
use fm::codespan_files::Files;
use fm::{FileId, FileMap};
use imbl::Vector;
use noirc_errors::Location;
use noirc_frontend::hir::comptime::ComptimeDebugger;
use noirc_frontend::node_interner::{FuncId, NodeInterner};

const LOCALS_SCOPE_REFERENCE: i64 = 1;

pub(crate) enum SteppingMode {
    Continue,
    StepIn,
    StepOver,
    StepOut,
}

pub(crate) struct ComptimeDapDebugger<'a, R: Read, W: Write> {
    server: &'a mut Server<R, W>,
    stepping_mode: SteppingMode,
    stop_depth: usize,
    breakpoints: HashMap<FileId, HashSet<usize>>,
    first_stop: bool,
    running: bool,
    /// Tracks the last location where we stopped, to avoid stopping twice
    /// on the same line during stepping (e.g. multiple statements on one line).
    last_stopped: Option<(FileId, usize)>,
}

impl<'a, R: Read, W: Write> ComptimeDapDebugger<'a, R, W> {
    pub(crate) fn new(
        server: &'a mut Server<R, W>,
        breakpoints: HashMap<FileId, HashSet<usize>>,
        stepping_mode: SteppingMode,
    ) -> Self {
        Self {
            server,
            stepping_mode,
            stop_depth: 0,
            breakpoints,
            first_stop: true,
            running: true,
            last_stopped: None,
        }
    }

    fn should_stop(&self, file: FileId, line: usize, call_depth: usize) -> bool {
        match self.stepping_mode {
            SteppingMode::StepIn => true,
            SteppingMode::StepOver => call_depth <= self.stop_depth,
            SteppingMode::StepOut => call_depth < self.stop_depth,
            SteppingMode::Continue => {
                self.breakpoints.get(&file).is_some_and(|lines| lines.contains(&line))
            }
        }
    }

    fn handle_stopped(
        &mut self,
        location: Location,
        interner: &NodeInterner,
        files: &FileMap,
        call_stack: &Vector<Location>,
        current_function: Option<FuncId>,
        call_stack_functions: &Vector<Option<FuncId>>,
        is_breakpoint: bool,
    ) {
        let reason = if self.first_stop {
            self.first_stop = false;
            StoppedEventReason::Entry
        } else if is_breakpoint {
            StoppedEventReason::Breakpoint
        } else {
            StoppedEventReason::Step
        };

        if let Err(e) = self.server.send_event(Event::Stopped(StoppedEventBody {
            reason,
            description: None,
            thread_id: Some(0),
            preserve_focus_hint: Some(false),
            text: None,
            all_threads_stopped: Some(false),
            hit_breakpoint_ids: None,
        })) {
            eprintln!("DAP error sending Stopped event: {e}");
            self.running = false;
            return;
        }

        self.dap_sub_loop(
            location,
            interner,
            files,
            call_stack,
            current_function,
            call_stack_functions,
        );
    }

    fn dap_sub_loop(
        &mut self,
        location: Location,
        interner: &NodeInterner,
        files: &FileMap,
        call_stack: &Vector<Location>,
        current_function: Option<FuncId>,
        call_stack_functions: &Vector<Option<FuncId>>,
    ) {
        loop {
            let req = match self.server.poll_request() {
                Ok(Some(req)) => req,
                Ok(None) => {
                    self.running = false;
                    break;
                }
                Err(e) => {
                    eprintln!("DAP error polling request: {e}");
                    self.running = false;
                    break;
                }
            };

            let result =
                match req.command {
                    Command::Threads => {
                        self.server.respond(req.success(ResponseBody::Threads(ThreadsResponse {
                            threads: vec![Thread { id: 0, name: "main".to_string() }],
                        })))
                    }
                    Command::StackTrace(_) => {
                        let frames = self.build_stack_frames(
                            location,
                            interner,
                            files,
                            call_stack,
                            current_function,
                            call_stack_functions,
                        );
                        let total = frames.len() as i64;
                        self.server.respond(req.success(ResponseBody::StackTrace(
                            StackTraceResponse { stack_frames: frames, total_frames: Some(total) },
                        )))
                    }
                    Command::Scopes(_) => {
                        self.server.respond(req.success(ResponseBody::Scopes(ScopesResponse {
                            scopes: vec![Scope {
                                name: "Locals".to_string(),
                                variables_reference: LOCALS_SCOPE_REFERENCE,
                                ..Scope::default()
                            }],
                        })))
                    }
                    Command::Variables(ref args) => {
                        let variables = if args.variables_reference == LOCALS_SCOPE_REFERENCE {
                            self.build_variables(interner, files)
                        } else {
                            vec![]
                        };
                        self.server.respond(
                            req.success(ResponseBody::Variables(VariablesResponse { variables })),
                        )
                    }
                    Command::SetBreakpoints(ref args) => {
                        let breakpoints = self.handle_set_breakpoints(args, files);
                        self.server.respond(req.success(ResponseBody::SetBreakpoints(
                            SetBreakpointsResponse { breakpoints },
                        )))
                    }
                    Command::Source(ref args) => {
                        let file_id = FileId::new(args.source_reference as usize - 1);
                        let content =
                            files.source(file_id).map(|s| s.to_string()).unwrap_or_default();
                        self.server.respond(req.success(ResponseBody::Source(SourceResponse {
                            content,
                            mime_type: None,
                        })))
                    }
                    Command::Continue(_) => {
                        self.stepping_mode = SteppingMode::Continue;
                        self.last_stopped = None;
                        self.respond_and_break(req);
                        break;
                    }
                    Command::StepIn(_) => {
                        self.stepping_mode = SteppingMode::StepIn;
                        self.last_stopped = None;
                        self.respond_and_break(req);
                        break;
                    }
                    Command::Next(_) => {
                        self.stepping_mode = SteppingMode::StepOver;
                        self.stop_depth = call_stack.len();
                        self.last_stopped = None;
                        self.respond_and_break(req);
                        break;
                    }
                    Command::StepOut(_) => {
                        self.stepping_mode = SteppingMode::StepOut;
                        self.stop_depth = call_stack.len();
                        self.last_stopped = None;
                        self.respond_and_break(req);
                        break;
                    }
                    Command::SetExceptionBreakpoints(_) => {
                        self.server.respond(req.success(ResponseBody::SetExceptionBreakpoints(
                            SetExceptionBreakpointsResponse { breakpoints: None },
                        )))
                    }
                    Command::ConfigurationDone => match req.ack() {
                        Ok(resp) => self.server.respond(resp),
                        Err(e) => {
                            eprintln!("DAP error: {e}");
                            Ok(())
                        }
                    },
                    Command::Disconnect(_) => {
                        let _ = req.ack().map(|r| self.server.respond(r));
                        self.running = false;
                        break;
                    }
                    _ => {
                        eprintln!("Unhandled DAP command in sub-loop: {:?}", req.command);
                        self.server.respond(req.error("Not supported"))
                    }
                };

            if let Err(e) = result {
                eprintln!("DAP error: {e}");
                self.running = false;
                break;
            }
        }
    }

    fn respond_and_break(&mut self, req: dap::requests::Request) {
        match req.ack() {
            Ok(resp) => {
                if let Err(e) = self.server.respond(resp) {
                    eprintln!("DAP error: {e}");
                    self.running = false;
                }
            }
            Err(e) => {
                eprintln!("DAP error: {e}");
                self.running = false;
            }
        }
    }

    fn build_stack_frames(
        &self,
        current_location: Location,
        interner: &NodeInterner,
        files: &FileMap,
        call_stack: &Vector<Location>,
        current_function: Option<FuncId>,
        call_stack_functions: &Vector<Option<FuncId>>,
    ) -> Vec<StackFrame> {
        let mut frames = Vec::new();

        // Top frame: current function
        let current_name = function_name(interner, current_function);
        frames.push(location_to_stack_frame(0, &current_name, current_location, files));

        // Upper frames from the call stack (most recent call site first).
        // call_stack_functions is parallel to call_stack: each entry is the
        // FuncId of the function that *contains* the corresponding call site.
        for (i, (&loc, &func_id)) in
            call_stack.iter().rev().zip(call_stack_functions.iter().rev()).enumerate()
        {
            let name = function_name(interner, func_id);
            frames.push(location_to_stack_frame((i + 1) as i64, &name, loc, files));
        }

        frames
    }

    fn build_variables(&self, interner: &NodeInterner, files: &FileMap) -> Vec<Variable> {
        interner
            .visible_comptime_variables()
            .into_iter()
            .map(|(id, value)| {
                let name = interner.definition_name(*id).to_string();
                let display_value = value.display(interner, files).to_string();
                let type_name = value.get_type().to_string();
                Variable {
                    name,
                    value: display_value,
                    type_field: Some(type_name),
                    ..Variable::default()
                }
            })
            .collect()
    }

    fn handle_set_breakpoints(
        &mut self,
        args: &dap::requests::SetBreakpointsArguments,
        files: &FileMap,
    ) -> Vec<Breakpoint> {
        let source_path = args.source.path.as_deref().unwrap_or("");
        let file_id = find_file_id(files, source_path);

        let Some(requested) = &args.breakpoints else {
            if let Some(fid) = file_id {
                self.breakpoints.remove(&fid);
            }
            return vec![];
        };

        if let Some(fid) = file_id {
            let lines: HashSet<usize> = requested.iter().map(|bp| bp.line as usize).collect();
            self.breakpoints.insert(fid, lines);

            requested
                .iter()
                .map(|bp| Breakpoint {
                    verified: true,
                    line: Some(bp.line),
                    ..Breakpoint::default()
                })
                .collect()
        } else {
            requested
                .iter()
                .map(|bp| Breakpoint {
                    verified: false,
                    message: Some(format!("File not found: {source_path}")),
                    line: Some(bp.line),
                    ..Breakpoint::default()
                })
                .collect()
        }
    }
}

impl<R: Read, W: Write> ComptimeDebugger for ComptimeDapDebugger<'_, R, W> {
    fn on_statement(
        &mut self,
        location: Location,
        interner: &NodeInterner,
        files: &FileMap,
        call_stack: &Vector<Location>,
        current_function: Option<FuncId>,
        call_stack_functions: &Vector<Option<FuncId>>,
    ) {
        if !self.running {
            return;
        }

        let Some((line, _column)) = location_to_line_column(files, location) else {
            return;
        };

        let is_breakpoint =
            self.breakpoints.get(&location.file).is_some_and(|lines| lines.contains(&line));

        let call_depth = call_stack.len();
        if self.should_stop(location.file, line, call_depth) {
            // Skip if we already stopped on this exact line (avoids double-stops
            // when multiple statements share a source line), unless it's a breakpoint.
            if !is_breakpoint && self.last_stopped == Some((location.file, line)) {
                return;
            }

            self.last_stopped = Some((location.file, line));
            self.handle_stopped(
                location,
                interner,
                files,
                call_stack,
                current_function,
                call_stack_functions,
                is_breakpoint,
            );
        }
    }
}

fn location_to_stack_frame(id: i64, name: &str, location: Location, files: &FileMap) -> StackFrame {
    let (line, column) = location_to_line_column(files, location).unwrap_or((1, 1));
    let source = files.get_absolute_name(location.file).ok().map(|file_name| {
        let path_buf = file_name.clone().into_path_buf();
        if path_buf.is_absolute() {
            Source { path: Some(file_name.to_string()), ..Source::default() }
        } else {
            // Stdlib and other embedded files: try to resolve to a real disk path
            // (works in debug builds where stdlib source is available on disk).
            // Falls back to source_reference for release builds.
            let disk_path = file_name.to_string().strip_prefix("std/").and_then(|rest| {
                let disk_root = noirc_driver::stdlib_disk_path()?;
                let resolved = disk_root.join(rest);
                resolved.is_file().then(|| resolved.to_string_lossy().to_string())
            });

            if let Some(abs_path) = disk_path {
                Source { path: Some(abs_path), ..Source::default() }
            } else {
                Source {
                    name: Some(file_name.to_string()),
                    path: Some(file_name.to_string()),
                    source_reference: Some(location.file.as_usize() as i32 + 1),
                    ..Source::default()
                }
            }
        }
    });

    StackFrame {
        id,
        name: name.to_string(),
        source,
        line: line as i64,
        column: column as i64,
        ..StackFrame::default()
    }
}

fn location_to_line_column(files: &FileMap, location: Location) -> Option<(usize, usize)> {
    let line = files.line_index(location.file, location.span.start() as usize).ok()?;
    let line_range = files.line_range(location.file, line).ok()?;
    let column = location.span.start() as usize - line_range.start + 1;
    Some((line + 1, column)) // Convert to 1-based
}

fn function_name(interner: &NodeInterner, func_id: Option<FuncId>) -> String {
    match func_id {
        Some(id) => interner.function_name(&id).to_string(),
        None => "<global>".to_string(),
    }
}

pub(crate) fn find_file_id(files: &FileMap, path: &str) -> Option<FileId> {
    let path = std::path::Path::new(path);
    for file_id in files.all_file_ids() {
        if let Ok(name) = files.get_absolute_name(*file_id) {
            if name.into_path_buf() == path {
                return Some(*file_id);
            }
        }
    }
    None
}
