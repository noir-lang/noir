use crate::DebugProject;
use crate::context::{
    DebugCommandResult, DebugContext, DebugExecutionResult, DebugLocation, DebugStackFrame,
    RunParams,
};

use crate::foreign_calls::DefaultDebugForeignCallExecutor;
use noirc_artifacts::debug::DebugArtifact;

use easy_repl::{CommandStatus, Repl, command};
use noirc_artifacts::program::CompiledProgram;
use std::cell::RefCell;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use acvm::brillig_vm::brillig::Opcode as BrilligOpcode;
use acvm::{
    AcirField, FieldElement,
    acir::{
        brillig::BitSize,
        circuit::{
            Circuit, Opcode, OpcodeLocation,
            brillig::{BrilligBytecode, BrilligFunctionId},
        },
        native_types::{Witness, WitnessMap},
    },
    brillig_vm::MemoryValue,
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use noirc_printable_type::PrintableValueDisplay;

use crate::{
    foreign_calls::DebugForeignCallExecutor, source_code_printer::print_source_code_location,
};

type Context<'a> = DebugContext<'a, Bn254BlackBoxSolver>;

#[derive(Debug, Clone)]
pub(super) enum DebugCommandAPI {
    AddBreakpoint(DebugLocation),
    AddBreakpointAtLine(i64),
    DeleteBreakpoint(DebugLocation),
    Restart,
    StepAcirOpcode,
    StepIntoOpcode,
    NextInto,
    NextOver,
    NextOut,
    Cont,
    UpdateWitness(u32, String),
    WriteBrilligMemory(usize, String, u32),
    ShowVariables,
    ShowWitnessMap,
    ShowWitness(u32),
    ShowBrilligMemory,
    ShowCurrentCallStack,
    ShowCurrentVmStatus,
    ShowOpcodes,
    Terminate,
}

#[derive(Debug)]
pub(super) enum DebuggerStatus {
    Idle,
    Busy,
    Final(DebugExecutionResult),
}

pub struct AsyncReplDebugger<'a> {
    circuits: Vec<Circuit<FieldElement>>,
    debug_artifact: &'a DebugArtifact,
    initial_witness: WitnessMap<FieldElement>,
    unconstrained_functions: Vec<BrilligBytecode<FieldElement>>,
    command_receiver: Receiver<DebugCommandAPI>,
    status_sender: Sender<DebuggerStatus>,
    last_result: DebugCommandResult,
    pedantic_solving: bool,
    raw_source_printing: bool,
}

impl<'a> AsyncReplDebugger<'a> {
    pub fn new(
        compiled_program: &CompiledProgram,
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap<FieldElement>,
        status_sender: Sender<DebuggerStatus>,
        command_receiver: Receiver<DebugCommandAPI>,
        raw_source_printing: bool,
        pedantic_solving: bool,
    ) -> Self {
        let last_result = DebugCommandResult::Ok;

        Self {
            command_receiver,
            status_sender,
            circuits: compiled_program.program.functions.clone(),
            debug_artifact,
            last_result,
            unconstrained_functions: compiled_program.program.unconstrained_functions.clone(),
            raw_source_printing,
            initial_witness,
            pedantic_solving,
        }
    }

    fn send_status(&mut self, status: DebuggerStatus) {
        self.status_sender.send(status).expect("Downstream channel closed");
    }

