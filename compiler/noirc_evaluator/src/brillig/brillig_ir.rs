//! This module is an abstraction layer over `Brillig`
//! To allow for separation of concerns, it knows nothing
//! about SSA types, and can therefore be tested independently.
//! `brillig_gen` is therefore the module which combines both
//! ssa types and types in this module.
//! A similar paradigm can be seen with the `acir_ir` module.
//!
//! The brillig ir provides instructions and codegens.
//! The instructions are low level operations that are printed via debug_show.
//! They should emit few opcodes. Codegens on the other hand orchestrate the
//! low level instructions to emit the desired high level operation.
pub(crate) mod artifact;
pub(crate) mod brillig_variable;
pub(crate) mod debug_show;
pub(crate) mod registers;

mod codegen_binary;
mod codegen_calls;
mod codegen_control_flow;
mod codegen_intrinsic;
mod codegen_memory;
mod codegen_stack;
mod entry_point;
mod instructions;

pub(crate) use instructions::BrilligBinaryOp;

use self::{artifact::BrilligArtifact, registers::BrilligRegistersContext};
use crate::ssa::ir::dfg::CallStack;
use acvm::{
    acir::brillig::{MemoryAddress, Opcode as BrilligOpcode},
    FieldElement,
};
use debug_show::DebugShow;

/// The Brillig VM does not apply a limit to the memory address space,
/// As a convention, we take use 64 bits. This means that we assume that
/// memory has 2^64 memory slots.
pub(crate) const BRILLIG_MEMORY_ADDRESSING_BIT_SIZE: u32 = 64;

// Registers reserved in runtime for special purposes.
pub(crate) enum ReservedRegisters {
    /// This register stores the free memory pointer. Allocations must be done after this pointer.
    FreeMemoryPointer = 0,
    /// This register stores the previous stack pointer. The registers of the caller are stored here.
    PreviousStackPointer = 1,
}

impl ReservedRegisters {
    /// The number of reserved registers.
    ///
    /// This is used to offset the general registers
    /// which should not overwrite the special register
    const NUM_RESERVED_REGISTERS: usize = 2;

    /// Returns the length of the reserved registers
    pub(crate) fn len() -> usize {
        Self::NUM_RESERVED_REGISTERS
    }

    /// Returns the free memory pointer register. This will get used to allocate memory in runtime.
    pub(crate) fn free_memory_pointer() -> MemoryAddress {
        MemoryAddress::from(ReservedRegisters::FreeMemoryPointer as usize)
    }

    /// Returns the previous stack pointer register. This will be used to restore the registers after a fn call.
    pub(crate) fn previous_stack_pointer() -> MemoryAddress {
        MemoryAddress::from(ReservedRegisters::PreviousStackPointer as usize)
    }

    /// Returns a user defined (non-reserved) register index.
    fn user_register_index(index: usize) -> MemoryAddress {
        MemoryAddress::from(index + ReservedRegisters::len())
    }
}

/// Brillig context object that is used while constructing the
/// Brillig bytecode.
pub(crate) struct BrilligContext {
    obj: BrilligArtifact,
    /// Tracks register allocations
    registers: BrilligRegistersContext,
    /// Context label, must be unique with respect to the function
    /// being linked.
    context_label: String,
    /// Section label, used to separate sections of code
    section_label: usize,
    /// Stores the next available section
    next_section: usize,
    /// IR printer
    debug_show: DebugShow,
    /// Counter for generating bigint ids in unconstrained functions
    bigint_new_id: u32,
}

impl BrilligContext {
    /// Initial context state
    pub(crate) fn new(enable_debug_trace: bool) -> BrilligContext {
        BrilligContext {
            obj: BrilligArtifact::default(),
            registers: BrilligRegistersContext::new(),
            context_label: String::default(),
            section_label: 0,
            next_section: 1,
            debug_show: DebugShow::new(enable_debug_trace),
            bigint_new_id: 0,
        }
    }

    pub(crate) fn get_new_bigint_id(&mut self) -> u32 {
        let result = self.bigint_new_id;
        self.bigint_new_id += 1;
        result
    }
    /// Adds a brillig instruction to the brillig byte code
    fn push_opcode(&mut self, opcode: BrilligOpcode<FieldElement>) {
        self.obj.push_opcode(opcode);
    }

    /// Returns the artifact
    pub(crate) fn artifact(self) -> BrilligArtifact {
        self.obj
    }

