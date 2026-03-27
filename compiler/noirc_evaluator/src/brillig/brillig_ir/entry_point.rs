use itertools::Itertools;

use crate::{
    brillig::{BrilligOptions, assert_u32, brillig_ir::registers::Allocated},
    ssa::ir::function::FunctionId,
};

use super::{
    BrilligContext, ReservedRegisters,
    artifact::{BrilligArtifact, BrilligParameter},
    brillig_variable::{BrilligVariable, SingleAddrVariable},
    debug_show::DebugToString,
    registers::Stack,
};
use acvm::acir::{
    AcirField,
    brillig::{HeapVector, MemoryAddress, lengths::SemiFlattenedLength},
};

impl<F: AcirField + DebugToString> BrilligContext<F, Stack> {
    /// Creates an entry point artifact that will jump to the function label provided.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_entry_point_artifact(
        arguments: Vec<BrilligParameter>,
        return_parameters: Vec<BrilligParameter>,
        target_function: FunctionId,
        globals_init: bool,
        globals_memory_size: usize,
        name: &str,
        options: &BrilligOptions,
    ) -> (BrilligArtifact<F>, usize) {
        let mut context = BrilligContext::new(name, options);

        context.set_globals_memory_size(Some(globals_memory_size));

        let stack_start = context.codegen_entry_point(
            &arguments,
            &return_parameters,
            target_function,
            globals_init,
        );

        context.add_external_call_instruction(target_function);

        context.codegen_exit_point(&arguments, &return_parameters);
        (context.into_artifact(), stack_start)
    }

    fn calldata_start_offset(&self) -> usize {
        let globals_size =
            self.globals_memory_size.expect("The memory size of globals should be set");
        self.layout().entry_point_start(globals_size)
    }

    fn return_data_start_offset(&self, calldata_size: usize) -> usize {
        let globals_size =
            self.globals_memory_size.expect("The memory size of globals should be set");
        self.layout().return_data_start(globals_size, calldata_size)
    }

    /// Adds the instructions needed to handle entry point parameters and global initialization.
    /// The runtime will leave the parameters in calldata.
    /// Arrays will be passed flattened.
    ///
    /// Memory layout for entry points:
    /// {reserved} {scratch} {globals} {entry point (call data + return data)} {stack} {heap}
    ///
    /// Globals are initialized before calldata is copied so that they can temporarily use more memory
    /// than their final read-only maximum, without overwriting calldata.
    ///
    /// # Returns
    /// The start of the stack memory region. The start of the stack is determined by the globals compiled as well as
    /// the amount of call data and return data. We return this information so that the [max stack depth check][super::ProcedureId::CheckMaxStackDepth]
    /// can check against the appropriate constant represented the max stack pointer we can have in memory.
    fn codegen_entry_point(
        &mut self,
        arguments: &[BrilligParameter],
        return_parameters: &[BrilligParameter],
        target_function: FunctionId,
        globals_init: bool,
    ) -> usize {
        // We need to allocate the variable for every argument first so any register allocation doesn't mangle the expected order.
        let argument_variables = self.allocate_function_arguments(arguments);

        let calldata_size = Self::flattened_tuple_size(arguments);
        let return_data_size = Self::flattened_tuple_size(return_parameters);

        // Set reserved registers constants
        self.const_instruction(
            SingleAddrVariable::new_usize(ReservedRegisters::usize_one()),
            1_usize.into(),
        );

        let return_data_start = self.return_data_start_offset(calldata_size);

        // The stack begins after the calldata region (calldata + return data)
        let stack_start = return_data_start + return_data_size;

        // The heap begins right after the stack.
        // Per-function spill regions are allocated from the heap in each function's prologue.
        self.const_instruction(
            SingleAddrVariable::new_usize(ReservedRegisters::free_memory_pointer()),
            (stack_start + self.layout().max_stack_size()).into(),
        );

        // Set initial value of the stack pointer: `return_data_start + return_data_size`
        self.const_instruction(
            SingleAddrVariable::new_usize(ReservedRegisters::stack_pointer()),
            stack_start.into(),
        );

        // The initialization of globals if after the creation of the reserved registers,
        // so that things such as the allocation of arrays can use the _free memory pointer_
        // (because only the pointers to the arrays will live in the global space, with the
        // content still residing on the heap).
        if globals_init {
            self.add_globals_init_instruction(target_function);
        }

        // Copy calldata.
        // Happens after global initialization to avoid any potential overwrite temporary registers during global init.
        self.copy_and_cast_calldata(arguments);

        let mut current_calldata_pointer = self.calldata_start_offset();

        // Initialize the variables with the calldata
        for (argument_variable, argument) in argument_variables.iter().zip_eq(arguments) {
            match (**argument_variable, argument) {
                (BrilligVariable::SingleAddr(single_address), BrilligParameter::SingleAddr(_)) => {
                    self.mov_instruction(
                        single_address.address,
                        MemoryAddress::direct(assert_u32(current_calldata_pointer)),
                    );
                }
                (BrilligVariable::BrilligArray(array), BrilligParameter::Array(_, _)) => {
                    // Arrays are now fully flat — allocate and copy flat data directly.
                    self.codegen_initialize_array(array);
                    let items_pointer = self.codegen_make_array_items_pointer(array);
                    let size_var = self.make_usize_constant_instruction(array.size.0.into());
                    let calldata_pointer =
                        self.make_usize_constant_instruction(current_calldata_pointer.into());
                    self.codegen_mem_copy(calldata_pointer.address, *items_pointer, *size_var);
                }
                (BrilligVariable::BrilligVector(vector), BrilligParameter::Vector(_, _)) => {
                    // Vectors are also flat — initialize and copy.
                    let flattened_size = argument.flattened_size();
                    let size_var = self.make_usize_constant_instruction(flattened_size.into());
                    self.codegen_initialize_vector(vector, *size_var, None);
                    let items_pointer = self.codegen_make_vector_items_pointer(vector);
                    let calldata_pointer =
                        self.make_usize_constant_instruction(current_calldata_pointer.into());
                    self.codegen_mem_copy(calldata_pointer.address, *items_pointer, *size_var);
                }
                _ => unreachable!("ICE: cannot match variables against arguments"),
            }

            current_calldata_pointer += argument.flattened_size();
        }

        stack_start
    }

    fn allocate_function_arguments(
        &self,
        arguments: &[BrilligParameter],
    ) -> Vec<Allocated<BrilligVariable, Stack>> {
        arguments
            .iter()
            .map(|argument| match argument {
                BrilligParameter::SingleAddr(bit_size) => {
                    self.allocate_single_addr(*bit_size).map(BrilligVariable::from)
                }
                BrilligParameter::Array(items, size) => {
                    // Use fully-flattened size: sum of flattened sizes of all item types * element count
                    let flat_per_element: u32 =
                        items.iter().map(|p| p.flattened_size() as u32).sum();
                    let flattened_size = SemiFlattenedLength(flat_per_element * size.0);

                    self.allocate_brillig_array(flattened_size).map(BrilligVariable::from)
                }
                BrilligParameter::Vector(_, _) => {
                    self.allocate_brillig_vector().map(BrilligVariable::from)
                }
            })
            .collect()
    }

    fn copy_and_cast_calldata(&mut self, arguments: &[BrilligParameter]) {
        let calldata_size = Self::flattened_tuple_size(arguments);

        self.calldata_copy_instruction(
            MemoryAddress::direct(assert_u32(self.calldata_start_offset())),
            calldata_size,
            0,
        );

        fn flat_bit_sizes(param: &BrilligParameter) -> Box<dyn Iterator<Item = u32> + '_> {
            match param {
                BrilligParameter::SingleAddr(bit_size) => Box::new(std::iter::once(*bit_size)),
                BrilligParameter::Array(item_types, item_count)
                | BrilligParameter::Vector(item_types, item_count) => Box::new(
                    (0..item_count.0).flat_map(move |_| item_types.iter().flat_map(flat_bit_sizes)),
                ),
            }
        }

        for (i, bit_size) in arguments.iter().flat_map(flat_bit_sizes).enumerate() {
            // cSpell:disable-next-line
            // Calldatacopy tags everything with field type, so when downcast when necessary
            if bit_size < F::max_num_bits() {
                self.cast_instruction(
                    SingleAddrVariable::new(
                        MemoryAddress::direct(assert_u32(self.calldata_start_offset() + i)),
                        bit_size,
                    ),
                    SingleAddrVariable::new_field(MemoryAddress::direct(assert_u32(
                        self.calldata_start_offset() + i,
                    ))),
                );
            }
        }
    }

    /// Adds the instructions needed to handle return parameters.
    /// The runtime expects the results in a contiguous memory region.
    /// Arrays are already fully flat, so we just copy the flat data directly.
    fn codegen_exit_point(
        &mut self,
        arguments: &[BrilligParameter],
        return_parameters: &[BrilligParameter],
    ) {
        // First, we allocate the registers that hold the returned variables from the function call.
        self.set_allocated_registers(vec![]);

        let returned_variables: Vec<_> = return_parameters
            .iter()
            .map(|return_parameter| match return_parameter {
                BrilligParameter::SingleAddr(bit_size) => {
                    self.allocate_single_addr(*bit_size).map(BrilligVariable::from)
                }
                BrilligParameter::Array(item_types, item_count) => {
                    let flat_per_element: u32 =
                        item_types.iter().map(|p| p.flattened_size() as u32).sum();
                    let flattened_size = SemiFlattenedLength(flat_per_element * item_count.0);
                    self.allocate_brillig_array(flattened_size).map(BrilligVariable::from)
                }
                BrilligParameter::Vector(..) => unreachable!("ICE: Cannot return vectors"),
            })
            .collect();

        let calldata_size = Self::flattened_tuple_size(arguments);
        let return_data_size = Self::flattened_tuple_size(return_parameters);

        // Return data has a reserved space after calldata
        let return_data_offset = self.return_data_start_offset(calldata_size);
        let mut return_data_index = return_data_offset;

        for (return_param, returned_variable) in
            return_parameters.iter().zip_eq(&returned_variables)
        {
            match return_param {
                BrilligParameter::SingleAddr(_) => {
                    self.mov_instruction(
                        MemoryAddress::direct(assert_u32(return_data_index)),
                        returned_variable.extract_single_addr().address,
                    );
                }
                BrilligParameter::Array(_, _) => {
                    // Arrays are flat — directly copy from items region to return data.
                    let array = returned_variable.extract_array();
                    let items_pointer = self.codegen_make_array_items_pointer(array);
                    let size_var = self.make_usize_constant_instruction(array.size.0.into());
                    let pointer_to_return_data =
                        self.make_usize_constant_instruction(return_data_index.into());
                    self.codegen_mem_copy(
                        *items_pointer,
                        pointer_to_return_data.address,
                        *size_var,
                    );
                }
                BrilligParameter::Vector(..) => {
                    unreachable!("ICE: Cannot return vectors from brillig entrypoints")
                }
            }

            return_data_index += return_param.flattened_size();
        }

        let return_pointer = self.make_usize_constant_instruction(return_data_offset.into());
        let return_size = self.make_usize_constant_instruction(return_data_size.into());
        let return_data = HeapVector { pointer: return_pointer.address, size: return_size.address };

        self.stop_instruction(return_data);
    }
}

