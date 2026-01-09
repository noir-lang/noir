use std::vec;

use acvm::{
    AcirField,
    acir::brillig::{
        BitSize, HeapArray, HeapValueType, IntegerBitSize, MemoryAddress, ValueOrArray,
    },
    brillig_vm::offsets,
};

use super::ProcedureId;
use crate::brillig::{
    BrilligVariable,
    brillig_ir::{
        BrilligContext, ReservedRegisters, assert_u32, assert_usize,
        brillig_variable::{BrilligArray, SingleAddrVariable},
        debug_show::DebugToString,
        registers::{Allocated, RegisterAllocator, ScratchSpace},
    },
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Call [ProcedureId::ArrayCopy].
    ///
    /// Conditionally copies a source array to a destination array.
    /// If the reference count of the source array is 1, then we can directly copy the pointer of the source array to the destination array.
    /// Otherwise a copy is made, and the ref-count of the original is decreased by 1.
    pub(crate) fn call_array_copy_procedure(
        &mut self,
        source_array: BrilligArray,
        destination_array: BrilligArray,
    ) {
        let [
            source_array_pointer_arg,
            source_array_memory_size_arg,
            destination_array_pointer_return,
        ] = self.make_scratch_registers();

        self.mov_instruction(source_array_pointer_arg, source_array.pointer);
        self.usize_const_instruction(
            source_array_memory_size_arg,
            (source_array.size + assert_usize(offsets::ARRAY_META_COUNT)).into(),
        );

        self.add_procedure_call_instruction(ProcedureId::ArrayCopy);

        self.mov_instruction(destination_array.pointer, destination_array_pointer_return);
    }
}

/// Compile [ProcedureId::ArrayCopy].
pub(super) fn compile_array_copy_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let [source_array_pointer_arg, source_array_memory_size_arg, destination_array_pointer_return] =
        brillig_context.allocate_scratch_registers();

    let rc = brillig_context.codegen_read_rc(source_array_pointer_arg);

    let is_rc_one = brillig_context.codegen_usize_equals_one(*rc);

    brillig_context.codegen_branch(is_rc_one.address, |ctx, cond| {
        if cond {
            // Reference count is 1, we can mutate the array directly
            ctx.mov_instruction(destination_array_pointer_return, source_array_pointer_arg);
        } else {
            // We need to copy the array; allocate the required space on the heap.
            ctx.codegen_allocate_mem(
                destination_array_pointer_return,
                source_array_memory_size_arg,
            );

            // First issue an array copy to the destination.
            // This copies the whole data structure, including metadata.
            ctx.codegen_mem_copy(
                source_array_pointer_arg,
                destination_array_pointer_return,
                SingleAddrVariable::new_usize(source_array_memory_size_arg),
            );
            // Then set the new RC to 1.
            ctx.codegen_initialize_rc(destination_array_pointer_return, 1);

            // Decrease the original ref count now that this copy is no longer pointing to it.
            // Copying an array is a potential implicit side effect of setting an item by index through a mutable variable;
            // we won't end up with two handles to the array, so we can split the RC between the old and the new.
            ctx.codegen_decrement_rc(source_array_pointer_arg, rc.address);

            // Increase our array copy counter if that flag is set
            if ctx.count_arrays_copied {
                ctx.codegen_increment_array_copy_counter();
            }
        }
    });
}

/// The metadata string needed to tell `print` to print out a u32
const PRINT_U32_TYPE_STRING: &str = "{\"kind\":\"unsignedinteger\",\"width\":32}";
// "{\"kind\":\"array\",\"length\":2,\"type\":{\"kind\":\"unsignedinteger\",\"width\":32}}";

/// Create and return the string `PRINT_U32_TYPE_STRING`
fn literal_string_to_value<F: AcirField + DebugToString, Registers: RegisterAllocator>(
    data: &str,
    brillig_context: &mut BrilligContext<F, Registers>,
) -> Allocated<ValueOrArray, Registers> {
    let brillig_array = brillig_context.allocate_brillig_array(data.len());

    // Allocate space on the heap.
    brillig_context.codegen_initialize_array(*brillig_array);

    // Get a pointer to where the items start on the heap.
    let items_pointer = brillig_context.codegen_make_array_items_pointer(*brillig_array);

    // Copy the data into the array.
    initialize_constant_string(brillig_context, data, *items_pointer);

    // Wrap the pointer into a `HeapArray`. The `BrilligArray` is no longer needed.
    items_pointer.map(|pointer| {
        ValueOrArray::HeapArray(HeapArray {
            pointer,
            size: assert_u32(data.len()),
        })
    })
}

/// Generate opcodes to initialize the memory at `pointer` to the bytes in the `data` string.
///
/// This function was adapted from `initialize_constant_array_comptime`.
fn initialize_constant_string<F: AcirField + DebugToString, Registers: RegisterAllocator>(
    brillig_context: &mut BrilligContext<F, Registers>,
    data: &str,
    pointer: MemoryAddress,
) {
    // Allocate a register for the iterator
    let write_pointer_register = brillig_context.allocate_register();
    brillig_context.mov_instruction(*write_pointer_register, pointer);

    for (element_idx, byte) in data.bytes().enumerate() {
        let byte_field = AcirField::from_le_bytes_reduce(&[byte]);
        // Store the item in memory
        brillig_context.indirect_const_instruction(*write_pointer_register, 32, byte_field);

        if element_idx != data.len() - 1 {
            // Increment the write_pointer_register
            brillig_context.memory_op_inc_by_usize_one(*write_pointer_register);
        }
    }
}

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// emit: `println(f"Total arrays copied: {array_copy_counter}")`
    pub(crate) fn emit_println_of_array_copy_counter(&mut self) {
        let array_copy_counter = BrilligVariable::from(SingleAddrVariable {
            address: self.array_copy_counter_address(),
            bit_size: 32,
        });

        let newline = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());
        let message_with_func_name = format!("Total arrays copied in {}: {{}}", &self.name());
        let message = literal_string_to_value(&message_with_func_name, self);
        let item_count = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());
        let value_to_print = ValueOrArray::MemoryAddress(array_copy_counter.extract_register());
        let type_string_metadata = literal_string_to_value(PRINT_U32_TYPE_STRING, self);
        let is_fmt_string = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());

        let inputs = [
            newline, // true
            *message,
            item_count,     // 1
            value_to_print, // array clone counter
            *type_string_metadata,
            is_fmt_string, // true
        ];

        let u1_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U1));
        let u8_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U8));
        let u32_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U32));

        let newline_type = u1_type.clone();
        let size = message_with_func_name.len();
        let msg_type = HeapValueType::Array {
            value_types: vec![u8_type.clone()],
            size: assert_u32(size),
        };
        let item_count_type = HeapValueType::field();
        let value_to_print_type = u32_type;
        let size = PRINT_U32_TYPE_STRING.len();
        let metadata_type = HeapValueType::Array {
            value_types: vec![u8_type],
            size: assert_u32(size),
        };
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
