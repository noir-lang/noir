//! This module is an abstraction layer over `Brillig`
//! To allow for separation of concerns, it knows nothing
//! about SSA types, and can therefore be tested independently.
//! `brillig_gen` is therefore the module which combines both
//! ssa types and types in this module.
//! A similar paradigm can be seen with the `acir_ir` module.
pub(crate) mod artifact;

use std::collections::HashMap;

use crate::ssa_refactor::ir::function::FunctionId;

use self::artifact::{BrilligArtifact, UnresolvedJumpLocation};
use acvm::{
    acir::brillig_vm::{
        BinaryFieldOp, BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, RegisterValueOrArray,
        Value,
    },
    FieldElement,
};

// Registers reserved for special purpose by BrilligGen when generating the bytecode
pub(crate) enum SpecialRegisters {
    // The index into the StackFrame, it is incremented for every nested call
    CallDepth = 0,
    /// Represent address where free memory is available for allocation
    Alloc = 1,
    /// Contains the address of an array which hold stack-frame data: the address of the memory where the registers are saved before a call
    /// The stack-frame has a fixed length, meaning the number nested function calls is limited to this length
    StackFrame = 2,
    /// Number of special registers
    Len = 3,
}

impl SpecialRegisters {
    // TODO: doc
    pub(crate) fn len() -> usize {
        SpecialRegisters::Len as usize
    }

    // TODO: doc
    pub(crate) fn stack_frame() -> RegisterIndex {
        RegisterIndex::from(SpecialRegisters::StackFrame as usize)
    }
    // TODO: doc
    pub(crate) fn call_depth() -> RegisterIndex {
        RegisterIndex::from(SpecialRegisters::CallDepth as usize)
    }
    // TODO: doc
    pub(crate) fn alloc() -> RegisterIndex {
        RegisterIndex::from(SpecialRegisters::Alloc as usize)
    }
}
/// Integer arithmetic in Brillig is limited to 127 bit
/// integers.
///
/// We could lift this in the future and have Brillig
/// do big integer arithmetic when it exceeds the field size
/// or we could have users re-implement big integer arithmetic
/// in Brillig.
/// Since constrained functions do not have this property, it
/// would mean that unconstrained functions will differ from
/// constrained functions in terms of syntax compatibility.
const BRILLIG_INTEGER_ARITHMETIC_BIT_SIZE: u32 = 127;

/// Brillig context object that is used while constructing the
/// Brillig bytecode.
pub(crate) struct BrilligContext {
    obj: BrilligArtifact,
    /// A usize indicating the latest un-used register.
    latest_register: usize,
    /// Context label, must be unique with respect to the function
    /// being linked.
    context_label: String,
    /// Section label, used to separate sections of code
    section_label: usize,
    ///
    function_labels: HashMap<FunctionId, String>,
}

impl BrilligContext {
    /// Adds a brillig instruction to the brillig byte code
    pub(crate) fn push_opcode(&mut self, opcode: BrilligOpcode) {
        self.obj.push_opcode(opcode);
    }

    /// Returns the artifact
    pub(crate) fn artifact(self) -> BrilligArtifact {
        self.obj
    }

    pub(crate) fn new(function_labels: HashMap<FunctionId, String>) -> BrilligContext {
        BrilligContext {
            obj: BrilligArtifact::new(),
            latest_register: SpecialRegisters::Len as usize,
            context_label: String::new(),
            section_label: 0,
            function_labels,
        }
    }

    // pub(crate) fn block_label(&self, block_id: BasicBlockId) -> String {
    //     self.obj.block_label(block_id)
    // }

