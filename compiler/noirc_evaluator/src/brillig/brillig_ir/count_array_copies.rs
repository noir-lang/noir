//! Per-site array and vector copy counting for the `--count-array-copies` debugging flag.
//!
//! When a [`CopySiteRegistry`] is attached to the [`BrilligContext`], the copy procedures emit
//! extra bytecode that tallies how many times each source location actually copies an
//! array/vector (as opposed to reusing it in place). The entry-point wrapper then prints a
//! summary of the totals once execution finishes. With no registry attached none of this
//! bytecode is emitted, so ordinary compilation is unaffected.

use std::sync::{Arc, Mutex};

use acvm::{
    AcirField,
    acir::brillig::{
        BitSize, HeapArray, HeapValueType, IntegerBitSize, MemoryAddress, ValueOrArray,
        lengths::{SemanticLength, SemiFlattenedLength},
    },
};
use noirc_errors::call_stack::{CallStackHelper, CallStackId};
use rustc_hash::FxHashMap;

use super::{
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
    brillig_variable::SingleAddrVariable,
    debug_show::DebugToString,
    registers::{Allocated, GlobalSpace, RegisterAllocator},
};
use crate::brillig::assert_u32;

/// Maximum number of distinct copy sites that are tracked per-site.
/// Sites beyond this limit are not tracked per-site (they still count toward the total).
pub(crate) const MAX_TRACK_SITES: usize = 256;

/// Number of per-site copy locations displayed at end of execution (top N by count).
pub(crate) const MAX_DISPLAY_SITES: usize = 25;

/// Inner state for [CopySiteRegistry], kept behind an `Arc<Mutex<…>>` so it can be
/// shared across all `BrilligContext` instances that are alive during one compilation.
#[derive(Debug, Default)]
struct CopySiteRegistryInner {
    /// Ordered list of call stack IDs (index = per-site counter slot).
    sites: Vec<CallStackId>,
    /// Map from `CallStackId` → slot index, used to deduplicate identical call sites.
    site_to_index: FxHashMap<CallStackId, usize>,
    /// Source-location labels resolved after compilation (set by `resolve_labels`).
    resolved_labels: Option<Vec<String>>,
}

/// Shared registry of array/vector copy sites, accumulated during Brillig compilation.
/// Each *unique* call site (identified by `CallStackId`) is assigned a sequential index
/// used to address its per-site counter in global memory.
/// Registering the same call site twice returns the same index.
/// After compilation, call `resolve_labels` to convert `CallStackId`s to readable strings.
#[derive(Clone, Debug, Default)]
pub struct CopySiteRegistry(Arc<Mutex<CopySiteRegistryInner>>);

impl CopySiteRegistry {
    /// Register a copy site for the given call stack location.
    /// Returns the site index (0-based).  Identical call sites share an index.
    pub(crate) fn register_site(&self, id: CallStackId) -> usize {
        let mut inner = self.0.lock().unwrap();
        if let Some(&idx) = inner.site_to_index.get(&id) {
            return idx;
        }
        let index = inner.sites.len();
        inner.site_to_index.insert(id, index);
        inner.sites.push(id);
        index
    }

    /// Resolve all registered `CallStackId`s to human-readable `"filename.nr:line"` strings.
    /// Must be called after Brillig compilation (once the `CallStackHelper` is fully populated)
    /// but before `get_resolved_labels` is used.
    pub(crate) fn resolve_labels(
        &self,
        call_stacks: &CallStackHelper,
        files: Option<&fm::FileManager>,
    ) {
        use fm::codespan_files::Files;
        let sites = self.0.lock().unwrap().sites.clone();
        let labels: Vec<String> = sites
            .iter()
            .map(|&id| {
                let stack = call_stacks.get_call_stack(id);
                let Some(location) = stack.last() else {
                    return "<unknown>".to_string();
                };
                let Some(fm) = files else {
                    return format!("<unknown>:{}", location.span.start());
                };
                let file_name = fm
                    .path(location.file)
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("<unknown>");
                let start_index = location.span.start() as usize;
                match fm.as_file_map().line_index(location.file, start_index) {
                    Ok(line_idx) => format!("{file_name}:{}", line_idx + 1),
                    Err(_) => file_name.to_string(),
                }
            })
            .collect();
        self.0.lock().unwrap().resolved_labels = Some(labels);
    }