#[cfg(test)]
mod tests {

    use acvm::{
        FieldElement,
        acir::brillig::lengths::{SemanticLength, SemiFlattenedLength},
    };

    use crate::{
        brillig::brillig_ir::{
            brillig_variable::{BrilligArray, BrilligVariable, SingleAddrVariable},
            entry_point::BrilligParameter,
            tests::{create_and_run_vm, create_context, create_entry_point_bytecode},
        },
        ssa::ir::function::FunctionId,
    };

    #[test]
    fn entry_point_with_nested_array_parameter() {
        // With flat arrays, a [[u8; 2], u8; 2] is stored as 6 contiguous scalars.
        // The BrilligParameter still describes the logical structure but the data
        // is fully flattened: [1, 2, 3, 4, 5, 6].
        let calldata = vec![
            FieldElement::from(1_usize),
            FieldElement::from(2_usize),
            FieldElement::from(3_usize),
            FieldElement::from(4_usize),
            FieldElement::from(5_usize),
            FieldElement::from(6_usize),
        ];
        let arguments = vec![BrilligParameter::Array(
            vec![
                BrilligParameter::Array(vec![BrilligParameter::SingleAddr(8)], SemanticLength(2)),
                BrilligParameter::SingleAddr(8),
            ],
            SemanticLength(2),
        )];
        let returns = vec![BrilligParameter::SingleAddr(8)];

        let mut context = create_context(FunctionId::test_new(0));

        // Allocate the parameter — fully flattened size is 6 (2 * (2 + 1))
        let array_pointer = context.allocate_register();
        let array_value = context.allocate_register();

        let items_pointer = context.codegen_make_array_items_pointer(BrilligArray {
            pointer: *array_pointer,
            size: SemiFlattenedLength(6),
        });

        // With flat arrays, the first element is directly at items_pointer[0].
        context.load_instruction(*array_value, *items_pointer);

        let return_value = BrilligVariable::from(SingleAddrVariable::new_usize(*array_value));
        context.codegen_return(&[return_value]);

        let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
        let (vm, return_data_offset, return_data_size) = create_and_run_vm(calldata, &bytecode);
        assert_eq!(return_data_size, 1, "Return data size is incorrect");
        assert_eq!(vm.get_memory()[return_data_offset].to_field(), FieldElement::from(1_usize));
    }