    /// Allocates an array of size `size` and stores the pointer to the array
    /// in `pointer_register`
    pub(crate) fn allocate_array(&mut self, pointer_register: RegisterIndex, size: u32) {
        let len = self.create_register();
        self.const_instruction(len, Value::from(size as usize));

        self.push_opcode(BrilligOpcode::Mov {
            destination: pointer_register,
            source: SpecialRegisters::alloc(),
        });
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            destination: SpecialRegisters::alloc(),
            op: BinaryIntOp::Add,
            bit_size: 32,
            lhs: SpecialRegisters::alloc(),
            rhs: len,
        });
    }

    /// Adds a label to the next opcode
    pub(crate) fn enter_context<T: ToString>(&mut self, label: T) {
        self.context_label = label.to_string();
        self.section_label = 0;
        // Add a context label to the next opcode
        self.obj.add_label_at_position(label.to_string(), self.obj.index_of_next_opcode());
        // Add a section label to the next opcode
        self.obj
            .add_label_at_position(self.current_section_label(), self.obj.index_of_next_opcode());
    }

    /// Increments the section label and adds a section label to the next opcode
    fn enter_next_section(&mut self) {
        self.section_label += 1;
        self.obj
            .add_label_at_position(self.current_section_label(), self.obj.index_of_next_opcode());
    }

    /// Internal function used to compute the section labels
    fn compute_section_label(&self, section: usize) -> String {
        format!("{}-{}", self.context_label, section)
    }

    /// Returns the next section label
    fn next_section_label(&self) -> String {
        self.compute_section_label(self.section_label + 1)
    }

    /// Returns the current section label
    fn current_section_label(&self) -> String {
        self.compute_section_label(self.section_label)
    }

    pub(crate) fn function_block_label(&self, func_id: &FunctionId) -> String {
        self.function_labels[func_id].clone()
    }

    /// Adds a unresolved `Jump` instruction to the bytecode.
    pub(crate) fn jump_instruction<T: ToString>(&mut self, target_label: T) {
        self.add_unresolved_jump(BrilligOpcode::Jump { location: 0 }, target_label.to_string());
    }

    /// Adds a unresolved `JumpIf` instruction to the bytecode.
    pub(crate) fn jump_if_instruction<T: ToString>(
        &mut self,
        condition: RegisterIndex,
        target_label: T,
    ) {
        self.add_unresolved_jump(
            BrilligOpcode::JumpIf { condition, location: 0 },
            target_label.to_string(),
        );
    }

    /// Adds a unresolved `Jump` instruction to the bytecode.
    fn add_unresolved_jump(
        &mut self,
        jmp_instruction: BrilligOpcode,
        destination: UnresolvedJumpLocation,
    ) {
        self.obj.add_unresolved_jump(jmp_instruction, destination);
    }

    /// Adds a unresolved `Call` instruction to the bytecode.
    fn add_unresolved_call(&mut self, destination: UnresolvedJumpLocation, func: FunctionId) {
        self.obj.add_unresolved_call(BrilligOpcode::Call { location: 0 }, destination, func);
    }

    /// Creates a new register.
    pub(crate) fn create_register(&mut self) -> RegisterIndex {
        let register = RegisterIndex::from(self.latest_register);
        self.latest_register += 1;
        register
    }
}

impl BrilligContext {
    /// Emits brillig bytecode to jump to a trap condition if `condition`
    /// is false.
    pub(crate) fn constrain_instruction(&mut self, condition: RegisterIndex) {
        self.add_unresolved_jump(
            BrilligOpcode::JumpIf { condition, location: 0 },
            self.next_section_label(),
        );
        self.push_opcode(BrilligOpcode::Trap);
        self.enter_next_section();
    }

    /// Processes a return instruction.
    ///
    /// For Brillig, the return is implicit, since there is no explicit return instruction.
    /// The caller will take `N` values from the Register starting at register index 0.
    /// `N` indicates the number of return values expected.
    ///
    /// Brillig does not have an explicit return instruction, so this
    /// method will move all register values to the first `N` values in
    /// the VM.
    pub(crate) fn return_instruction(&mut self, return_registers: &[RegisterIndex]) {
        for (destination_index, return_register) in return_registers.iter().enumerate() {
            // If the destination register index is more than the latest register,
            // we update the latest register to be the destination register because the
            // brillig vm will expand the number of registers internally, when it encounters
            // a register that has not been initialized.
            if destination_index > self.latest_register {
                self.latest_register = destination_index;
            }
            self.mov_instruction(
                (destination_index + SpecialRegisters::len()).into(),
                *return_register,
            );
        }
        self.stop_instruction();
    }

    /// Emits a `mov` instruction.
    ///
    /// Copies the value at `source` into `destination`
    pub(crate) fn mov_instruction(&mut self, destination: RegisterIndex, source: RegisterIndex) {
        self.push_opcode(BrilligOpcode::Mov { destination, source });
    }

