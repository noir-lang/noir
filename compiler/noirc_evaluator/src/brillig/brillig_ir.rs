//! This module is an abstraction layer over `Brillig`.
//! To allow for separation of concerns, it knows nothing
//! about SSA types, and can therefore be tested independently.
//! `brillig_gen` is therefore the module which combines both
//! ssa types and types in this module.
//! A similar paradigm can be seen with the `acir_ir` module.
//!
//! The brillig IR provides instructions and codegens.
//! The instructions are low level operations that are printed via `debug_show`.
//! They should emit few opcodes. Codegens on the other hand orchestrate the
//! low level instructions to emit the desired high level operation.
pub mod artifact;
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

use std::{cell::RefCell, rc::Rc};

use artifact::Label;
use brillig_variable::SingleAddrVariable;
pub(crate) use instructions::BrilligBinaryOp;
use noirc_errors::call_stack::CallStackId;
use registers::{RegisterAllocator, ScratchSpace};

use crate::brillig::assert_u32;

pub(crate) use self::registers::LayoutConfig;
use self::{artifact::BrilligArtifact, debug_show::DebugToString, registers::Stack};
use acvm::{
    AcirField,
    acir::brillig::{MemoryAddress, Opcode as BrilligOpcode},
    brillig_vm::{FREE_MEMORY_POINTER_ADDRESS, STACK_POINTER_ADDRESS},
};
use debug_show::DebugShow;

use super::{BrilligOptions, FunctionId, GlobalSpace, ProcedureId};

/// The Brillig VM does not apply a limit to the memory address space,
/// As a convention, we take use 32 bits. This means that we assume that
/// memory has 2^32 memory slots.
pub(crate) const BRILLIG_MEMORY_ADDRESSING_BIT_SIZE: u32 = 32;

/// Registers reserved in runtime for special purposes.
pub(crate) struct ReservedRegisters;

impl ReservedRegisters {
    /// The number of reserved registers. These are allocated in the first memory positions.
    /// The stack should start after the reserved registers.
    const NUM_RESERVED_REGISTERS: usize = 3;

    /// Returns the length of the reserved registers
    pub(crate) fn len() -> usize {
        Self::NUM_RESERVED_REGISTERS
    }

    /// This register stores the stack pointer. All relative memory addresses are relative to this pointer.
    pub(crate) fn stack_pointer() -> MemoryAddress {
        STACK_POINTER_ADDRESS
    }

    /// This register stores the free memory pointer. Allocations must be done after this pointer.
    ///
    /// This represents the heap, and we make sure during entry point generation that it is initialized
    /// with a value that lies beyond the maximum stack size, so there can never be an overlap.
    pub(crate) fn free_memory_pointer() -> MemoryAddress {
        FREE_MEMORY_POINTER_ADDRESS
    }

    /// This register stores a 1_usize constant.
    pub(crate) fn usize_one() -> MemoryAddress {
        MemoryAddress::direct(2)
    }
}

/// Brillig context object that is used while constructing the
/// Brillig bytecode.
pub(crate) struct BrilligContext<F, Registers> {
    obj: BrilligArtifact<F>,
    /// Tracks register allocations
    registers: Rc<RefCell<Registers>>,
    /// Context label, must be unique with respect to the function being linked.
    context_label: Label,
    /// Section label, used to separate sections of code within a context.
    current_section: usize,
    /// Stores the next available section.
    next_section: usize,
    /// IR printer
    debug_show: DebugShow,
    /// Whether this context can call procedures or not.
    /// This is used to prevent a procedure from calling another procedure.
    can_call_procedures: bool,
    /// Insert extra assertions that we expect to be true, at the cost of larger bytecode size.
    enable_debug_assertions: bool,
    /// Count the number of arrays that are copied, and output this to stdout
    count_arrays_copied: bool,

    globals_memory_size: Option<usize>,
}

impl<F, R: RegisterAllocator> BrilligContext<F, R> {
    /// Memory layout information. See [self::registers] for more information about the memory layout.
    pub(crate) fn layout(&self) -> LayoutConfig {
        self.registers().layout()
    }

    /// Enable the insertion of bytecode with extra assertions during testing.
    pub(crate) fn enable_debug_assertions(&self) -> bool {
        self.enable_debug_assertions
    }

    /// Returns the address of the implicit debug variable containing the count of
    /// implicitly copied arrays as a result of RC's copy on write semantics.
    pub(crate) fn array_copy_counter_address(&self) -> MemoryAddress {
        assert!(
            self.count_arrays_copied,
            "`count_arrays_copied` is not set, so the array copy counter does not exist"
        );

        // The copy counter is always put in the first global slot
        MemoryAddress::direct(assert_u32(GlobalSpace::start_with_layout(&self.layout())))
    }