    /// Sets a current call stack that the next pushed opcodes will be associated with.
    pub(crate) fn set_call_stack(&mut self, call_stack: CallStack) {
        self.obj.set_call_stack(call_stack);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::vec;

    use acvm::acir::brillig::{
        ForeignCallParam, ForeignCallResult, HeapArray, HeapVector, MemoryAddress, ValueOrArray,
    };
    use acvm::brillig_vm::brillig::HeapValueType;
    use acvm::brillig_vm::{VMStatus, VM};
    use acvm::{BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement};

    use crate::brillig::brillig_ir::{BrilligBinaryOp, BrilligContext};

    use super::artifact::{BrilligParameter, GeneratedBrillig};
    use super::{BrilligOpcode, ReservedRegisters};

    pub(crate) struct DummyBlackBoxSolver;

    impl BlackBoxFunctionSolver<FieldElement> for DummyBlackBoxSolver {
        fn schnorr_verify(
            &self,
            _public_key_x: &FieldElement,
            _public_key_y: &FieldElement,
            _signature: &[u8; 64],
            _message: &[u8],
        ) -> Result<bool, BlackBoxResolutionError> {
            Ok(true)
        }
        fn pedersen_commitment(
            &self,
            _inputs: &[FieldElement],
            _domain_separator: u32,
        ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
            Ok((2_u128.into(), 3_u128.into()))
        }
        fn pedersen_hash(
            &self,
            _inputs: &[FieldElement],
            _domain_separator: u32,
        ) -> Result<FieldElement, BlackBoxResolutionError> {
            Ok(6_u128.into())
        }
        fn multi_scalar_mul(
            &self,
            _points: &[FieldElement],
            _scalars_lo: &[FieldElement],
            _scalars_hi: &[FieldElement],
        ) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
            Ok((4_u128.into(), 5_u128.into(), 0_u128.into()))
        }

        fn ec_add(
            &self,
            _input1_x: &FieldElement,
            _input1_y: &FieldElement,
            _input1_infinite: &FieldElement,
            _input2_x: &FieldElement,
            _input2_y: &FieldElement,
            _input2_infinite: &FieldElement,
        ) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
            panic!("Path not trodden by this test")
        }

        fn poseidon2_permutation(
            &self,
            _inputs: &[FieldElement],
            _len: u32,
        ) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
            Ok(vec![0_u128.into(), 1_u128.into(), 2_u128.into(), 3_u128.into()])
        }
    }

    pub(crate) fn create_context() -> BrilligContext {
        let mut context = BrilligContext::new(true);
        context.enter_context("test");
        context
    }

    pub(crate) fn create_entry_point_bytecode(
        context: BrilligContext,
        arguments: Vec<BrilligParameter>,
        returns: Vec<BrilligParameter>,
    ) -> GeneratedBrillig {
        let artifact = context.artifact();
        let mut entry_point_artifact =
            BrilligContext::new_entry_point_artifact(arguments, returns, "test".to_string());
        entry_point_artifact.link_with(&artifact);
        entry_point_artifact.finish()
    }

    pub(crate) fn create_and_run_vm(
        calldata: Vec<FieldElement>,
        bytecode: &[BrilligOpcode<FieldElement>],
    ) -> (VM<'_, FieldElement, DummyBlackBoxSolver>, usize, usize) {
        let mut vm = VM::new(calldata, bytecode, vec![], &DummyBlackBoxSolver);

        let status = vm.process_opcodes();
        if let VMStatus::Finished { return_data_offset, return_data_size } = status {
            (vm, return_data_offset, return_data_size)
        } else {
            panic!("VM did not finish")
        }
    }

    /// Test a Brillig foreign call returning a vector
    #[test]
    fn test_brillig_ir_foreign_call_return_vector() {
        // pseudo-noir:
        //
        // #[oracle(get_number_sequence)]
        // unconstrained fn get_number_sequence(size: u32) -> Vec<u32> {
        // }
        //
        // unconstrained fn main() -> Vec<u32> {
        //   let the_sequence = get_number_sequence(12);
        //   assert(the_sequence.len() == 12);
        // }
        let mut context = BrilligContext::new(true);
        let r_stack = ReservedRegisters::free_memory_pointer();
        // Start stack pointer at 0
        context.usize_const_instruction(r_stack, FieldElement::from(ReservedRegisters::len() + 3));
        let r_input_size = MemoryAddress::from(ReservedRegisters::len());
        let r_array_ptr = MemoryAddress::from(ReservedRegisters::len() + 1);
        let r_output_size = MemoryAddress::from(ReservedRegisters::len() + 2);
        let r_equality = MemoryAddress::from(ReservedRegisters::len() + 3);
        context.usize_const_instruction(r_input_size, FieldElement::from(12_usize));
        // copy our stack frame to r_array_ptr
        context.mov_instruction(r_array_ptr, r_stack);
        context.foreign_call_instruction(
            "make_number_sequence".into(),
            &[ValueOrArray::MemoryAddress(r_input_size)],
            &[HeapValueType::Simple(32)],
            &[ValueOrArray::HeapVector(HeapVector { pointer: r_stack, size: r_output_size })],
            &[HeapValueType::Vector { value_types: vec![HeapValueType::Simple(32)] }],
        );
        // push stack frame by r_returned_size
        context.memory_op_instruction(r_stack, r_output_size, r_stack, BrilligBinaryOp::Add);
        // check r_input_size == r_output_size
        context.memory_op_instruction(
            r_input_size,
            r_output_size,
            r_equality,
            BrilligBinaryOp::Equals,
        );
        // We push a JumpIf and Trap opcode directly as the constrain instruction
        // uses unresolved jumps which requires a block to be constructed in SSA and
        // we don't need this for Brillig IR tests
        context.push_opcode(BrilligOpcode::JumpIf { condition: r_equality, location: 8 });
        context.push_opcode(BrilligOpcode::Trap { revert_data: HeapArray::default() });

        context.stop_instruction();

        let bytecode: Vec<BrilligOpcode<FieldElement>> = context.artifact().finish().byte_code;
        let number_sequence: Vec<FieldElement> =
            (0_usize..12_usize).map(FieldElement::from).collect();
        let mut vm = VM::new(
            vec![],
            &bytecode,
            vec![ForeignCallResult { values: vec![ForeignCallParam::Array(number_sequence)] }],
            &DummyBlackBoxSolver,
        );
        let status = vm.process_opcodes();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });
    }
}