    pub(super) fn start_debugging(
        mut self,
        foreign_call_executor: Box<dyn DebugForeignCallExecutor + 'a>,
    ) {
        let blackbox_solver = &Bn254BlackBoxSolver(self.pedantic_solving);
        let circuits = &self.circuits.clone();
        let unconstrained_functions = &self.unconstrained_functions.clone();
        let mut context = DebugContext::new(
            blackbox_solver,
            circuits,
            self.debug_artifact,
            self.initial_witness.clone(),
            foreign_call_executor,
            unconstrained_functions,
        );

        if context.get_current_debug_location().is_none() {
            // handle circuit with no opcodes
            self.last_result = DebugCommandResult::Done;
        }

        println!("Debugger ready to receive messages..");
        loop {
            // recv blocks until it receives message
            if let Ok(received) = self.command_receiver.recv() {
                self.send_status(DebuggerStatus::Busy);
                match received {
                    DebugCommandAPI::AddBreakpoint(debug_location) => {
                        Self::add_breakpoint_at(&mut context, debug_location);
                    }
                    DebugCommandAPI::DeleteBreakpoint(debug_location) => {
                        Self::delete_breakpoint_at(&mut context, debug_location);
                    }
                    DebugCommandAPI::Restart => {
                        self.restart_session(&mut context);
                    }
                    DebugCommandAPI::WriteBrilligMemory(index, value, bit_size) => {
                        Self::write_brillig_memory(&mut context, index, value, bit_size);
                    }
                    DebugCommandAPI::UpdateWitness(index, value) => {
                        Self::update_witness(&mut context, index, value);
                    }
                    DebugCommandAPI::StepAcirOpcode => {
                        self.handle_step(&mut context, |context| context.step_acir_opcode());
                    }
                    DebugCommandAPI::StepIntoOpcode => {
                        self.handle_step(&mut context, |context| context.step_into_opcode());
                    }
                    DebugCommandAPI::NextInto => {
                        self.handle_step(&mut context, |context| context.next_into());
                    }
                    DebugCommandAPI::NextOver => {
                        self.handle_step(&mut context, |context| context.next_over());
                    }
                    DebugCommandAPI::NextOut => {
                        self.handle_step(&mut context, |context| context.next_out());
                    }
                    DebugCommandAPI::Cont => self.handle_step(&mut context, |context| {
                        println!("(Continuing execution...)");
                        context.cont()
                    }),
                    DebugCommandAPI::AddBreakpointAtLine(line_number) => {
                        Self::add_breakpoint_at_line(&mut context, line_number);
                    }
                    DebugCommandAPI::ShowVariables => {
                        Self::show_variables(&mut context);
                    }
                    DebugCommandAPI::ShowWitnessMap => {
                        Self::show_witness_map(&mut context);
                    }
                    DebugCommandAPI::ShowWitness(index) => {
                        Self::show_witness(&mut context, index);
                    }
                    DebugCommandAPI::ShowBrilligMemory => {
                        Self::show_brillig_memory(&mut context);
                    }
                    DebugCommandAPI::ShowCurrentCallStack => {
                        self.show_current_call_stack(&mut context);
                    }
                    DebugCommandAPI::ShowOpcodes => {
                        self.show_opcodes(&mut context);
                    }
                    DebugCommandAPI::ShowCurrentVmStatus => {
                        self.show_current_vm_status(&mut context);
                    }
                    DebugCommandAPI::Terminate => {
                        self.terminate(context);
                        break;
                    }
                };
            } else {
                println!("Upstream channel closed. Terminating debugger");
                break;
            }
            self.send_status(DebuggerStatus::Idle);
        }
    }

    fn show_current_vm_status(&self, context: &mut Context<'_>) {
        let location = context.get_current_debug_location();

        match location {
            None => println!("Finished execution"),
            Some(location) => {
                let circuit_id = location.circuit_id;
                let opcodes = context.get_opcodes_of_circuit(circuit_id);

                match &location.opcode_location {
                    OpcodeLocation::Acir(ip) => {
                        println!("At opcode {} :: {}", location, opcodes[*ip]);
                    }
                    OpcodeLocation::Brillig { acir_index, brillig_index } => {
                        let brillig_bytecode =
                            if let Opcode::BrilligCall { id, .. } = opcodes[*acir_index] {
                                &self.unconstrained_functions[id.as_usize()].bytecode
                            } else {
                                unreachable!("Brillig location does not contain Brillig opcodes");
                            };
                        println!(
                            "At opcode {} :: {:?}",
                            location, brillig_bytecode[*brillig_index]
                        );
                    }
                }
                let locations = context.get_source_location_for_debug_location(&location);

                print_source_code_location(
                    self.debug_artifact,
                    &locations,
                    self.raw_source_printing,
                );
            }
        }
    }

    fn show_stack_frame(
        &self,
        context: &mut Context<'_>,
        index: usize,
        debug_location: &DebugLocation,
    ) {
        let opcodes = context.get_opcodes();
        match &debug_location.opcode_location {
            OpcodeLocation::Acir(instruction_pointer) => {
                println!(
                    "Frame #{index}, opcode {} :: {}",
                    debug_location, opcodes[*instruction_pointer]
                );
            }
            OpcodeLocation::Brillig { acir_index, brillig_index } => {
                let brillig_bytecode = if let Opcode::BrilligCall { id, .. } = opcodes[*acir_index]
                {
                    &self.unconstrained_functions[id.as_usize()].bytecode
                } else {
                    unreachable!("Brillig location does not contain Brillig opcodes");
                };
                println!(
                    "Frame #{index}, opcode {} :: {:?}",
                    debug_location, brillig_bytecode[*brillig_index]
                );
            }
        }
        let locations = context.get_source_location_for_debug_location(debug_location);
        print_source_code_location(self.debug_artifact, &locations, self.raw_source_printing);
    }