    /// If this flag is set, compile the array copy counter as a global.
    pub(crate) fn count_array_copies(&self) -> bool {
        self.count_arrays_copied
    }

    /// Set the globals memory size if it is not already set.
    /// If it is already set, this will assert that the two values must be equal.
    pub(crate) fn set_globals_memory_size(&mut self, new_size: Option<usize>) {
        if self.globals_memory_size.is_some() {
            assert_eq!(
                self.globals_memory_size, new_size,
                "Tried to change globals_memory_size to a different value"
            );
        }
        self.globals_memory_size = new_size;
    }

    /// Returns the artifact, discarding the rest of the context.
    pub(crate) fn into_artifact(self) -> BrilligArtifact<F> {
        self.obj
    }

    /// Returns the artifact.
    pub(crate) fn artifact(&self) -> &BrilligArtifact<F> {
        &self.obj
    }

    pub(crate) fn name(&self) -> &str {
        &self.obj.name
    }
}

/// Regular brillig context to codegen user defined functions
impl<F: AcirField + DebugToString> BrilligContext<F, Stack> {
    pub(crate) fn new(function_name: &str, options: &BrilligOptions) -> BrilligContext<F, Stack> {
        let mut obj = BrilligArtifact::default();
        obj.name = function_name.to_owned();
        BrilligContext {
            obj,
            registers: Rc::new(RefCell::new(Stack::new(options.layout))),
            context_label: Label::entrypoint(),
            current_section: 0,
            next_section: 1,
            debug_show: DebugShow::new(options.enable_debug_trace),
            enable_debug_assertions: options.enable_debug_assertions,
            count_arrays_copied: options.enable_array_copy_counter,
            can_call_procedures: true,
            globals_memory_size: None,
        }
    }
}

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Splits a two's complement signed integer in the sign bit and the absolute value.
    /// For example, -6 i8 (11111010) is split to 00000110 (6, absolute value) and 1 (is_negative).
    pub(crate) fn absolute_value(
        &mut self,
        num: SingleAddrVariable,
        absolute_value: SingleAddrVariable,
        result_is_negative: SingleAddrVariable,
    ) {
        let max_positive = self
            .make_constant_instruction(((1_u128 << (num.bit_size - 1)) - 1).into(), num.bit_size);

        // Compute if num is negative
        self.binary_instruction(*max_positive, num, result_is_negative, BrilligBinaryOp::LessThan);

        // Two's complement of num
        let zero = self.make_constant_instruction(0_usize.into(), num.bit_size);
        let twos_complement = self.allocate_single_addr(num.bit_size);
        self.binary_instruction(*zero, num, *twos_complement, BrilligBinaryOp::Sub);

        // absolute_value = result_is_negative ? twos_complement : num
        self.codegen_branch(result_is_negative.address, |ctx, is_negative| {
            if is_negative {
                ctx.mov_instruction(absolute_value.address, twos_complement.address);
            } else {
                ctx.mov_instruction(absolute_value.address, num.address);
            }
        });
    }

    pub(crate) fn convert_signed_division(
        &mut self,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
    ) {
        let left_is_negative = self.allocate_single_addr_bool();
        let left_abs_value = self.allocate_single_addr(left.bit_size);

        let right_is_negative = self.allocate_single_addr_bool();
        let right_abs_value = self.allocate_single_addr(right.bit_size);

        let result_is_negative = self.allocate_single_addr_bool();

        // Compute both absolute values
        self.absolute_value(left, *left_abs_value, *left_is_negative);
        self.absolute_value(right, *right_abs_value, *right_is_negative);

        // Perform the division on the absolute values
        self.binary_instruction(
            *left_abs_value,
            *right_abs_value,
            result,
            BrilligBinaryOp::UnsignedDiv,
        );

        // Compute result sign
        self.binary_instruction(
            *left_is_negative,
            *right_is_negative,
            *result_is_negative,
            BrilligBinaryOp::Xor,
        );

        self.codegen_branch(result_is_negative.address, |ctx, is_negative| {
            if is_negative {
                // If result has to be negative, perform two's complement
                let zero = ctx.make_constant_instruction(0_usize.into(), result.bit_size);
                ctx.binary_instruction(*zero, result, result, BrilligBinaryOp::Sub);
            } else {
                // else the result is positive and so it must be less than '2**(bit_size-1)'
                let max = 1_u128 << (left.bit_size - 1);
                let max = ctx.make_constant_instruction(max.into(), left.bit_size);
                let no_overflow = ctx.allocate_single_addr_bool();
                ctx.binary_instruction(result, *max, *no_overflow, BrilligBinaryOp::LessThan);
                ctx.codegen_if_not(no_overflow.address, |ctx2| {
                    ctx2.codegen_constrain(
                        *no_overflow,
                        Some("Attempt to divide with overflow".to_string()),
                    );
                });
            }
        });
    }
}