    /// Processes a binary instruction according `operation`.
    ///
    /// This method will compute lhs <operation> rhs
    /// and store the result in the `result` register.
    pub(crate) fn binary_instruction(
        &mut self,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
        result: RegisterIndex,
        operation: BrilligBinaryOp,
    ) {
        match operation {
            BrilligBinaryOp::Field { op } => {
                let opcode = BrilligOpcode::BinaryFieldOp { op, destination: result, lhs, rhs };
                self.push_opcode(opcode);
            }
            BrilligBinaryOp::Integer { op, bit_size } => {
                let opcode =
                    BrilligOpcode::BinaryIntOp { op, destination: result, bit_size, lhs, rhs };
                self.push_opcode(opcode);
            }
            BrilligBinaryOp::Modulo { is_signed_integer, bit_size } => {
                self.modulo_instruction(result, lhs, rhs, bit_size, is_signed_integer);
            }
        }
    }

    /// Stores the value of `constant` in the `result` register
    pub(crate) fn const_instruction(&mut self, result: RegisterIndex, constant: Value) {
        self.push_opcode(BrilligOpcode::Const { destination: result, value: constant });
    }

    /// Processes a not instruction.
    ///
    /// Not is computed using a subtraction operation as there is no native not instruction
    /// in Brillig.
    pub(crate) fn not_instruction(&mut self, condition: RegisterIndex, result: RegisterIndex) {
        let one = self.make_constant(Value::from(FieldElement::one()));

        // Compile !x as (1 - x)
        let opcode = BrilligOpcode::BinaryIntOp {
            destination: result,
            op: BinaryIntOp::Sub,
            bit_size: 1,
            lhs: one,
            rhs: condition,
        };
        self.push_opcode(opcode);
    }

    /// Processes a foreign call instruction.
    ///
    /// Note: the function being called is external and will
    /// not be linked during brillig generation.
    pub(crate) fn foreign_call_instruction(
        &mut self,
        func_name: String,
        inputs: &[RegisterValueOrArray],
        outputs: &[RegisterValueOrArray],
    ) {
        // TODO(https://github.com/noir-lang/acvm/issues/366): Enable multiple inputs and outputs to a foreign call
        let opcode = BrilligOpcode::ForeignCall {
            function: func_name,
            destination: outputs[0],
            input: inputs[0],
        };
        self.push_opcode(opcode);
    }

    /// Emits a load instruction
    pub(crate) fn load_instruction(
        &mut self,
        destination: RegisterIndex,
        source_pointer: RegisterIndex,
    ) {
        self.push_opcode(BrilligOpcode::Load { destination, source_pointer });
    }

    /// Emits a store instruction
    pub(crate) fn store_instruction(
        &mut self,
        destination_pointer: RegisterIndex,
        source: RegisterIndex,
    ) {
        self.push_opcode(BrilligOpcode::Store { destination_pointer, source });
    }

    /// Emits a truncate instruction.
    ///
    /// Note: Truncation is used as an optimization in the SSA IR
    /// for the ACIR generation pass; ACIR gen does not overflow
    /// on every integer operation since it would be in-efficient.
    /// Instead truncation instructions are emitted as to when a
    /// truncation should be done.
    /// For Brillig, all integer operations will overflow as its cheap.
    pub(crate) fn truncate_instruction(
        &mut self,
        destination_of_truncated_value: RegisterIndex,
        value_to_truncate: RegisterIndex,
    ) {
        // Effectively a no-op because brillig already has implicit truncation on integer
        // operations. We need only copy the value to it's destination.
        self.mov_instruction(destination_of_truncated_value, value_to_truncate);
    }

    /// Emits a stop instruction
    pub(crate) fn stop_instruction(&mut self) {
        self.push_opcode(BrilligOpcode::Stop);
    }

    /// Returns a register which holds the value of a constant
    pub(crate) fn make_constant(&mut self, constant: Value) -> RegisterIndex {
        let register = self.create_register();
        self.const_instruction(register, constant);
        register
    }