    fn show_current_call_stack(&mut self, context: &mut Context<'_>) {
        let call_stack = context.get_call_stack();

        if call_stack.is_empty() {
            println!("Finished execution. Call stack empty.");
            return;
        }

        for (i, frame_location) in call_stack.iter().enumerate() {
            self.show_stack_frame(context, i, frame_location);
        }
    }

    fn show_opcodes(&mut self, context: &mut Context<'_>) {
        for i in 0..self.circuits.len() {
            self.show_opcodes_of_circuit(context, i as u32);
        }
    }

    fn show_opcodes_of_circuit(&mut self, context: &mut Context<'_>, circuit_id: u32) {
        let current_opcode_location =
            context.get_current_debug_location().and_then(|debug_location| {
                if debug_location.circuit_id == circuit_id {
                    Some(debug_location.opcode_location)
                } else {
                    None
                }
            });
        let opcodes = context.get_opcodes_of_circuit(circuit_id);
        let current_acir_index = match current_opcode_location {
            Some(OpcodeLocation::Acir(ip)) => Some(ip),
            Some(OpcodeLocation::Brillig { acir_index, .. }) => Some(acir_index),
            None => None,
        };
        let current_brillig_index = match current_opcode_location {
            Some(OpcodeLocation::Brillig { brillig_index, .. }) => brillig_index,
            _ => 0,
        };
        let outer_marker = |acir_index| {
            if current_acir_index == Some(acir_index) {
                "->"
            } else if context.is_breakpoint_set(&DebugLocation {
                circuit_id,
                opcode_location: OpcodeLocation::Acir(acir_index),
                brillig_function_id: None,
            }) {
                " *"
            } else {
                ""
            }
        };
        let brillig_marker = |acir_index, brillig_index, brillig_function_id| {
            if current_acir_index == Some(acir_index) && brillig_index == current_brillig_index {
                "->"
            } else if context.is_breakpoint_set(&DebugLocation {
                circuit_id,
                opcode_location: OpcodeLocation::Brillig { acir_index, brillig_index },
                brillig_function_id: Some(brillig_function_id),
            }) {
                " *"
            } else {
                ""
            }
        };
        let print_brillig_bytecode =
            |acir_index,
             bytecode: &[BrilligOpcode<FieldElement>],
             brillig_function_id: BrilligFunctionId| {
                for (brillig_index, brillig_opcode) in bytecode.iter().enumerate() {
                    println!(
                        "{:>2}:{:>3}.{:<2} |{:2} {:?}",
                        circuit_id,
                        acir_index,
                        brillig_index,
                        brillig_marker(acir_index, brillig_index, brillig_function_id),
                        brillig_opcode
                    );
                }
            };
        for (acir_index, opcode) in opcodes.iter().enumerate() {
            let marker = outer_marker(acir_index);
            match &opcode {
                Opcode::BrilligCall { id, inputs, outputs, .. } => {
                    println!(
                        "{circuit_id:>2}:{acir_index:>3} {marker:2} BRILLIG CALL id={id} inputs={inputs:?}"
                    );
                    println!("          |       outputs={outputs:?}");
                    let bytecode = &self.unconstrained_functions[id.as_usize()].bytecode;
                    print_brillig_bytecode(acir_index, bytecode, *id);
                }
                _ => println!("{circuit_id:>2}:{acir_index:>3} {marker:2} {opcode:?}"),
            }
        }
    }

    fn add_breakpoint_at(context: &mut Context<'_>, location: DebugLocation) {
        if !context.is_valid_debug_location(&location) {
            println!("Invalid location {location}");
        } else if context.add_breakpoint(location) {
            println!("Added breakpoint at {location}");
        } else {
            println!("Breakpoint at {location} already set");
        }
    }