    /// Return a snapshot of all resolved site labels in registration order.
    /// Returns an empty `Vec` if `resolve_labels` has not been called yet.
    pub(crate) fn get_resolved_labels(&self) -> Vec<String> {
        self.0.lock().unwrap().resolved_labels.clone().unwrap_or_default()
    }
}

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// True when per-site array copy counting is enabled, i.e. a [`CopySiteRegistry`] is attached.
    pub(crate) fn count_array_copies(&self) -> bool {
        self.copy_site_registry.is_some()
    }

    /// Returns the address of the implicit debug variable containing the count of
    /// implicitly copied arrays as a result of RC's copy on write semantics.
    pub(crate) fn array_copy_counter_address(&self) -> MemoryAddress {
        assert!(
            self.count_array_copies(),
            "the array copy counter is only allocated when `--count-array-copies` is set"
        );

        // The copy counter is always put in the first global slot
        MemoryAddress::direct(assert_u32(GlobalSpace::start_with_layout(&self.layout())))
    }

    /// Returns the global memory address of the per-site copy counter for `site_index`.
    /// Per-site counters occupy slots 1..=MAX_TRACK_SITES right after the total counter (slot 0).
    pub(crate) fn per_site_counter_address(&self, site_index: usize) -> MemoryAddress {
        assert!(
            site_index < MAX_TRACK_SITES,
            "site_index {site_index} exceeds MAX_TRACK_SITES {MAX_TRACK_SITES}"
        );
        MemoryAddress::direct(assert_u32(
            GlobalSpace::start_with_layout(&self.layout()) + 1 + site_index,
        ))
    }

    /// Registers the current location as a per-site copy site and returns the global memory
    /// address of its counter, or `None` when copy-counting is disabled, no registry is attached,
    /// or the site is beyond [`MAX_TRACK_SITES`].
    ///
    /// Sites are deduplicated by `CallStackId`, so the same call site compiled more than once
    /// shares a single counter.
    fn register_per_site_counter(&self) -> Option<MemoryAddress> {
        let registry = self.copy_site_registry.clone()?;
        let site_index = registry.register_site(self.current_call_stack_id());
        (site_index < MAX_TRACK_SITES).then(|| self.per_site_counter_address(site_index))
    }

    pub(crate) fn codegen_increment_array_copy_counter(&mut self) {
        let array_copy_counter = self.array_copy_counter_address();
        self.codegen_usize_op(array_copy_counter, array_copy_counter, BrilligBinaryOp::Add, 1);
    }

    /// If copy-counting is enabled, registers this as a per-site copy location and
    /// emits runtime code that increments the per-site counter whenever a copy actually occurred
    /// (i.e. `source_pointer != dest_pointer` after a copy procedure call).
    pub(crate) fn codegen_count_if_copy_occurred(
        &mut self,
        source_pointer: MemoryAddress,
        dest_pointer: MemoryAddress,
    ) {
        let Some(counter_addr) = self.register_per_site_counter() else {
            return;
        };

        // Emit: if source_pointer != dest_pointer { counter_addr += 1 }
        // We use: did_not_copy = (source == dest); if did_not_copy => skip increment
        let did_not_copy = self.allocate_single_addr_bool();
        self.memory_op_instruction(
            source_pointer,
            dest_pointer,
            did_not_copy.address,
            BrilligBinaryOp::Equals,
        );
        self.codegen_if_not(did_not_copy.address, |ctx| {
            ctx.codegen_usize_op(counter_addr, counter_addr, BrilligBinaryOp::Add, 1);
        });
    }

    /// Like `codegen_count_if_copy_occurred` but driven by an explicit boolean flag register
    /// rather than a pointer comparison. Registers this as a per-site copy location and emits
    /// runtime code that increments the per-site counter when `flag != 0`.
    pub(crate) fn codegen_count_if_nonzero(&mut self, flag: MemoryAddress) {
        let Some(counter_addr) = self.register_per_site_counter() else {
            return;
        };

        // if flag != 0 { counter_addr += 1 }
        self.codegen_if(flag, |ctx| {
            ctx.codegen_usize_op(counter_addr, counter_addr, BrilligBinaryOp::Add, 1);
        });
    }

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
        let message_len = message.len();
        let message_value = literal_string_to_value(message, self);
        let item_count = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());
        let value_to_print = ValueOrArray::MemoryAddress(value_addr);
        let type_string_metadata = literal_string_to_value(PRINT_U32_TYPE_STRING, self);
        let is_fmt_string = ValueOrArray::MemoryAddress(ReservedRegisters::usize_one());

        let inputs = [
            newline, // true
            *message_value,
            item_count,     // 1
            value_to_print, // the u32 counter value
            *type_string_metadata,
            is_fmt_string, // true
        ];

        let u1_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U1));
        let u8_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U8));
        let u32_type = HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U32));

        let newline_type = u1_type.clone();
        let size = SemanticLength(assert_u32(message_len));
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

/// The metadata string needed to tell `print` to print out a u32
const PRINT_U32_TYPE_STRING: &str = "{\"kind\":\"unsignedinteger\",\"width\":32}";

/// Allocate the string `data` as a Brillig array on the heap and return it wrapped as a
/// `HeapArray` value suitable for passing to a `print` foreign call.
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