    /// Computes left % right by emitting the necessary Brillig opcodes.
    ///
    /// This is done by using the following formula:
    ///
    /// a % b = a - (b * (a / b))
    ///
    /// Brillig does not have an explicit modulo operation,
    /// so we must emit multiple opcodes and process it differently
    /// to other binary instructions.
    pub(crate) fn modulo_instruction(
        &mut self,
        result_register: RegisterIndex,
        left: RegisterIndex,
        right: RegisterIndex,
        bit_size: u32,
        signed: bool,
    ) {
        let scratch_register_i = self.create_register();
        let scratch_register_j = self.create_register();

        // i = left / right
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            op: match signed {
                true => BinaryIntOp::SignedDiv,
                false => BinaryIntOp::UnsignedDiv,
            },
            destination: scratch_register_i,
            bit_size,
            lhs: left,
            rhs: right,
        });

        // j = i * right
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Mul,
            destination: scratch_register_j,
            bit_size,
            lhs: scratch_register_i,
            rhs: right,
        });

        // result_register = left - j
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Sub,
            destination: result_register,
            bit_size,
            lhs: left,
            rhs: scratch_register_j,
        });
    }

    /// Returns the ith register after the special ones
    pub(crate) fn register(&self, i: usize) -> RegisterIndex {
        RegisterIndex::from(SpecialRegisters::Len as usize + i)
    }

    /// Saves all of the registers that have been used up until this point
    /// in the pointer passed in and returns the latest register before
    /// saving the registers.
    fn save_all_used_registers(&mut self, register_index: RegisterIndex) -> usize {
        // Save all of the used registers at this point in memory
        // because the function call will/may overwrite them.
        //
        // Copy the registers to memory
        let registers_len = self.latest_register; // abstraction leak -- put in memory.rs?

        // Store `1` in a register
        let one = self.create_register();
        self.push_opcode(BrilligOpcode::Const { destination: one, value: Value::from(1_usize) });

        self.allocate_array(register_index, registers_len as u32);
        let stack_pointer = self.create_register();
        self.binary_instruction(
            SpecialRegisters::stack_frame(),
            SpecialRegisters::call_depth(),
            stack_pointer,
            BrilligBinaryOp::Integer { op: BinaryIntOp::Add, bit_size: 32 },
        );
        self.store_instruction(stack_pointer, register_index);
        for i in SpecialRegisters::len()..registers_len {
            self.push_opcode(BrilligOpcode::Store {
                destination_pointer: register_index,
                source: RegisterIndex::from(i),
            });
            self.push_opcode(BrilligOpcode::BinaryIntOp {
                destination: register_index,
                op: BinaryIntOp::Add,
                bit_size: 32,
                lhs: register_index,
                rhs: one,
            });
        }

        registers_len
    }

    /// Call a function
    ///
    /// We first save the current registers into a new array, whose address is saved into the StackFrame (at CallDepth index)
    /// We move the arguments to registers 0,..n (plus the offset for the special registers)
    /// We increment CallDepth (in case the called function is doing some nested call)
    /// We add the unresolved call instruction (which will jump to the entry block of the called function)
    /// We decrement the CallDepth
    /// We copy the return values (which are on registers 0,..n (plus the offset for the special registers))
    /// into their location of the 'results', in the array where the registers were saved
    /// We load the registers from the memory, and we can continue the execution
    pub(crate) fn call(
        &mut self,
        arguments: &[RegisterIndex],
        results: &[RegisterIndex],
        function_entry_block_label: String,
        func_id: FunctionId,
    ) {
        // Initialize the registers which will hold the value(s) returned from the call
        //
        //
        // We need to initialize because we are going to save all the registers to memory
        for register in results {
            self.push_opcode(BrilligOpcode::Const {
                destination: *register,
                value: Value::from(0_usize),
            });
        }

        // Save all the registers
        //
        // This includes the arguments and return values
        // TODO: change name `register_index`
        let register_index = self.create_register();
        let latest_register_len_before_saving_registers =
            self.save_all_used_registers(register_index);

        // Store `1` in a register
        let one = self.create_register();
        self.push_opcode(BrilligOpcode::Const { destination: one, value: Value::from(1_usize) });

        // Move argument values to the front of the registers
        //
        // This means that the arguments will be in the first `n` registers after
        // the special registers which are reserved.
        for (i, argument) in arguments.iter().enumerate() {
            self.push_opcode(BrilligOpcode::Mov {
                destination: self.register(i),
                source: *argument,
            });
        }

        // Increment depth_call
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            destination: SpecialRegisters::call_depth(),
            op: BinaryIntOp::Add,
            bit_size: 32,
            lhs: SpecialRegisters::call_depth(),
            rhs: one,
        });

        // Call instruction
        self.add_unresolved_call(function_entry_block_label, func_id);

        // Decrement depth_call
        let one = self.make_constant(1u128.into());
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            destination: SpecialRegisters::call_depth(),
            op: BinaryIntOp::Sub,
            bit_size: 32,
            lhs: SpecialRegisters::call_depth(),
            rhs: one,
        });

        // Copy from registers SpecialRegisters::len,.. to the result registers, but at their saved memory location
        //
        // This stack_address value is the same as `register_index`
        // ie we are trying to get the pointer to the array that stored all of the
        // registers before the function call.
        //
        // The function call may have overwritten the original `registers` so we cannot reuse `register_index`
        // Note: Right now, the memory could have been overwritten by the function call
        // so the memory address that the registers were saved at is not the same as `register_index`.
        let stack_address = self.create_register();
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            destination: stack_address,
            op: BinaryIntOp::Add,
            bit_size: 32,
            // array that stores all of the pointers to the saved registers
            // before the call instruction.
            lhs: SpecialRegisters::stack_frame(),
            rhs: SpecialRegisters::call_depth(),
        });
        self.load_instruction(stack_address, stack_address);

        // Copy the result registers to the memory location of the previous saved registers:
        // TODO: rename `reg_adr`
        let reg_adr = self.create_register();
        for (i, result) in results.iter().enumerate() {
            let offset =
                self.make_constant(Value::from(result.to_usize() - SpecialRegisters::len()));
            self.binary_instruction(
                stack_address,
                offset,
                reg_adr,
                BrilligBinaryOp::Integer { op: BinaryIntOp::Add, bit_size: 32 },
            );
            self.store_instruction(reg_adr, self.register(i));
        }

        // Load the saved registers
        // TODO: rename `tmp`
        let tmp = self.create_register();
        for i in SpecialRegisters::len()..latest_register_len_before_saving_registers {
            self.const_instruction(tmp, Value::from(i - SpecialRegisters::len()));
            self.binary_instruction(
                stack_address,
                tmp,
                reg_adr,
                BrilligBinaryOp::Integer { op: BinaryIntOp::Add, bit_size: 32 },
            );
            self.load_instruction(RegisterIndex::from(i), reg_adr);
        }
    }
    /// Emits a modulo instruction against 2**target_bit_size
    ///
    /// Integer arithmetic in Brillig is currently constrained to 127 bit integers.
    /// We restrict the cast operation, so that integer types over 127 bits
    /// cannot be created.
    pub(crate) fn cast_instruction(
        &mut self,
        destination: RegisterIndex,
        source: RegisterIndex,
        target_bit_size: u32,
    ) {
        assert!(
            target_bit_size <= BRILLIG_INTEGER_ARITHMETIC_BIT_SIZE,
            "tried to cast to a bit size greater than allowed {target_bit_size}"
        );

        // The brillig VM performs all arithmetic operations modulo 2**bit_size
        // So to cast any value to a target bit size we can just issue a no-op arithmetic operation
        // With bit size equal to target_bit_size
        let zero = self.make_constant(Value::from(FieldElement::zero()));
        self.binary_instruction(
            source,
            zero,
            destination,
            BrilligBinaryOp::Integer { op: BinaryIntOp::Add, bit_size: target_bit_size },
        );
    }

    ///
    pub(crate) fn initialize_entry_function(&mut self, num_arguments: usize) {
        // This was chosen arbitrarily
        const MAXIMUM_NUMBER_OF_NESTED_CALLS: u32 = 50;

        // Translate the inputs by the special registers offset
        for i in (0..num_arguments).into_iter().rev() {
            self.mov_instruction(self.register(i), RegisterIndex::from(i));
        }

        // Initialize the first three registers to be the special registers
        //
        // Initialize the call-depth
        self.const_instruction(SpecialRegisters::call_depth(), Value::from(0_usize));
        assert_eq!(RegisterIndex::from(0), SpecialRegisters::call_depth());
        //
        // Initialize alloc
        self.const_instruction(SpecialRegisters::alloc(), Value::from(0_usize));
        assert_eq!(RegisterIndex::from(1), SpecialRegisters::alloc());
        //
        // Initialize the stack-frame
        self.allocate_array(SpecialRegisters::stack_frame(), MAXIMUM_NUMBER_OF_NESTED_CALLS);
        assert_eq!(RegisterIndex::from(2), SpecialRegisters::stack_frame());
    }
}

/// Type to encapsulate the binary operation types in Brillig
pub(crate) enum BrilligBinaryOp {
    Field { op: BinaryFieldOp },
    Integer { op: BinaryIntOp, bit_size: u32 },
    // Modulo operation requires more than one opcode
    // Brillig.
    Modulo { is_signed_integer: bool, bit_size: u32 },
}
