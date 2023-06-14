//! This module is an abstraction layer over `Brillig`
//! To allow for separation of concerns, it knows nothing
//! about SSA types, and can therefore be tested independently.
//! `brillig_gen` is therefore the module which combines both
//! ssa types and types in this module.
//! A similar paradigm can be seen with the `acir_ir` module.
pub(crate) mod artifact;
pub(crate) mod memory;

use crate::ssa_refactor::ir::{basic_block::BasicBlockId, function::FunctionId};

use self::{
    artifact::{BrilligArtifact, UnresolvedLocation},
    memory::BrilligMemory,
};
use acvm::{
    acir::brillig_vm::{
        BinaryFieldOp, BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, RegisterValueOrArray,
        Value,
    },
    FieldElement,
};

pub(crate) enum SpecialRegisters {
    CallDepth = 0,
    StackFrame = 1,
    Len = 2,
}

impl SpecialRegisters {
    pub(crate) fn len() -> usize {
        SpecialRegisters::Len as usize
    }
}

/// Brillig context object that is used while constructing the
/// Brillig bytecode.
pub(crate) struct BrilligContext {
    obj: BrilligArtifact,
    /// A usize indicating the latest un-used register.
    latest_register: usize,
    /// Tracks memory allocations
    memory: BrilligMemory,
}

impl BrilligContext {
    /// Adds a brillig instruction to the brillig byte code
    pub(crate) fn push_opcode(&mut self, opcode: BrilligOpcode) {
        self.obj.byte_code.push(opcode);
    }

    /// Returns the artifact
    pub(crate) fn artifact(self) -> BrilligArtifact {
        self.obj
    }

    pub(crate) fn new(func: FunctionId) -> BrilligContext {
        BrilligContext {
            obj: BrilligArtifact::new(func),
            latest_register: SpecialRegisters::Len as usize,
            memory: BrilligMemory::default(),
        }
    }

    pub(crate) fn block_label(&self, block_id: BasicBlockId) -> String {
        self.obj.block_label(block_id)
    }

    /// Allocates an array of size `size` and stores the pointer to the array
    /// in `pointer_register`
    pub(crate) fn allocate_array(&mut self, pointer_register: RegisterIndex, size: u32) {
        let array_pointer = self.memory.allocate(size as usize);
        self.push_opcode(BrilligOpcode::Const {
            destination: pointer_register,
            value: Value::from(array_pointer),
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
    fn add_unresolved_call(
        &mut self,
        call_instruction: BrilligOpcode,
        destination: UnresolvedLocation,
        func: FunctionId,
    ) {
        self.obj.add_unresolved_call(call_instruction, destination, func);
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
            // If the destination register index is more than the latest register,
            // we update the latest register to be the destination register because the
            // brillig vm will expand the number of registers internally, when it encounters
            // a register that has not been initialized.
            if destination_index > self.latest_register {
                self.latest_register = destination_index;
            }
            self.mov_instruction(destination_index.into(), *return_register);
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

    fn call_depth(&self) -> RegisterIndex {
        RegisterIndex::from(SpecialRegisters::CallDepth as usize)
    }

    /// Returns the ith register after the special ones
    fn register(&self, i: usize) -> RegisterIndex {
        RegisterIndex::from(SpecialRegisters::Len as usize + i)
    }
    pub(crate) fn stack_frame(&self) -> RegisterIndex {
        RegisterIndex::from(SpecialRegisters::StackFrame as usize)
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

    // TODO: document
    pub(crate) fn call(
        &mut self,
        arguments: &[RegisterIndex],
        results: &[RegisterIndex],
        label: String,
        func_id: FunctionId,
    ) {
        let register_index = self.create_register();
        let latest_register_len_before_saving_registers =
            self.save_all_used_registers(register_index);

        // Store `1` in a register
        let one = self.create_register();
        self.push_opcode(BrilligOpcode::Const { destination: one, value: Value::from(1_usize) });

        // Put the arguments on registers, starting at SpecialRegisters::Len
        for (i, argument) in arguments.iter().enumerate() {
            self.push_opcode(BrilligOpcode::Mov {
                destination: RegisterIndex::from(i + SpecialRegisters::Len as usize),
                source: *argument,
            });
        }

        // Increment depth_call
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            destination: self.call_depth(),
            op: BinaryIntOp::Add,
            bit_size: 32,
            lhs: self.call_depth(),
            rhs: one,
        });

        // Call instruction
        self.add_unresolved_call(
            BrilligOpcode::Call { location: 0 },
            UnresolvedLocation::Label(label),
            func_id,
        );

        // Copy from registers SpecialRegisters::len,.. to the result registers, but at their saved memory location
        //
        // This stack_address value is the same as `register_index`
        // ie we are trying to get the pointer to the array that stored all of the
        // registers before the function call.
        //
        // The function call may have overwritten the original `registers` so we cannot reuse `register_index`
        // Note: Right now, the memory could have been overwritten by the function call
        // so the memory address that the registers were saved at is not the same as `register_index`.
        let stack_adr = self.create_register();
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            destination: stack_adr,
            op: BinaryIntOp::Add,
            bit_size: 32,
            // array that stores all of the pointers to the saved registers
            lhs: self.stack_frame(),
            rhs: self.call_depth(), // TODO: decrement this by 1
        });
        self.load_instruction(stack_adr, stack_adr);

        // Copy the result registers to the memory location of the previous saved registers
        let reg_adr = self.create_register();
        for (i, result) in results.iter().enumerate() {
            self.binary_instruction(
                stack_adr,
                *result,
                reg_adr,
                BrilligBinaryOp::Integer { op: BinaryIntOp::Add, bit_size: 32 },
            );
            self.store_instruction(reg_adr, self.register(i));
        }

        // Load the saved registers
        let tmp = self.create_register();
        for i in 0..latest_register_len_before_saving_registers {
            self.const_instruction(tmp, Value::from(i));
            self.binary_instruction(
                stack_adr,
                tmp,
                reg_adr,
                BrilligBinaryOp::Integer { op: BinaryIntOp::Add, bit_size: 32 },
            );
            self.load_instruction(RegisterIndex::from(i + SpecialRegisters::Len as usize), reg_adr);
        }

        // Decrement depth_call
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            destination: self.call_depth(),
            op: BinaryIntOp::Sub,
            bit_size: 32,
            lhs: self.call_depth(),
            rhs: self.make_constant(1u128.into()),
        });
    }

    pub(crate) fn initialise_main(&mut self, input_len: usize) {
        // translate the inputs by the special registers offset
        for i in 0..input_len {
            self.push_opcode(BrilligOpcode::Mov {
                destination: RegisterIndex::from(i + SpecialRegisters::Len as usize),
                source: RegisterIndex::from(i),
            });
            //initialise the calldepth
            self.push_opcode(BrilligOpcode::Const {
                destination: self.call_depth(),
                value: Value::from(0_usize),
            });
            //initialise the stackframe
            self.allocate_array(self.stack_frame(), 50);
        }
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
