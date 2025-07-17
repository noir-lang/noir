use std::vec;

use acvm::{
    AcirField,
    acir::brillig::{BitSize, HeapValueType, IntegerBitSize, MemoryAddress, ValueOrArray},
};

use super::ProcedureId;
use crate::brillig::{
    BrilligVariable,
    brillig_ir::{
        BRILLIG_MEMORY_ADDRESSING_BIT_SIZE, BrilligBinaryOp, BrilligContext, ReservedRegisters,
        brillig_variable::{BrilligArray, SingleAddrVariable},
        debug_show::DebugToString,
        registers::{RegisterAllocator, ScratchSpace},
    },
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Conditionally copies a source array to a destination array.
    /// If the reference count of the source array is 1, then we can directly copy the pointer of the source array to the destination array.
    pub(crate) fn call_array_copy_procedure(
        &mut self,
        source_array: BrilligArray,
        destination_array: BrilligArray,
    ) {
        let source_array_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
        let source_array_memory_size_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
        let new_array_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 2);

        self.mov_instruction(source_array_pointer_arg, source_array.pointer);
        self.usize_const_instruction(source_array_memory_size_arg, (source_array.size + 1).into());

        self.add_procedure_call_instruction(ProcedureId::ArrayCopy);

        self.mov_instruction(destination_array.pointer, new_array_pointer_return);
    }
}

pub(super) fn compile_array_copy_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let source_array_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
    let source_array_memory_size_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
    let new_array_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 2);

    brillig_context.set_allocated_registers(vec![
        source_array_pointer_arg,
        source_array_memory_size_arg,
        new_array_pointer_return,
    ]);

    let rc = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.load_instruction(rc.address, source_array_pointer_arg);

    let is_rc_one = SingleAddrVariable::new(brillig_context.allocate_register(), 1);
    brillig_context.codegen_usize_op(rc.address, is_rc_one.address, BrilligBinaryOp::Equals, 1);

    brillig_context.codegen_branch(is_rc_one.address, |ctx, cond| {
        if cond {
            // Reference count is 1, we can mutate the array directly
            ctx.mov_instruction(new_array_pointer_return, source_array_pointer_arg);
        } else {
            // First issue a array copy to the destination
            ctx.codegen_allocate_mem(new_array_pointer_return, source_array_memory_size_arg);

            ctx.codegen_mem_copy(
                source_array_pointer_arg,
                new_array_pointer_return,
                SingleAddrVariable::new_usize(source_array_memory_size_arg),
            );
            // Then set the new rc to 1
            ctx.indirect_const_instruction(
                new_array_pointer_return,
                BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
                1_usize.into(),
            );

            // Decrease the original ref count now that this copy is no longer pointing to it
            ctx.codegen_usize_op(rc.address, rc.address, BrilligBinaryOp::Sub, 1);

            // Increase our array copy counter if that flag is set
            if ctx.count_arrays_copied {
                ctx.codegen_increment_array_copy_counter();
            }
        }
    });
}

/// The message to print when copying an array.
const ARRAY_COPY_COUNTER_MESSAGE: &str = "Total arrays copied: {}";

/// The metadata string needed to tell `print` to print out a u32
const PRINT_U32_TYPE_STRING: &str = "{\"kind\":\"unsignedinteger\",\"width\":32}";
// "{\"kind\":\"array\",\"length\":2,\"type\":{\"kind\":\"unsignedinteger\",\"width\":32}}";

// Create and return the string `PRINT_U32_TYPE_STRING`
fn literal_string_to_value<F: AcirField + DebugToString, Registers: RegisterAllocator>(
    target: &str,
    brillig_context: &mut BrilligContext<F, Registers>,
) -> ValueOrArray {
    let brillig_array =
        BrilligArray { pointer: brillig_context.allocate_register(), size: target.len() };

    brillig_context.codegen_initialize_array(brillig_array);

    let items_pointer = brillig_context.codegen_make_array_items_pointer(brillig_array);

    initialize_constant_string(brillig_context, target, items_pointer);
    brillig_context.deallocate_register(items_pointer);

    ValueOrArray::HeapArray(acvm::acir::brillig::HeapArray {
        pointer: brillig_context.codegen_make_array_items_pointer(brillig_array),
        size: brillig_array.size,
    })
}

// This function was adapted from `initialize_constant_array_comptime`
fn initialize_constant_string<F: AcirField + DebugToString, Registers: RegisterAllocator>(
    brillig_context: &mut BrilligContext<F, Registers>,
    data: &str,
    pointer: MemoryAddress,
) {
    // Allocate a register for the iterator
    let write_pointer_register = brillig_context.allocate_register();
    brillig_context.mov_instruction(write_pointer_register, pointer);

    for (element_idx, byte) in data.bytes().enumerate() {
        let byte_field = AcirField::from_le_bytes_reduce(&[byte]);
        // Store the item in memory
        brillig_context.indirect_const_instruction(write_pointer_register, 32, byte_field);

        if element_idx != data.len() - 1 {
            // Increment the write_pointer_register
            brillig_context.memory_op_instruction(
                write_pointer_register,
                ReservedRegisters::usize_one(),
                write_pointer_register,
                BrilligBinaryOp::Add,
            );
        }
    }

    brillig_context.deallocate_register(write_pointer_register);
}

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// emit: `println(f"Total arrays copied: {array_copy_counter}")`
    pub(crate) fn emit_println_of_array_copy_counter(&mut self) {
        let array_copy_counter = BrilligVariable::SingleAddr(SingleAddrVariable {
            address: self.array_copy_counter_address(),
            bit_size: 32,
        });

        let newline = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());
        let message = literal_string_to_value(ARRAY_COPY_COUNTER_MESSAGE, self);
        let item_count = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());
        let value_to_print = ValueOrArray::MemoryAddress(array_copy_counter.extract_register());
        let type_string_metadata = literal_string_to_value(PRINT_U32_TYPE_STRING, self);
        let is_fmt_string = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());

        let inputs = [
            newline, // true
            message,
            item_count,     // 1
            value_to_print, // array clone counter
            type_string_metadata,
            is_fmt_string, // true
        ];

        let u1_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U1));
        let u8_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U8));
        let u32_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U32));

        let newline_type = u1_type.clone();
        let size = ARRAY_COPY_COUNTER_MESSAGE.len();
        let msg_type = HeapValueType::Array { value_types: vec![u8_type.clone()], size };
        let item_count_type = HeapValueType::field();
        let value_to_print_type = u32_type;
        let size = PRINT_U32_TYPE_STRING.len();
        let metadata_type = HeapValueType::Array { value_types: vec![u8_type], size };
        let is_fmt_string_type = u1_type;

        let input_types = [
            newline_type,
            msg_type,
            item_count_type,
            value_to_print_type,
            metadata_type,
            is_fmt_string_type,
        ];

        self.foreign_call_instruction("print".to_string(), &inputs, &input_types, &[], &[]);
    }
}