    fn add_breakpoint_at_line(context: &mut Context<'_>, line_number: i64) {
        let best_location = context.find_opcode_at_current_file_line(line_number);
        match best_location {
            Some(location) => {
                println!("Added breakpoint at line {line_number}");
                Self::add_breakpoint_at(context, location);
            }
            None => println!("No opcode at line {line_number}"),
        }
    }

    fn delete_breakpoint_at(context: &mut Context<'_>, location: DebugLocation) {
        if context.delete_breakpoint(&location) {
            println!("Breakpoint at {location} deleted");
        } else {
            println!("Breakpoint at {location} not set");
        }
    }

    fn handle_result(&mut self, result: DebugCommandResult) {
        self.last_result = result;
        match &self.last_result {
            DebugCommandResult::Done => {
                println!("Execution finished");
            }
            DebugCommandResult::Ok => (),
            DebugCommandResult::BreakpointReached(location) => {
                println!("Stopped at breakpoint in opcode {location}");
            }
            DebugCommandResult::Error(error) => {
                println!("ERROR: {error}");
            }
        }
    }

    fn handle_step<F>(&mut self, context: &mut Context<'_>, step: F)
    where
        F: Fn(&mut Context) -> DebugCommandResult,
    {
        let should_execute = match self.last_result {
            DebugCommandResult::Ok | DebugCommandResult::BreakpointReached(..) => true,
            DebugCommandResult::Done => {
                println!("Execution finished");
                false
            }
            DebugCommandResult::Error(ref error) => {
                println!("ERROR: {error}");
                self.show_current_vm_status(context);
                false
            }
        };
        if should_execute {
            let result = step(context);
            self.show_current_vm_status(context);
            self.handle_result(result);
        }
    }

    fn restart_session(&mut self, context: &mut Context<'_>) {
        context.restart();
        self.last_result = DebugCommandResult::Ok;
        println!("Restarted debugging session.");
        self.show_current_vm_status(context);
    }

    fn show_witness_map(context: &mut Context<'_>) {
        let witness_map = context.get_witness_map();
        // NOTE: we need to clone() here to get the iterator
        for (witness, value) in witness_map.clone().into_iter() {
            println!("_{} = {value}", witness.witness_index());
        }
    }

    fn show_witness(context: &mut Context<'_>, index: u32) {
        if let Some(value) = context.get_witness_map().get_index(index) {
            println!("_{index} = {value}");
        }
    }

    fn update_witness(context: &mut Context<'_>, index: u32, value: String) {
        let Some(field_value) = FieldElement::try_from_str(&value) else {
            println!("Invalid witness value: {value}");
            return;
        };

        let witness = Witness::from(index);
        _ = context.overwrite_witness(witness, field_value);
        println!("_{index} = {value}");
    }

    fn show_brillig_memory(context: &mut Context<'_>) {
        if !context.is_executing_brillig() {
            println!("Not executing a Brillig block");
            return;
        }

        let Some(memory) = context.get_brillig_memory() else {
            // this can happen when just entering the Brillig block since ACVM
            // would have not initialized the Brillig VM yet; in fact, the
            // Brillig code may be skipped altogether
            println!("Brillig VM memory not available");
            return;
        };

        for (index, value) in memory.iter().enumerate() {
            // Zero field is the default value, we omit it when printing memory
            if let MemoryValue::Field(field) = value {
                if field == &FieldElement::zero() {
                    continue;
                }
            }
            println!("{index} = {value}");
        }
    }
    fn write_brillig_memory(context: &mut Context<'_>, index: usize, value: String, bit_size: u32) {
        let Some(field_value) = FieldElement::try_from_str(&value) else {
            println!("Invalid value: {value}");
            return;
        };

        let Ok(bit_size) = BitSize::try_from_u32::<FieldElement>(bit_size) else {
            println!("Invalid bit size: {bit_size}");
            return;
        };

        if !context.is_executing_brillig() {
            println!("Not executing a Brillig block");
            return;
        }
        context.write_brillig_memory(index, field_value, bit_size);
    }

    fn show_variables(context: &mut Context<'_>) {
        let variables: Vec<DebugStackFrame<FieldElement>> =
            context.get_variables().iter().map(DebugStackFrame::from).collect();
        for frame in variables {
            println!("{}({})", frame.function_name, frame.function_params.join(", "));
            for (var_name, value, var_type) in frame.variables.iter() {
                let printable_value =
                    PrintableValueDisplay::Plain((*value).clone(), (*var_type).clone());
                println!("  {var_name}:{var_type:?} = {printable_value}");
            }
        }
    }

    fn terminate(self, context: Context<'_>) {
        let result = if context.is_solved() {
            let solved_witness_stack = context.finalize();
            DebugExecutionResult::Solved(solved_witness_stack)
        } else {
            match self.last_result {
                // Expose the last known error
                DebugCommandResult::Error(error) => DebugExecutionResult::Error(error),
                _ => DebugExecutionResult::Incomplete,
            }
        };
        self.status_sender.send(DebuggerStatus::Final(result)).expect("Downstream channel closed");
    }
}

struct DebugController {
    command_sender: Sender<DebugCommandAPI>,
    status_receiver: Receiver<DebuggerStatus>,
}
impl DebugController {
    fn debugger_status(&self) -> DebuggerStatus {
        self.status_receiver.recv().expect("Debugger closed connection unexpectedly")
    }