/// Special brillig context to codegen compiler intrinsic shared procedures
impl<F: AcirField + DebugToString> BrilligContext<F, ScratchSpace> {
    /// Create a [BrilligContext] with a [ScratchSpace] for passing procedure arguments.
    pub(crate) fn new_for_procedure(
        procedure_id: ProcedureId,
        options: &BrilligOptions,
    ) -> BrilligContext<F, ScratchSpace> {
        let mut obj = BrilligArtifact::default();
        obj.procedure = Some(procedure_id);
        BrilligContext {
            obj,
            registers: Rc::new(RefCell::new(ScratchSpace::new(options.layout))),
            context_label: Label::entrypoint(),
            current_section: 0,
            next_section: 1,
            debug_show: DebugShow::new(options.enable_debug_trace),
            enable_debug_assertions: options.enable_debug_assertions,
            count_arrays_copied: options.enable_array_copy_counter,
            can_call_procedures: false,
            globals_memory_size: None,
        }
    }
}

/// Special brillig context to codegen global values initialization
impl<F: AcirField + DebugToString> BrilligContext<F, GlobalSpace> {
    /// Create a [BrilligContext] with a [GlobalSpace] for memory allocations.
    pub(crate) fn new_for_global_init(
        options: &BrilligOptions,
        entry_point: FunctionId,
    ) -> BrilligContext<F, GlobalSpace> {
        BrilligContext {
            obj: BrilligArtifact::default(),
            registers: Rc::new(RefCell::new(GlobalSpace::new(options.layout))),
            context_label: Label::globals_init(entry_point),
            current_section: 0,
            next_section: 1,
            debug_show: DebugShow::new(options.enable_debug_trace),
            enable_debug_assertions: options.enable_debug_assertions,
            count_arrays_copied: options.enable_array_copy_counter,
            can_call_procedures: false,
            globals_memory_size: None,
        }
    }

