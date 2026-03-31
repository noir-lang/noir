use std::vec;

use acvm::{
    AcirField,
    acir::brillig::{
        BitSize, HeapArray, HeapValueType, IntegerBitSize, MemoryAddress, ValueOrArray,
        lengths::{SemanticLength, SemiFlattenedLength},
    },
    brillig_vm::offsets,
};

use super::ProcedureId;
use crate::brillig::{
    assert_u32,
    brillig_ir::{
        BrilligBinaryOp, BrilligContext, ReservedRegisters,
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
        debug_assert_eq!(
            source_array.size, destination_array.size,
            "ICE: source and destination arrays in copy must have the same size, but got {} and {}",
            source_array.size, destination_array.size
        );
        let [
            source_array_pointer_arg,
            source_array_memory_size_arg,
            destination_array_pointer_return,
        ] = self.make_scratch_registers();

        self.mov_instruction(source_array_pointer_arg, source_array.pointer);
        self.usize_const_instruction(
            source_array_memory_size_arg,
            (source_array.size.0 + offsets::ARRAY_META_COUNT).into(),
        );

        self.add_procedure_call_instruction(ProcedureId::ArrayCopy);

        self.mov_instruction(destination_array.pointer, destination_array_pointer_return);

        self.codegen_count_if_copy_occurred(source_array.pointer, destination_array.pointer);
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
    let brillig_array =
        brillig_context.allocate_brillig_array(SemiFlattenedLength(assert_u32(data.len())));

    // Allocate space on the heap.
    brillig_context.codegen_initialize_array(*brillig_array);

    // Get a pointer to where the items start on the heap.
    let items_pointer = brillig_context.codegen_make_array_items_pointer(*brillig_array);

    // Copy the data into the array.
    initialize_constant_string(brillig_context, data, *items_pointer);

    // Wrap the pointer into a `HeapArray`. The `BrilligArray` is no longer needed.
    let size = SemiFlattenedLength(assert_u32(data.len()));
    items_pointer.map(|pointer| ValueOrArray::HeapArray(HeapArray { pointer, size }))
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
    /// Emit print statements for the total array copy count, then for the top
    /// [`MAX_DISPLAY_SITES`] most-copied locations sorted descending by count.
    ///
    /// Uses a compile-time-unrolled selection sort: [`MAX_DISPLAY_SITES`] outer iterations, each
    /// running a runtime inner loop to find the maximum remaining counter, printing its label
    /// via a compile-time if-else chain, then zeroing that slot in a working heap buffer.
    ///
    /// Multiple internal tracking slots that resolve to the same source label are merged
    /// (their counts are summed) before sorting, so each unique source line appears once.
    pub(crate) fn emit_println_of_array_copy_counter(&mut self) {
        use crate::brillig::{MAX_DISPLAY_SITES, MAX_TRACK_SITES};

        // Print total.
        let total_addr = self.array_copy_counter_address();
        let total_msg = format!("Total arrays copied in {}: {{}}", self.name());
        self.emit_println_u32(&total_msg, total_addr);

        // Retrieve resolved labels; nothing more to do if there are none.
        let Some(registry) = self.copy_site_registry.clone() else {
            return;
        };
        let labels = registry.get_resolved_labels();
        if labels.is_empty() {
            return;
        }
        let n = labels.len().min(MAX_TRACK_SITES);

        // Merge tracking slots that share the same resolved label.
        // `dedup_labels[j]` is the unique label for slot j in the work buffer.
        // `dedup_groups[j]` lists the original per-site counter indices that contribute to slot j.
        let mut dedup_labels: Vec<&str> = Vec::new();
        let mut dedup_groups: Vec<Vec<usize>> = Vec::new();
        for (i, label) in labels[..n].iter().enumerate() {
            if let Some(pos) = dedup_labels.iter().position(|&l| l == label.as_str()) {
                dedup_groups[pos].push(i);
            } else {
                dedup_labels.push(label.as_str());
                dedup_groups.push(vec![i]);
            }
        }
        let m = dedup_labels.len(); // number of unique locations

        // Allocate a working heap buffer of M slots.
        // Each slot holds the merged (summed) count for one unique source location.
        let m_reg = self.make_usize_constant_instruction(F::from(m));
        let work_ptr = self.allocate_single_addr_usize();
        self.codegen_allocate_mem(work_ptr.address, m_reg.address);

        for (j, group) in dedup_groups.iter().enumerate() {
            // Sum all per-site counter values for this group into a temporary register.
            let sum = self.allocate_single_addr_usize();
            self.usize_const_instruction(sum.address, F::from(0_usize));
            for &idx in group {
                // counter_addr is a direct global address; mov copies the value directly.
                let counter_addr = self.per_site_counter_address(idx);
                let cur = self.allocate_single_addr_usize();
                self.mov_instruction(cur.address, counter_addr);
                self.memory_op_instruction(
                    sum.address,
                    cur.address,
                    sum.address,
                    BrilligBinaryOp::Add,
                );
            }
            let j_reg = self.make_usize_constant_instruction(F::from(j));
            self.codegen_store_with_offset(work_ptr.address, *j_reg, sum.address);
        }

        // Registers that persist across all MAX_DISPLAY_SITES outer iterations.
        let max_val = self.allocate_single_addr_usize();
        let max_idx = self.allocate_single_addr_usize();
        let bound_reg = self.make_usize_constant_instruction(F::from(m));
        let one_addr = ReservedRegisters::usize_one();

        for _ in 0..MAX_DISPLAY_SITES {
            // Initialise: max = working[0], max_idx = 0.
            self.load_instruction(max_val.address, work_ptr.address);
            self.usize_const_instruction(max_idx.address, F::from(0_usize));

            // Inner runtime loop: scan working[1..m] and track the maximum.
            let max_val_addr = max_val.address;
            let max_idx_addr = max_idx.address;
            let work_ptr_addr = work_ptr.address;
            self.codegen_for_loop(Some(one_addr), bound_reg.address, None, |ctx, i_var| {
                let cur_val = ctx.allocate_single_addr_usize();
                ctx.codegen_load_with_offset(work_ptr_addr, i_var, cur_val.address);
                let is_greater = ctx.allocate_single_addr_bool();
                // is_greater = (max_val < cur_val)
                ctx.memory_op_instruction(
                    max_val_addr,
                    cur_val.address,
                    is_greater.address,
                    BrilligBinaryOp::LessThan,
                );
                ctx.codegen_if(is_greater.address, |ctx| {
                    ctx.mov_instruction(max_val_addr, cur_val.address);
                    ctx.mov_instruction(max_idx_addr, i_var.address);
                });
            });

            // Skip if the maximum counter is zero (no more nonzero sites to display).
            let max_is_zero = self.allocate_single_addr_bool();
            self.codegen_usize_op(max_val.address, max_is_zero.address, BrilligBinaryOp::Equals, 0);
            let work_ptr_addr = work_ptr.address;
            self.codegen_if_not(max_is_zero.address, |ctx| {
                // Compile-time if-else chain: print the label for whichever slot holds the max.
                for (j, label) in dedup_labels.iter().enumerate() {
                    let is_match = ctx.allocate_single_addr_bool();
                    ctx.codegen_usize_op(
                        max_idx_addr,
                        is_match.address,
                        BrilligBinaryOp::Equals,
                        j,
                    );
                    ctx.codegen_if(is_match.address, |ctx| {
                        let msg = format!("  {label}: {{}}");
                        ctx.emit_println_u32(&msg, max_val_addr);
                    });
                }
                // Zero out the selected slot so it is not picked in subsequent iterations.
                let zero = ctx.make_usize_constant_instruction(F::from(0_usize));
                ctx.codegen_store_with_offset(
                    work_ptr_addr,
                    SingleAddrVariable::new_usize(max_idx_addr),
                    zero.address,
                );
            });
        }
    }

    /// Emit a `print` foreign call that prints `message` as a format string with one u32 substitution.
    fn emit_println_u32(&mut self, message: &str, value_addr: MemoryAddress) {
        let newline = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());
        let message_val = literal_string_to_value(message, self);
        let item_count = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());
        let value_to_print = ValueOrArray::MemoryAddress(value_addr);
        let type_string_metadata = literal_string_to_value(PRINT_U32_TYPE_STRING, self);
        let is_fmt_string = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());

        let inputs = [
            newline, // true
            *message_val,
            item_count,     // 1
            value_to_print, // the u32 counter value
            *type_string_metadata,
            is_fmt_string, // true
        ];

        let u1_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U1));
        let u8_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U8));
        let u32_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U32));

        let newline_type = u1_type.clone();
        let size = SemanticLength(assert_u32(message.len()));
        let msg_type = HeapValueType::Array { value_types: vec![u8_type.clone()], size };
        let item_count_type = HeapValueType::field();
        let value_to_print_type = u32_type;
        let size = SemanticLength(assert_u32(PRINT_U32_TYPE_STRING.len()));
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
