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
pub(crate) mod procedures;
pub(crate) mod registers;

mod codegen_binary;
mod codegen_calls;
mod codegen_control_flow;
mod codegen_intrinsic;
mod codegen_memory;
mod codegen_stack;
mod entry_point;
mod instructions;

use artifact::Label;
pub(crate) use instructions::BrilligBinaryOp;
use registers::{RegisterAllocator, ScratchSpace};

use self::{artifact::BrilligArtifact, debug_show::DebugToString, registers::Stack};
use crate::ssa::ir::dfg::CallStack;
use acvm::{
    acir::brillig::{MemoryAddress, Opcode as BrilligOpcode},
    AcirField,
};
use debug_show::DebugShow;

/// The Brillig VM does not apply a limit to the memory address space,
/// As a convention, we take use 32 bits. This means that we assume that
/// memory has 2^32 memory slots.
pub(crate) const BRILLIG_MEMORY_ADDRESSING_BIT_SIZE: u32 = 32;

// Registers reserved in runtime for special purposes.
pub(crate) enum ReservedRegisters {
    /// This register stores the free memory pointer. Allocations must be done after this pointer.
    FreeMemoryPointer = 0,
    /// This register stores the previous stack pointer. The registers of the caller are stored here.
    PreviousStackPointer = 1,
    /// This register stores a 1_usize constant.
    UsizeOne = 2,
}

impl ReservedRegisters {
    /// The number of reserved registers.
    ///
    /// This is used to offset the general registers
    /// which should not overwrite the special register
    const NUM_RESERVED_REGISTERS: usize = 3;

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

    /// Returns the usize one register. This will be used to perform arithmetic operations.
    pub(crate) fn usize_one() -> MemoryAddress {
        MemoryAddress::from(ReservedRegisters::UsizeOne as usize)
    }
}

/// Brillig context object that is used while constructing the
/// Brillig bytecode.
pub(crate) struct BrilligContext<F, Registers> {
    obj: BrilligArtifact<F>,
    /// Tracks register allocations
    registers: Registers,
    /// Context label, must be unique with respect to the function
    /// being linked.
    context_label: Label,
    /// Section label, used to separate sections of code
    current_section: usize,
    /// Stores the next available section
    next_section: usize,
    /// IR printer
    debug_show: DebugShow,
    /// Whether this context can call procedures or not.
    /// This is used to prevent a procedure from calling another procedure.
    can_call_procedures: bool,
}

/// Regular brillig context to codegen user defined functions
impl<F: AcirField + DebugToString> BrilligContext<F, Stack> {
    pub(crate) fn new(enable_debug_trace: bool) -> BrilligContext<F, Stack> {
        BrilligContext {
            obj: BrilligArtifact::default(),
            registers: Stack::new(),
            context_label: Label::entrypoint(),
            current_section: 0,
            next_section: 1,
            debug_show: DebugShow::new(enable_debug_trace),
            can_call_procedures: true,
        }
    }
    /// Allows disabling procedures so tests don't need a linking pass
    pub(crate) fn disable_procedures(&mut self) {
        self.can_call_procedures = false;
    }
}

/// Special brillig context to codegen compiler intrinsic shared procedures
impl<F: AcirField + DebugToString> BrilligContext<F, ScratchSpace> {
    pub(crate) fn new_for_procedure(enable_debug_trace: bool) -> BrilligContext<F, ScratchSpace> {
        BrilligContext {
            obj: BrilligArtifact::default(),
            registers: ScratchSpace::new(),
            context_label: Label::entrypoint(),
            current_section: 0,
            next_section: 1,
            debug_show: DebugShow::new(enable_debug_trace),
            can_call_procedures: false,
        }
    }
}

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Adds a brillig instruction to the brillig byte code
    fn push_opcode(&mut self, opcode: BrilligOpcode<F>) {
        self.obj.push_opcode(opcode);
    }

    /// Returns the artifact
    pub(crate) fn artifact(self) -> BrilligArtifact<F> {
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
        BitSize, ForeignCallParam, ForeignCallResult, HeapArray, HeapVector, IntegerBitSize,
        MemoryAddress, ValueOrArray,
    };
    use acvm::brillig_vm::brillig::HeapValueType;
    use acvm::brillig_vm::{VMStatus, VM};
    use acvm::{BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement};

    use crate::brillig::brillig_ir::{BrilligBinaryOp, BrilligContext};
    use crate::ssa::ir::function::FunctionId;

    use super::artifact::{BrilligParameter, GeneratedBrillig, Label};
    use super::registers::Stack;
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

    pub(crate) fn create_context(id: FunctionId) -> BrilligContext<FieldElement, Stack> {
        let mut context = BrilligContext::new(true);
        context.enter_context(Label::function(id));
        context
    }

    pub(crate) fn create_entry_point_bytecode(
        context: BrilligContext<FieldElement, Stack>,
        arguments: Vec<BrilligParameter>,
        returns: Vec<BrilligParameter>,
    ) -> GeneratedBrillig<FieldElement> {
        let artifact = context.artifact();
        let mut entry_point_artifact =
            BrilligContext::new_entry_point_artifact(arguments, returns, FunctionId::test_new(0));
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
            &[HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U32))],
            &[ValueOrArray::HeapVector(HeapVector { pointer: r_stack, size: r_output_size })],
            &[HeapValueType::Vector {
                value_types: vec![HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U32))],
            }],
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
