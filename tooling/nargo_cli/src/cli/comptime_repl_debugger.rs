use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

use fm::codespan_files::Files;
use fm::{FileId, FileMap};
use imbl::Vector;
use noirc_errors::Location;
use noirc_frontend::hir::comptime::ComptimeDebugger;
use noirc_frontend::node_interner::{FuncId, NodeInterner};

use super::comptime_debugger::SteppingMode;

const CONTEXT_LINES: usize = 3;

pub(crate) struct ComptimeReplDebugger {
    stepping_mode: SteppingMode,
    stop_depth: usize,
    breakpoints: HashMap<FileId, HashSet<usize>>,
    first_stop: bool,
    running: bool,
    last_stopped: Option<(FileId, usize)>,
}

impl ComptimeReplDebugger {
    pub(crate) fn new() -> Self {
        Self {
            stepping_mode: SteppingMode::StepIn,
            stop_depth: 0,
            breakpoints: HashMap::new(),
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

    fn print_location(
        &self,
        location: Location,
        files: &FileMap,
        line: usize,
        interner: &NodeInterner,
        current_function: Option<FuncId>,
    ) {
        let file_name =
            files.get_absolute_name(location.file).map(|n| n.to_string()).unwrap_or_default();
        let func_name = function_name(interner, current_function);

        println!("At {file_name}:{line} in {func_name}");

        let Ok(source) = files.source(location.file) else {
            return;
        };

        let lines: Vec<&str> = source.lines().collect();
        let start = line.saturating_sub(CONTEXT_LINES + 1);
        let end = (line + CONTEXT_LINES).min(lines.len());

        for i in start..end {
            let line_num = i + 1;
            let marker = if line_num == line { "->" } else { "  " };
            println!("{marker} {line_num:>4} | {}", lines[i]);
        }
    }

    fn print_variables(&self, interner: &NodeInterner, files: &FileMap) {
        let vars: Vec<_> = interner.visible_comptime_variables().collect();
        if vars.is_empty() {
            println!("  (no variables)");
            return;
        }
        for (id, value) in vars {
            let name = interner.definition_name(*id);
            let display_value = value.display(interner, files);
            let type_name = value.get_type();
            println!("  {name}: {type_name} = {display_value}");
        }
    }

    fn print_stack_trace(
        &self,
        location: Location,
        interner: &NodeInterner,
        files: &FileMap,
        call_stack: &Vector<Location>,
        current_function: Option<FuncId>,
        call_stack_functions: &Vector<Option<FuncId>>,
    ) {
        let current_name = function_name(interner, current_function);
        let (line, col) = location_to_line_column(files, location).unwrap_or((1, 1));
        let file_name =
            files.get_absolute_name(location.file).map(|n| n.to_string()).unwrap_or_default();
        println!("#0  {current_name} at {file_name}:{line}:{col}");

        for (i, (&loc, &func_id)) in
            call_stack.iter().rev().zip(call_stack_functions.iter().rev()).enumerate()
        {
            let name = function_name(interner, func_id);
            let (line, col) = location_to_line_column(files, loc).unwrap_or((1, 1));
            let file_name =
                files.get_absolute_name(loc.file).map(|n| n.to_string()).unwrap_or_default();
            println!("#{:<2} {name} at {file_name}:{line}:{col}", i + 1);
        }
    }

    fn repl_loop(
        &mut self,
        location: Location,
        interner: &NodeInterner,
        files: &FileMap,
        call_stack: &Vector<Location>,
        current_function: Option<FuncId>,
        call_stack_functions: &Vector<Option<FuncId>>,
    ) {
        loop {
            print!("debug> ");
            let _ = io::stdout().flush();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() || input.is_empty() {
                self.running = false;
                break;
            }

            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            let cmd = parts.first().map(|s| *s).unwrap_or("");

            match cmd {
                "s" | "step" => {
                    self.stepping_mode = SteppingMode::StepIn;
                    self.last_stopped = None;
                    break;
                }
                "n" | "next" => {
                    self.stepping_mode = SteppingMode::StepOver;
                    self.stop_depth = call_stack.len();
                    self.last_stopped = None;
                    break;
                }
                "o" | "out" => {
                    self.stepping_mode = SteppingMode::StepOut;
                    self.stop_depth = call_stack.len();
                    self.last_stopped = None;
                    break;
                }
                "c" | "continue" => {
                    self.stepping_mode = SteppingMode::Continue;
                    self.last_stopped = None;
                    break;
                }
                "v" | "vars" => {
                    self.print_variables(interner, files);
                }
                "bt" | "stacktrace" => {
                    self.print_stack_trace(
                        location,
                        interner,
                        files,
                        call_stack,
                        current_function,
                        call_stack_functions,
                    );
                }
                "b" | "break" => {
                    if let Some(line_str) = parts.get(1) {
                        if let Ok(line_num) = line_str.parse::<usize>() {
                            self.breakpoints.entry(location.file).or_default().insert(line_num);
                            let file_name = files
                                .get_absolute_name(location.file)
                                .map(|n| n.to_string())
                                .unwrap_or_default();
                            println!("Breakpoint set at {file_name}:{line_num}");
                        } else {
                            println!("Usage: break <line_number>");
                        }
                    } else {
                        println!("Usage: break <line_number>");
                    }
                }
                "q" | "quit" => {
                    self.running = false;
                    break;
                }
                "h" | "help" | "" => {
                    println!("Commands:");
                    println!("  s, step       Step into next statement");
                    println!("  n, next       Step over (skip function calls)");
                    println!("  o, out        Step out of current function");
                    println!("  c, continue   Continue until breakpoint");
                    println!("  v, vars       Show local variables");
                    println!("  bt, stacktrace  Show call stack");
                    println!("  b, break <N>  Set breakpoint at line N");
                    println!("  q, quit       Stop debugging");
                    println!("  h, help       Show this help");
                }
                _ => {
                    println!("Unknown command: {cmd}. Type 'help' for commands.");
                }
            }
        }
    }
}

impl ComptimeDebugger for ComptimeReplDebugger {
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
            if !is_breakpoint && self.last_stopped == Some((location.file, line)) {
                return;
            }

            self.last_stopped = Some((location.file, line));

            if self.first_stop {
                self.first_stop = false;
                println!("Debugger started. Type 'help' for commands.");
            } else if is_breakpoint {
                println!("Breakpoint hit.");
            }

            self.print_location(location, files, line, interner, current_function);
            self.repl_loop(
                location,
                interner,
                files,
                call_stack,
                current_function,
                call_stack_functions,
            );
        }
    }
}

fn location_to_line_column(files: &FileMap, location: Location) -> Option<(usize, usize)> {
    let line = files.line_index(location.file, location.span.start() as usize).ok()?;
    let line_range = files.line_range(location.file, line).ok()?;
    let column = location.span.start() as usize - line_range.start + 1;
    Some((line + 1, column))
}

fn function_name(interner: &NodeInterner, func_id: Option<FuncId>) -> String {
    match func_id {
        Some(id) => interner.function_name(&id).to_string(),
        None => "<global>".to_string(),
    }
}
