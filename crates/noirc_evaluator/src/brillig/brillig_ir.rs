//! This module is an abstraction layer over `Brillig`
//! To allow for separation of concerns, it knows nothing
//! about SSA types, and can therefore be tested independently.
//! `brillig_gen` is therefore the module which combines both
//! ssa types and types in this module.
//! A similar paradigm can be seen with the `acir_ir` module.
pub(crate) mod artifact;
pub(crate) mod memory;

use self::artifact::{BrilligArtifact, UnresolvedJumpLocation};
use acvm::{
    acir::brillig_vm::{
        BinaryFieldOp, BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, RegisterValueOrArray,
        Value,
    },
    FieldElement,
};

#[derive(Default)]
pub(crate) struct BrilligContext {
    obj: BrilligArtifact,
    /// A usize indicating the latest un-used register.
    latest_register: usize,
    /// Tracks memory allocations
    memory: memory::BrilligMemory,
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

    pub(crate) fn allocate_array(&mut self, pointer_register: RegisterIndex, size: u32) {
        let array_pointer = self.memory.allocate(size as usize);
        self.push_opcode(BrilligOpcode::Const {
            destination: pointer_register,
            value: Value::from(array_pointer),
        });
    }

    pub(crate) fn add_label_to_next_opcode(&mut self, label: String) {
        self.obj.add_label_at_position(label, self.obj.index_of_next_opcode());
    }

    /// Adds a unresolved `Jump` instruction to the bytecode.
    pub(crate) fn jump_instruction<T: ToString>(&mut self, target_label: T) {
        self.add_unresolved_jump(
            BrilligOpcode::Jump { location: 0 },
            UnresolvedJumpLocation::Label(target_label.to_string()),
        );
    }

    /// Adds a unresolved `JumpIf` instruction to the bytecode.
    pub(crate) fn jump_if_instruction<T: ToString>(
        &mut self,
        condition: RegisterIndex,
        target_label: T,
    ) {
        self.add_unresolved_jump(
            BrilligOpcode::JumpIf { condition, location: 0 },
            UnresolvedJumpLocation::Label(target_label.to_string()),
        );
    }

    fn add_unresolved_jump(
        &mut self,
        jmp_instruction: BrilligOpcode,
        destination: UnresolvedJumpLocation,
    ) {
        self.obj.add_unresolved_jump(jmp_instruction, destination);
    }

    /// Creates a new register.
    pub(crate) fn create_register(&mut self) -> RegisterIndex {
        let register = RegisterIndex::from(self.latest_register);
        self.latest_register += 1;
        register
    }
}

impl BrilligContext {
    pub(crate) fn constrain_instruction(&mut self, condition: RegisterIndex) {
        // jump to the relative location after the trap
        self.add_unresolved_jump(
            BrilligOpcode::JumpIf { condition, location: 0 },
            UnresolvedJumpLocation::Relative(2),
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
        self.push_opcode(BrilligOpcode::Stop);
    }

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

    pub(crate) fn foreign_call_instruction(
        &mut self,
        func_name: String,
        inputs: &[RegisterIndex],
        outputs: &[RegisterIndex],
    ) {
        let opcode = BrilligOpcode::ForeignCall {
            function: func_name,
            destination: RegisterValueOrArray::RegisterIndex(outputs[0]),
            input: RegisterValueOrArray::RegisterIndex(inputs[0]),
        };
        self.push_opcode(opcode);
    }

    pub(crate) fn load_instruction(
        &mut self,
        destination: RegisterIndex,
        source_pointer: RegisterIndex,
    ) {
        self.push_opcode(BrilligOpcode::Load { destination, source_pointer });
    }

    pub(crate) fn store_instruction(
        &mut self,
        destination_pointer: RegisterIndex,
        source: RegisterIndex,
    ) {
        self.push_opcode(BrilligOpcode::Store { destination_pointer, source });
    }

    pub(crate) fn truncate_instruction(
        &mut self,
        destination_of_truncated_value: RegisterIndex,
        value_to_truncate: RegisterIndex,
    ) {
        // Effectively a no-op because brillig already has implicit truncation on integer
        // operations. We need only copy the value to it's destination.
        self.mov_instruction(destination_of_truncated_value, value_to_truncate);
    }

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
}

/// Type to encapsulate the binary operation types in Brillig
pub(crate) enum BrilligBinaryOp {
    Field { op: BinaryFieldOp },
    Integer { op: BinaryIntOp, bit_size: u32 },
    // Modulo operation requires more than one opcode
    // Brillig.
    Modulo { is_signed_integer: bool, bit_size: u32 },
}