    fn call_debugger(&self, command: DebugCommandAPI) {
        self.call_debugger_no_wait_for_idle(command);
        self.wait_for_idle();
    }

    fn call_debugger_no_wait_for_idle(&self, command: DebugCommandAPI) {
        self.command_sender.send(command).expect("Could not communicate with debugger");
    }

    fn get_final_result(&self) -> DebugExecutionResult {
        loop {
            let status = self.debugger_status();
            if let DebuggerStatus::Final(result) = status {
                return result;
            }
        }
    }

    fn wait_for_idle(&self) {
        loop {
            let status = self.debugger_status();
            if let DebuggerStatus::Idle = status {
                break;
            };
        }
    }

    pub fn step_acir_opcode(&self) {
        self.call_debugger(DebugCommandAPI::StepAcirOpcode);
    }
    pub fn cont(&self) {
        self.call_debugger(DebugCommandAPI::Cont);
    }
    pub fn step_into_opcode(&self) {
        self.call_debugger(DebugCommandAPI::StepIntoOpcode);
    }
    pub fn next_into(&self) {
        self.call_debugger(DebugCommandAPI::NextInto);
    }
    pub fn next_over(&self) {
        self.call_debugger(DebugCommandAPI::NextOver);
    }
    pub fn next_out(&self) {
        self.call_debugger(DebugCommandAPI::NextOut);
    }
    pub fn restart_session(&self) {
        self.call_debugger(DebugCommandAPI::Restart);
    }
    pub fn add_breakpoint_at_line(&self, line_number: i64) {
        self.call_debugger(DebugCommandAPI::AddBreakpointAtLine(line_number));
    }
    pub fn add_breakpoint_at(&self, location: DebugLocation) {
        self.call_debugger(DebugCommandAPI::AddBreakpoint(location));
    }
    pub fn delete_breakpoint_at(&self, location: DebugLocation) {
        self.call_debugger(DebugCommandAPI::DeleteBreakpoint(location));
    }
    pub fn update_witness(&self, index: u32, value: String) {
        self.call_debugger(DebugCommandAPI::UpdateWitness(index, value));
    }
    pub fn write_brillig_memory(&self, index: usize, value: String, bit_size: u32) {
        self.call_debugger(DebugCommandAPI::WriteBrilligMemory(index, value, bit_size));
    }
    pub fn show_vars(&self) {
        self.call_debugger(DebugCommandAPI::ShowVariables);
    }
    pub fn show_opcodes(&self) {
        self.call_debugger(DebugCommandAPI::ShowOpcodes);
    }
    pub fn show_witness_map(&self) {
        self.call_debugger(DebugCommandAPI::ShowWitnessMap);
    }
    pub fn show_witness(&self, index: u32) {
        self.call_debugger(DebugCommandAPI::ShowWitness(index));
    }
    pub fn show_brillig_memory(&self) {
        self.call_debugger(DebugCommandAPI::ShowBrilligMemory);
    }
    pub fn show_current_call_stack(&self) {
        self.call_debugger(DebugCommandAPI::ShowCurrentCallStack);
    }
    pub fn show_current_vm_status(&self) {
        self.call_debugger(DebugCommandAPI::ShowCurrentVmStatus);
    }
    pub fn terminate(&self) {
        self.call_debugger_no_wait_for_idle(DebugCommandAPI::Terminate);
    }
}

pub fn run(project: DebugProject, run_params: RunParams) -> DebugExecutionResult {
    let debug_artifact = DebugArtifact {
        debug_symbols: project.compiled_program.debug.clone(),
        file_map: project.compiled_program.file_map.clone(),
    };

    let foreign_call_executor = Box::new(DefaultDebugForeignCallExecutor::from_artifact(
        std::io::stdout(),
        run_params.oracle_resolver_url,
        &debug_artifact,
        Some(project.root_dir),
        project.package_name,
    ));

    let (command_tx, command_rx) = mpsc::channel::<DebugCommandAPI>();
    let (status_tx, status_rx) = mpsc::channel::<DebuggerStatus>();
    thread::spawn(move || {
        let debugger = AsyncReplDebugger::new(
            &project.compiled_program,
            &debug_artifact,
            project.initial_witness,
            status_tx,
            command_rx,
            run_params.raw_source_printing.unwrap_or(false),
            run_params.pedantic_solving,
        );
        debugger.start_debugging(foreign_call_executor);
    });

    let context =
        RefCell::new(DebugController { command_sender: command_tx, status_receiver: status_rx });
    let ref_context = &context;

    ref_context.borrow().show_current_vm_status();

    let mut repl = Repl::builder()
        .add(
            "step",
            command! {
                "step to the next ACIR opcode",
                () => || {
                    ref_context.borrow_mut().step_acir_opcode();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "into",
            command! {
                "step into to the next opcode",
                () => || {
                    ref_context.borrow_mut().step_into_opcode();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "next",
            command! {
                "step until a new source location is reached",
                () => || {
                    ref_context.borrow_mut().next_into();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "over",
            command! {
                "step until a new source location is reached without diving into function calls",
                () => || {
                    ref_context.borrow_mut().next_over();
                    Ok(CommandStatus::Done)
                }
            }
        )
        .add(
            "out",
            command! {
                "step until a new source location is reached and the current stack frame is finished",
                () => || {
                    ref_context.borrow_mut().next_out();
                    Ok(CommandStatus::Done)
                }
            }
        )
        .add(
            "continue",
            command! {
                "continue execution until the end of the program",
                () => || {
                    ref_context.borrow_mut().cont();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "restart",
            command! {
                "restart the debugging session",
                () => || {
                    ref_context.borrow_mut().restart_session();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "opcodes",
            command! {
                "display ACIR opcodes",
                () => || {
                    ref_context.borrow().show_opcodes();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "break",
            command! {
                "add a breakpoint at a line of the current file",
                (line_number: i64) => |line_number| {
                    ref_context.borrow_mut().add_breakpoint_at_line(line_number);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "break",
            command! {
                "add a breakpoint at an opcode location",
                (LOCATION:DebugLocation) => |location| {
                    ref_context.borrow_mut().add_breakpoint_at(location);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "delete",
            command! {
                "delete breakpoint at an opcode location",
                (LOCATION:DebugLocation) => |location| {
                    ref_context.borrow_mut().delete_breakpoint_at(location);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "witness",
            command! {
                "show witness map",
                () => || {
                    ref_context.borrow().show_witness_map();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "witness",
            command! {
                "display a single witness from the witness map",
                (index: u32) => |index| {
                    ref_context.borrow().show_witness(index);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "witness",
            command! {
                "update a witness with the given value",
                (index: u32, value: String) => |index, value| {
                    ref_context.borrow_mut().update_witness(index, value);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "memory",
            command! {
                "show Brillig memory (valid when executing a Brillig block)",
                () => || {
                    ref_context.borrow().show_brillig_memory();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "memset",
            command! {
                "update a Brillig memory cell with the given value",
                (index: usize, value: String, bit_size: u32) => |index, value, bit_size| {
                    ref_context.borrow_mut().write_brillig_memory(index, value, bit_size);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "stacktrace",
            command! {
                "display the current stack trace",
                () => || {
                    ref_context.borrow().show_current_call_stack();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "vars",
            command! {
                "show variables for each function scope available at this point in execution",
                () => || {
                    ref_context.borrow_mut().show_vars();
                    Ok(CommandStatus::Done)
                }
            },
        )
        .build()
        .expect("Failed to initialize debugger repl");

    repl.run().expect("Debugger error");
    // REPL execution has finished.
    // Drop it so that we can move fields out from `context` again.
    drop(repl);

    context.borrow().terminate();
    context.borrow().get_final_result()
}
