//! This module is an abstraction layer over `Brillig`
//! To allow for separation of concerns, it knows nothing
//! about SSA types, and can therefore be tested independently.
//! `brillig_gen` is therefore the module which combines both
//! ssa types and types in this module.
//! A similar paradigm can be seen with the `acir_ir` module.
pub(crate) mod artifact;
pub(crate) mod registers;

use crate::ssa_refactor::ir::{basic_block::BasicBlockId, function::FunctionId};

use self::artifact::{BrilligArtifact, UnresolvedLocation};
use self::registers::BrilligRegistersContext;
use acvm::{
    acir::brillig_vm::{
        BinaryFieldOp, BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, RegisterValueOrArray,
        Value,
    },
    FieldElement,
};

// Registers reserved for special purpose by BrilligGen when generating the bytecode
pub(crate) enum SpecialRegisters {
    /// Contains the address of an array which hold stack-frame data: the address of the memory where the registers are saved before a call
    /// The stack-frame has a fixed length, meaning the number nested function calls is limited to this length
    StackFrame = 0,
    /// The amount of special registers.
    _Len = 1,
}

/// We don't want to allocate over the stack register, our single special register.
const NUM_RESERVED_REGISTERS: usize = (SpecialRegisters::StackFrame as usize) + 1_usize;

impl SpecialRegisters {
    // TODO: doc
    pub(crate) fn len() -> usize {
        SpecialRegisters::_Len as usize
    }

    // TODO: doc
    pub(crate) fn stack_pointer() -> RegisterIndex {
        RegisterIndex::from(SpecialRegisters::StackFrame as usize)
    }
}

/// Brillig context object that is used while constructing the
/// Brillig bytecode.
pub(crate) struct BrilligContext {
    obj: BrilligArtifact,
    registers: BrilligRegistersContext,
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

    pub(crate) fn new(func: FunctionId) -> BrilligContext {
        BrilligContext {
            obj: BrilligArtifact::new(func),
            registers: BrilligRegistersContext::new(),
        }
    }

    pub(crate) fn block_label(&self, block_id: BasicBlockId) -> String {
        self.obj.block_label(block_id)
    }

    /// Allocates an array of size `size` and stores the pointer to the array
    /// in `pointer_register`
    pub(crate) fn allocate_array(&mut self, pointer_register: RegisterIndex, size: u32) {
        let len = self.allocate_register();
        self.const_instruction(len, Value::from(size as usize));

        self.push_opcode(BrilligOpcode::Mov {
            destination: pointer_register,
            source: SpecialRegisters::stack_pointer(),
        });
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            destination: SpecialRegisters::stack_pointer(),
            op: BinaryIntOp::Add,
            bit_size: 64,
            lhs: SpecialRegisters::stack_pointer(),
            rhs: len,
        });
    }

    /// Adds a label to the next opcode
    pub(crate) fn add_label_to_next_opcode<T: ToString>(&mut self, label: T) {
        self.obj.add_label_at_position(label.to_string(), self.obj.index_of_next_opcode());
    }

    /// Adds a unresolved `Jump` instruction to the bytecode.
    pub(crate) fn jump_instruction(&mut self, target_label: BasicBlockId) {
        self.add_unresolved_jump(
            BrilligOpcode::Jump { location: 0 },
            UnresolvedLocation::Label(self.block_label(target_label)),
        );
    }

    /// Adds a unresolved `JumpIf` instruction to the bytecode.
    pub(crate) fn jump_if_instruction(
        &mut self,
        condition: RegisterIndex,
        target_label: BasicBlockId,
    ) {
        self.add_unresolved_jump(
            BrilligOpcode::JumpIf { condition, location: 0 },
            UnresolvedLocation::Label(self.block_label(target_label)),
        );
    }

    /// Adds a unresolved `Jump` instruction to the bytecode.
    fn add_unresolved_jump(
        &mut self,
        jmp_instruction: BrilligOpcode,
        destination: UnresolvedLocation,
    ) {
        self.obj.add_unresolved_jump(jmp_instruction, destination);
    }

    /// Adds a unresolved `Call` instruction to the bytecode.
    pub(crate) fn add_unresolved_call(
        &mut self,
        destination: UnresolvedLocation,
        func: FunctionId,
    ) {
        self.obj.add_unresolved_call(BrilligOpcode::Call { location: 0 }, destination, func);
    }

    /// Allocates an unused register.
    pub(crate) fn allocate_register(&mut self) -> RegisterIndex {
        self.registers.allocate_register()
    }

    /// Push a register to the deallocation list, ready for reuse.
    /// TODO(AD): Currently only used for constants. Later, do lifecycle analysis.
    pub(crate) fn deallocate_register(&mut self, register_index: RegisterIndex) {
        self.registers.deallocate_register(register_index);
    }
}