    #[test]
    fn entry_point_with_nested_array_return() {
        let flattened_array = vec![
            FieldElement::from(1_usize),
            FieldElement::from(2_usize),
            FieldElement::from(3_usize),
            FieldElement::from(4_usize),
            FieldElement::from(5_usize),
            FieldElement::from(6_usize),
        ];
        let array_param = BrilligParameter::Array(
            vec![
                BrilligParameter::Array(vec![BrilligParameter::SingleAddr(8)], SemanticLength(2)),
                BrilligParameter::SingleAddr(8),
            ],
            SemanticLength(2),
        );
        let arguments = vec![array_param.clone()];
        let returns = vec![array_param];

        let mut context = create_context(FunctionId::test_new(0));

        // Allocate the parameter — fully flattened size is 6 (2 * (2 + 1))
        let return_register = context.allocate_brillig_array(SemiFlattenedLength(6));

        context.codegen_return(&[return_register.to_var()]);

        let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
        let (vm, return_data_pointer, return_data_size) =
            create_and_run_vm(flattened_array.clone(), &bytecode);
        let memory = vm.get_memory();

        assert_eq!(
            memory[return_data_pointer..(return_data_pointer + flattened_array.len())]
                .iter()
                .map(|mem_val| mem_val.to_field())
                .collect::<Vec<_>>(),
            flattened_array
        );
        assert_eq!(return_data_size, flattened_array.len());
    }
}