    /// Total size of the global memory space.
    pub(crate) fn global_space_size(&self) -> usize {
        // `GlobalSpace::start` is inclusive so we must add one to get the accurate total global memory size
        (self.registers().max_memory_address() + 1) - self.registers().start()
    }
}

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Adds a brillig instruction to the brillig byte code
    fn push_opcode(&mut self, opcode: BrilligOpcode<F>) {
        self.obj.push_opcode(opcode);
    }

    /// Sets a current call stack that the next pushed opcodes will be associated with.
    pub(crate) fn set_call_stack(&mut self, call_stack: CallStackId) {
        self.obj.set_call_stack(call_stack);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::vec;

    use acvm::acir::brillig::{
        BitSize, ForeignCallParam, ForeignCallResult, HeapVector, IntegerBitSize, MemoryAddress,
        ValueOrArray,
    };
    use acvm::brillig_vm::brillig::HeapValueType;
    use acvm::brillig_vm::{VM, VMStatus, offsets};
    use acvm::{BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement};

    use crate::brillig::brillig_ir::{BrilligBinaryOp, BrilligContext};
    use crate::brillig::{BrilligOptions, assert_u32, assert_usize, brillig_gen::gen_brillig_for};
    use crate::ssa::ir::function::FunctionId;
    use crate::ssa::ssa_gen::Ssa;

    use super::artifact::{BrilligParameter, GeneratedBrillig, Label, LabelType};
    use super::procedures::compile_procedure;
    use super::registers::Stack;
    use super::{BrilligOpcode, ReservedRegisters};

    pub(crate) struct DummyBlackBoxSolver;

    impl BlackBoxFunctionSolver<FieldElement> for DummyBlackBoxSolver {
        fn pedantic_solving(&self) -> bool {
            true
        }

        fn multi_scalar_mul(
            &self,
            _points: &[FieldElement],
            _scalars_lo: &[FieldElement],
            _scalars_hi: &[FieldElement],
            _predicate: bool,
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
            _predicate: bool,
        ) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
            panic!("Path not trodden by this test")
        }

        fn poseidon2_permutation(
            &self,
            _inputs: &[FieldElement],
        ) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
            Ok(vec![0_u128.into(), 1_u128.into(), 2_u128.into(), 3_u128.into()])
        }
    }

    pub(crate) fn create_context(id: FunctionId) -> BrilligContext<FieldElement, Stack> {
        let options = BrilligOptions {
            enable_debug_trace: true,
            enable_debug_assertions: true,
            enable_array_copy_counter: false,
            ..Default::default()
        };
        let mut context = BrilligContext::new("test", &options);
        context.enter_context(Label::function(id));
        context
    }

    pub(crate) fn create_entry_point_bytecode(
        context: BrilligContext<FieldElement, Stack>,
        arguments: Vec<BrilligParameter>,
        returns: Vec<BrilligParameter>,
    ) -> GeneratedBrillig<FieldElement> {
        let options = BrilligOptions {
            enable_debug_trace: false,
            enable_debug_assertions: context.enable_debug_assertions,
            enable_array_copy_counter: context.count_arrays_copied,
            ..Default::default()
        };
        let artifact = context.into_artifact();
        let (mut entry_point_artifact, stack_start) = BrilligContext::new_entry_point_artifact(
            arguments,
            returns,
            FunctionId::test_new(0),
            false,
            0,
            "entry_point",
            &options,
        );
        entry_point_artifact.link_with(&artifact);
        while let Some(unresolved_fn_label) = entry_point_artifact.first_unresolved_function_call()
        {
            let LabelType::Procedure(procedure_id) = unresolved_fn_label.label_type else {
                panic!("Test functions cannot be linked with other functions");
            };
            let procedure_artifact = compile_procedure(procedure_id, &options, stack_start);
            entry_point_artifact.link_with(&procedure_artifact);
        }
        entry_point_artifact.finish()
    }

    pub(crate) fn create_and_run_vm(
        calldata: Vec<FieldElement>,
        bytecode: &[BrilligOpcode<FieldElement>],
    ) -> (VM<'_, FieldElement, DummyBlackBoxSolver>, usize, usize) {
        let profiling_active = false;
        let mut vm = VM::new(calldata, bytecode, &DummyBlackBoxSolver, profiling_active, None);

        let status = vm.process_opcodes();
        if let VMStatus::Finished { return_data_offset, return_data_size } = status {
            (vm, assert_usize(return_data_offset), assert_usize(return_data_size))
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

        // Enable debug trace so we can see what the bytecode is if the test fails.
        let options = BrilligOptions {
            enable_debug_trace: true,
            enable_debug_assertions: true,
            enable_array_copy_counter: false,
            show_opcode_advisories: false,
            layout: Default::default(),
        };
        let mut context = BrilligContext::new("test", &options);

        // Allocate variables
        let r_input_size = MemoryAddress::direct(assert_u32(ReservedRegisters::len()));
        let r_output_ptr = r_input_size.offset(1);
        let r_output_size = r_input_size.offset(2);
        let r_equality = r_input_size.offset(3);

        let r_free = ReservedRegisters::free_memory_pointer();
        // Set the free memory pointer after the variables allocated above.
        let r_free_value = ReservedRegisters::len() + 4;
        context.usize_const_instruction(r_free, FieldElement::from(r_free_value));

        context.usize_const_instruction(r_input_size, FieldElement::from(12_usize));
        // The output pointer points at the heap.
        context.usize_const_instruction(
            r_output_ptr,
            FieldElement::from(r_free_value + assert_usize(offsets::VECTOR_ITEMS)),
        );
        context.foreign_call_instruction(
            "make_number_sequence".into(),
            &[ValueOrArray::MemoryAddress(r_input_size)],
            &[HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U32))],
            &[ValueOrArray::HeapVector(HeapVector { pointer: r_output_ptr, size: r_output_size })],
            &[HeapValueType::Vector {
                value_types: vec![HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U32))],
            }],
        );

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
        context.push_opcode(BrilligOpcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        });
        // If we got the expected number of items, jump to the STOP, otherwise fall through to TRAP.
        context.push_opcode(BrilligOpcode::JumpIf { condition: r_equality, location: 8 });
        let empty_data =
            HeapVector { pointer: MemoryAddress::direct(0), size: MemoryAddress::direct(0) };
        context.push_opcode(BrilligOpcode::Trap { revert_data: empty_data });
        context.stop_instruction(empty_data);

        let bytecode: Vec<BrilligOpcode<FieldElement>> = context.into_artifact().finish().byte_code;

        let mut vm = VM::new(vec![], &bytecode, &DummyBlackBoxSolver, false, None);

        // Run the VM up to the foreign call. Assert the expected call parameters.
        let status = vm.process_opcodes();
        assert_eq!(
            status,
            VMStatus::ForeignCallWait {
                function: "make_number_sequence".to_string(),
                inputs: vec![ForeignCallParam::Single(FieldElement::from(12u128))]
            }
        );
        // Create the response.
        let number_sequence: Vec<FieldElement> =
            (0_usize..12_usize).map(FieldElement::from).collect();
        let response = ForeignCallResult { values: vec![ForeignCallParam::Array(number_sequence)] };
        vm.resolve_foreign_call(response);

        // The equality check should succeed
        let status = vm.process_opcodes();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });
    }

    /// Test demonstrating that `initialize_constant_array_runtime` is protected from
    /// iterator overflow by the VM's allocation overflow check (`process_free_memory_op`).
    ///
    /// `codegen_for_loop` uses wrapping arithmetic for iterator increments. If the iterator
    /// could overflow before reaching the bound, it would cause an infinite loop.
    /// However, this cannot happen through `initialize_constant_array_runtime` because:
    ///
    /// 1. Array allocation adds to `free_memory_pointer` via `BinaryIntOp::Add`
    /// 2. VM's `process_free_memory_op` intercepts adds to register 1 and uses `checked_add`
    /// 3. If allocation would overflow, VM fails with "Out of memory" before the loop runs
    ///
    /// This test compiles SSA with a large constant array, then patches `free_memory_pointer`
    /// to near u32::MAX to demonstrate that allocation triggers "Out of memory".
    #[test]
    fn initialize_constant_array_protected_by_allocation_check() {
        // SSA with array of 11 identical tuples - triggers initialize_constant_array_runtime
        let src = r#"
            brillig(inline) predicate_pure fn main f0 {
              b0(v0: u32):
                v1 = make_array [
                    u32 42, u32 42, u32 42,
                    u32 42, u32 42, u32 42,
                    u32 42, u32 42, u32 42,
                    u32 42, u32 42, u32 42,
                    u32 42, u32 42, u32 42,
                    u32 42, u32 42, u32 42,
                    u32 42, u32 42, u32 42,
                    u32 42, u32 42, u32 42,
                    u32 42, u32 42, u32 42,
                    u32 42, u32 42, u32 42,
                    u32 42, u32 42, u32 42
                ] : [(u32, u32, u32); 11]
                return
            }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();
        let options = BrilligOptions::default();
        let brillig = ssa.to_brillig(&options);
        let args = vec![BrilligParameter::SingleAddr(32)];
        let mut generated = gen_brillig_for(main, args, &brillig, &options).unwrap();

        // Find the first BinaryIntOp::Add that writes to the free_memory_pointer (FMP)
        // and insert a patch just before it.
        // We patch the opcode rather than using the memory layout config because if we indicate a heap start
        // near u32::MAX we will fail with a trap in the CheckMaxStackDepth procedure.
        // Patching the initialization of the FMP lets us simulate prior allocations exhausting memory without having
        // to actually allocate GBs of memory for this test.
        let mut insert_pos = None;
        for (i, opcode) in generated.byte_code.iter().enumerate() {
            if let BrilligOpcode::BinaryIntOp { destination, op, .. } = opcode {
                use acvm::acir::brillig::BinaryIntOp;
                if *destination == ReservedRegisters::free_memory_pointer()
                    && *op == BinaryIntOp::Add
                {
                    insert_pos = Some(i);
                    break;
                }
            }
        }
        let insert_pos = insert_pos.expect("Should find BinaryIntOp::Add to FMP");

        // Insert opcode to set free_memory_pointer near u32::MAX
        // This simulates what would happen if prior allocations exhausted memory
        let patch_opcode = BrilligOpcode::Const {
            destination: ReservedRegisters::free_memory_pointer(), // FREE_MEMORY_POINTER_ADDRESS
            value: FieldElement::from(u32::MAX - 10), // Near max, allocation will overflow
            bit_size: BitSize::Integer(IntegerBitSize::U32),
        };
        generated.byte_code.insert(insert_pos, patch_opcode);

        let mut vm = VM::new(
            vec![FieldElement::from(0u64)],
            &generated.byte_code,
            &DummyBlackBoxSolver,
            false,
            None,
        );

        let status = vm.process_opcodes();

        let VMStatus::Failure {
            reason: acvm::brillig_vm::FailureReason::RuntimeError { message },
            ..
        } = status
        else {
            panic!("Expected 'Out of memory' error from allocation overflow, got: {:?}", status)
        };
        assert!(message.contains("Out of memory"), "Expected 'Out of memory', got: {}", message);
    }
}