impl BrilligContext {
    /// Emits brillig bytecode to jump to a trap condition if `condition`
    /// is false.
    pub(crate) fn constrain_instruction(&mut self, condition: RegisterIndex) {
        // Jump to the relative location after the trap
        self.add_unresolved_jump(
            BrilligOpcode::JumpIf { condition, location: 0 },
            UnresolvedLocation::Relative(2),
        );
        self.push_opcode(BrilligOpcode::Trap);
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
            // TODO(AD) really dont think this could have ever happened - how could
            // packing the registers starting at 0 cause > the highest live register count?
            // // If the destination register index is more than the latest register,
            // // we update the latest register to be the destination register because the
            // // brillig vm will expand the number of registers internally, when it encounters
            // // a register that has not been initialized.
            // if destination_index > self.latest_register {
            //     self.latest_register = destination_index;
            // }
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
        let one_register = self.make_constant(Value::from(FieldElement::one()));

        // Compile !x as (1 - x)
        let opcode = BrilligOpcode::BinaryIntOp {
            destination: result,
            op: BinaryIntOp::Sub,
            bit_size: 1,
            lhs: one_register,
            rhs: condition,
        };
        self.push_opcode(opcode);
        // Mark register slot as available for reuse
        self.deallocate_register(one_register);
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
        let register = self.allocate_register();
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
        let scratch_register_i = self.allocate_register();
        let scratch_register_j = self.allocate_register();

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
        RegisterIndex::from(NUM_RESERVED_REGISTERS + i)
    }

    /// Saves all of the registers that have been used up until this point.
    fn save_all_used_registers(&mut self) -> Vec<RegisterIndex> {
        // Save all of the used registers at this point in memory
        // because the function call will/may overwrite them.
        //
        // Note that here it is important that the stack pointer register is at register 0,
        // as after the first register save we add to the
        let used_registers = self.registers.used_registers_iter().collect::<Vec<_>>();
        for register in used_registers.iter() {
            self.store_instruction(SpecialRegisters::stack_pointer(), *register);
            // Add one to our stack pointer
            self.usize_op(SpecialRegisters::stack_pointer(), BinaryIntOp::Add, 1);
        }
        used_registers
    }

    /// Loads all of the registers that have been save by save_all_used_registers.
    fn load_all_saved_registers(&mut self, used_registers: &[RegisterIndex]) {
        // Load all of the used registers that we saved.
        // We do all the reverse operations of save_all_used_registers.
        // Iterate our registers in reverse
        for register in used_registers.iter().rev() {
            // Subtract one from our stack pointer
            self.usize_op(SpecialRegisters::stack_pointer(), BinaryIntOp::Sub, 1);
            self.load_instruction(*register, SpecialRegisters::stack_pointer());
        }
    }

    /// Utility method to perform a binary instruction with a constant value
    pub(crate) fn usize_op(
        &mut self,
        destination: RegisterIndex,
        op: BinaryIntOp,
        constant: usize,
    ) {
        let const_register = self.make_constant(Value::from(constant));
        self.binary_instruction(
            destination,
            destination,
            const_register,
            // TODO(AD): magic constant
            BrilligBinaryOp::Integer { op, bit_size: 64 },
        );
        // Mark as no longer used for this purpose, frees for reuse
        self.deallocate_register(const_register);
    }

    // Used before a call instruction.
    // Save all the registers we have used to the stack.
    // Move argument values to the front of the register indices.
    pub(crate) fn pre_call_save_registers_prep_args(
        &mut self,
        arguments: &[RegisterIndex],
    ) -> Vec<RegisterIndex> {
        // Save all the registers we have used to the stack.
        let saved_registers = self.save_all_used_registers();

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

        saved_registers
    }

    // Used after a call instruction.
    // Move return values to the front of the register indices.
    // Load all the registers we have previous saved in save_registers_prep_args.
    pub(crate) fn post_call_prep_returns_load_registers(
        &mut self,
        result_registers: &[RegisterIndex],
        saved_registers: &[RegisterIndex],
    ) {
        // Allocate our result registers and write into them
        // We assume the return values of our call are held in 0..num results register indices
        for (i, result_register) in result_registers.iter().enumerate() {
            self.mov_instruction(*result_register, self.register(i));
        }

        // Restore all the same registers we have, in exact reverse order.
        // Note that we have allocated some registers above, which we will not be handling here,
        // only restoring registers that were used prior to the call finishing.
        // After the call instruction, the stack frame pointer should be back to where we left off,
        // so we do our instructions in reverse order.
        self.load_all_saved_registers(saved_registers);
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
