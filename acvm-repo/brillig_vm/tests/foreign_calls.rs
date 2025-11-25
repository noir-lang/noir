use acir::{
    AcirField, FieldElement,
    brillig::{
        BitSize, ForeignCallParam, ForeignCallResult, HeapArray, HeapValueType, HeapVector,
        IntegerBitSize, MemoryAddress, Opcode, ValueOrArray,
    },
};
use acvm_blackbox_solver::StubbedBlackBoxSolver;
use brillig_vm::{
    FREE_MEMORY_POINTER_ADDRESS, FailureReason, MEMORY_ADDRESSING_BIT_SIZE, Memory, MemoryValue,
    VM, VMStatus, offsets,
};

/// Set up for a foreign call test
///
/// # Returns
/// Tuple of (finished VM memory, internal VM foreign call counter)
fn run_foreign_call_test<F: AcirField>(
    calldata: Vec<F>,
    opcodes: &[Opcode<F>],
    expected_foreign_call_status: VMStatus<F>,
    foreign_call_result: Vec<ForeignCallParam<F>>,
    expected_final_status: VMStatus<F>,
) -> (Memory<F>, usize) {
    let solver = StubbedBlackBoxSolver::default();

    let mut vm = VM::new(calldata, opcodes, &solver, false, None);

    let status = vm.process_opcodes();
    assert_eq!(status, expected_foreign_call_status);

    vm.resolve_foreign_call(ForeignCallResult { values: foreign_call_result });
    let status = vm.process_opcode();
    assert_eq!(*status, expected_final_status);
    let counter = vm.foreign_call_counter();
    (vm.take_memory(), counter)
}

#[test]
fn foreign_call_opcode_simple_result() {
    let r_input = MemoryAddress::direct(0);
    let r_result = MemoryAddress::direct(1);

    let double_program = vec![
        // Load input address with value 5
        Opcode::Const {
            destination: r_input,
            value: (5u128).into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // Call foreign function "double" with the input address
        Opcode::ForeignCall {
            function: "double".into(),
            destinations: vec![ValueOrArray::MemoryAddress(r_result)],
            destination_value_types: vec![HeapValueType::Simple(BitSize::Integer(
                MEMORY_ADDRESSING_BIT_SIZE,
            ))],
            inputs: vec![ValueOrArray::MemoryAddress(r_input)],
            input_value_types: vec![HeapValueType::Simple(BitSize::Integer(
                MEMORY_ADDRESSING_BIT_SIZE,
            ))],
        },
    ];

    let (memory, foreign_call_counter) = run_foreign_call_test(
        vec![],
        &double_program,
        VMStatus::ForeignCallWait {
            function: "double".into(),
            inputs: vec![FieldElement::from(5usize).into()],
        },
        vec![FieldElement::from(10u128).into()],
        VMStatus::Finished { return_data_offset: 0, return_data_size: 0 },
    );

    // Check result address
    let result_value = memory.read(r_result);
    assert_eq!(result_value, (10u32).into());

    // Ensure the foreign call counter has been incremented
    assert_eq!(foreign_call_counter, 1);
}

#[test]
fn foreign_call_opcode_memory_result() {
    let r_input = MemoryAddress::direct(0);
    let r_output = MemoryAddress::direct(1);

    // Define a simple 2x2 matrix in memory
    let initial_matrix: Vec<FieldElement> =
        vec![(1u128).into(), (2u128).into(), (3u128).into(), (4u128).into()];

    // Transpose of the matrix (but arbitrary for this test, the 'correct value')
    let expected_result: Vec<FieldElement> =
        vec![(1u128).into(), (3u128).into(), (2u128).into(), (4u128).into()];

    let invert_program = vec![
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(initial_matrix.len() as u32),
        },
        Opcode::Const {
            destination: MemoryAddress::direct(1),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        Opcode::CalldataCopy {
            destination_address: MemoryAddress::direct(2),
            size_address: MemoryAddress::direct(0),
            offset_address: MemoryAddress::direct(1),
        },
        // input = 2
        Opcode::Const {
            destination: r_input,
            value: 2_usize.into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // output = 2
        Opcode::Const {
            destination: r_output,
            value: 2_usize.into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // *output = matrix_2x2_transpose(*input)
        Opcode::ForeignCall {
            function: "matrix_2x2_transpose".into(),
            destinations: vec![ValueOrArray::HeapArray(HeapArray {
                pointer: r_output,
                size: initial_matrix.len(),
            })],
            destination_value_types: vec![HeapValueType::Array {
                size: initial_matrix.len(),
                value_types: vec![HeapValueType::field()],
            }],
            inputs: vec![ValueOrArray::HeapArray(HeapArray {
                pointer: r_input,
                size: initial_matrix.len(),
            })],
            input_value_types: vec![HeapValueType::Array {
                value_types: vec![HeapValueType::field()],
                size: initial_matrix.len(),
            }],
        },
    ];

    let (memory, counter) = run_foreign_call_test(
        initial_matrix.clone(),
        &invert_program,
        VMStatus::ForeignCallWait {
            function: "matrix_2x2_transpose".into(),
            inputs: vec![initial_matrix.into()],
        },
        vec![expected_result.clone().into()],
        VMStatus::Finished { return_data_offset: 0, return_data_size: 0 },
    );

    // Check result in memory
    let result_values = memory.read_slice(MemoryAddress::direct(2), 4);
    let result_fields: Vec<_> = result_values.iter().map(|v| v.to_field()).collect();
    assert_eq!(result_fields, expected_result);

    // Ensure the foreign call counter has been incremented
    assert_eq!(counter, 1);
}

/// Calling a simple foreign call function that takes any string input, concatenates it with itself, and reverses the concatenation
#[test]
fn foreign_call_opcode_vector_input_and_output() {
    // Address 0 and 1 are reserved; using slots 2, 3, ... for variables.
    // The input is going to be a HeapVector; these register will hold the addresses of items and size on the heap.
    let r_input_pointer = MemoryAddress::direct(2);
    let r_input_size = MemoryAddress::direct(3);
    // The output is going to be a HeapVector as well, with registers pointing at addresses on the heap where the VM needs to write the items and the size.
    let r_output_pointer = MemoryAddress::direct(4);
    let r_output_size = MemoryAddress::direct(5);

    // Address where we copy the input data.
    let r_input_addr = MemoryAddress::direct(6);

    // Our first string to use the identity function with
    let input_string: Vec<FieldElement> =
        vec![(1u128).into(), (2u128).into(), (3u128).into(), (4u128).into()];
    // Double the string (concatenate it with itself)
    let mut output_string: Vec<_> =
        input_string.iter().cloned().chain(input_string.clone()).collect();
    // Reverse the concatenated string
    output_string.reverse();

    // The free memory starts where the input data ends.
    let free_memory_start = r_input_addr.to_usize() + input_string.len();
    let free_memory_start_addr = MemoryAddress::direct(free_memory_start);

    let string_double_program = vec![
        // @100 = length of the input
        Opcode::Const {
            destination: MemoryAddress::direct(100),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(input_string.len() as u32),
        },
        // @101 = offset of the input (an parameter for calldata copy)
        Opcode::Const {
            destination: MemoryAddress::direct(101),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        // copy the input string from calldata at offset 0 to memory
        Opcode::CalldataCopy {
            destination_address: r_input_addr,
            size_address: MemoryAddress::direct(100),
            offset_address: MemoryAddress::direct(101),
        },
        // input_pointer = input_addr
        Opcode::Const {
            destination: r_input_pointer,
            value: r_input_addr.to_usize().into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // input_size = input_string.len() (constant here, rather than a pointer into a vector structure)
        Opcode::Const {
            destination: r_input_size,
            value: input_string.len().into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // free_memory_pointer = input_addr + input_size
        Opcode::Const {
            destination: FREE_MEMORY_POINTER_ADDRESS,
            value: free_memory_start.into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // output_pointer = free_memory_pointer + 3
        Opcode::Const {
            destination: r_output_pointer,
            value: free_memory_start_addr.offset(offsets::VECTOR_ITEMS).to_usize().into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        Opcode::Const {
            destination: r_output_size,
            value: free_memory_start_addr.offset(offsets::VECTOR_SIZE).to_usize().into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // output_pointer[0..output_size] = string_double(input_pointer[0...input_size])
        Opcode::ForeignCall {
            function: "string_double".into(),
            destinations: vec![ValueOrArray::HeapVector(HeapVector {
                pointer: r_output_pointer,
                size: r_output_size,
            })],
            destination_value_types: vec![HeapValueType::Vector {
                value_types: vec![HeapValueType::field()],
            }],
            inputs: vec![ValueOrArray::HeapVector(HeapVector {
                pointer: r_input_pointer,
                size: r_input_size,
            })],
            input_value_types: vec![HeapValueType::Vector {
                value_types: vec![HeapValueType::field()],
            }],
        },
    ];

    let (memory, counter) = run_foreign_call_test(
        input_string.clone(),
        &string_double_program,
        VMStatus::ForeignCallWait {
            function: "string_double".into(),
            inputs: vec![input_string.clone().into()],
        },
        vec![ForeignCallParam::Array(output_string.clone())],
        VMStatus::Finished { return_data_offset: 0, return_data_size: 0 },
    );

    // Check result in memory: it should have been written to the free memory.
    let result_values: Vec<_> = memory
        .read_slice(
            MemoryAddress::direct(free_memory_start + offsets::VECTOR_ITEMS),
            output_string.len(),
        )
        .iter()
        .map(|mem_val| mem_val.clone().to_field())
        .collect();
    assert_eq!(result_values, output_string);

    // Check that the vector address has been updated.
    let vector_addr = free_memory_start_addr;

    // Check that the vector size is written onto the stack.
    let vector_size = memory.read(r_output_size);
    assert_eq!(vector_size.to_usize(), output_string.len());

    // The test above does not contain the opcodes that would copy the data from the stack to the heap.
    // Note that the VM did not write the size to the heap, because `codegen_brillig_vector_to_heap_vector`
    // allocates two variables on the stack that it uses as pointers to the heap, rather than returning the
    // heap addresses in the `HeapVector`. But, the VM only treats the `pointer` field as a pointer, ie.
    // it looks up the address stored at it, then writes to that address, but it writes directly to `size`.
    // Then, `codegen_initialize_externally_returned_vector` copies it to its final destination as part of
    // `codegen_initialize_vector_metadata`. We *could* give the heap address in `size`, but it would be
    // an exception to how `HeapVector`s generally look like. We could also use `write_ref` in the VM,
    // but that's not what the AVM does.
    let unset_size = memory.read(vector_addr.offset(offsets::VECTOR_SIZE));
    assert_eq!(unset_size, MemoryValue::Field(FieldElement::zero()));

    // Ensure the foreign call counter has been incremented
    assert_eq!(counter, 1);
}

#[test]
fn foreign_call_opcode_memory_alloc_result() {
    let r_input = MemoryAddress::direct(0);
    let r_output = MemoryAddress::direct(1);

    // Define a simple 2x2 matrix in memory
    let initial_matrix: Vec<FieldElement> =
        vec![(1u128).into(), (2u128).into(), (3u128).into(), (4u128).into()];

    // Transpose of the matrix (but arbitrary for this test, the 'correct value')
    let expected_result: Vec<FieldElement> =
        vec![(1u128).into(), (3u128).into(), (2u128).into(), (4u128).into()];

    let invert_program = vec![
        Opcode::Const {
            destination: MemoryAddress::direct(100),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(initial_matrix.len() as u32),
        },
        Opcode::Const {
            destination: MemoryAddress::direct(101),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        Opcode::CalldataCopy {
            destination_address: MemoryAddress::direct(2),
            size_address: MemoryAddress::direct(100),
            offset_address: MemoryAddress::direct(101),
        },
        // input = 2
        Opcode::Const {
            destination: r_input,
            value: (2u128).into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // output = 6
        Opcode::Const {
            destination: r_output,
            value: (6u128).into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // *output = matrix_2x2_transpose(*input)
        Opcode::ForeignCall {
            function: "matrix_2x2_transpose".into(),
            destinations: vec![ValueOrArray::HeapArray(HeapArray {
                pointer: r_output,
                size: initial_matrix.len(),
            })],
            destination_value_types: vec![HeapValueType::Array {
                size: initial_matrix.len(),
                value_types: vec![HeapValueType::field()],
            }],
            inputs: vec![ValueOrArray::HeapArray(HeapArray {
                pointer: r_input,
                size: initial_matrix.len(),
            })],
            input_value_types: vec![HeapValueType::Array {
                value_types: vec![HeapValueType::field()],
                size: initial_matrix.len(),
            }],
        },
    ];

    let (memory, counter) = run_foreign_call_test(
        initial_matrix.clone(),
        &invert_program,
        VMStatus::ForeignCallWait {
            function: "matrix_2x2_transpose".into(),
            inputs: vec![initial_matrix.clone().into()],
        },
        vec![expected_result.clone().into()],
        VMStatus::Finished { return_data_offset: 0, return_data_size: 0 },
    );

    // Check initial memory still in place
    let initial_values: Vec<_> = memory
        .read_slice(MemoryAddress::direct(2), 4)
        .iter()
        .map(|mem_val| mem_val.clone().to_field())
        .collect();
    assert_eq!(initial_values, initial_matrix);

    // Check result in memory
    let result_values: Vec<_> = memory
        .read_slice(MemoryAddress::direct(6), 4)
        .iter()
        .map(|mem_val| mem_val.clone().to_field())
        .collect();
    assert_eq!(result_values, expected_result);

    // Ensure the foreign call counter has been incremented
    assert_eq!(counter, 1);
}

#[test]
fn foreign_call_opcode_multiple_array_inputs_result() {
    let r_input_a = MemoryAddress::direct(0);
    let r_input_b = MemoryAddress::direct(1);
    let r_output = MemoryAddress::direct(2);

    // Define a simple 2x2 matrix in memory
    let matrix_a: Vec<FieldElement> =
        vec![(1u128).into(), (2u128).into(), (3u128).into(), (4u128).into()];

    let matrix_b: Vec<FieldElement> =
        vec![(10u128).into(), (11u128).into(), (12u128).into(), (13u128).into()];

    // Transpose of the matrix (but arbitrary for this test, the 'correct value')
    let expected_result: Vec<FieldElement> =
        vec![(34u128).into(), (37u128).into(), (78u128).into(), (85u128).into()];

    let matrix_mul_program = vec![
        Opcode::Const {
            destination: MemoryAddress::direct(100),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(matrix_a.len() + matrix_b.len()),
        },
        Opcode::Const {
            destination: MemoryAddress::direct(101),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        Opcode::CalldataCopy {
            destination_address: MemoryAddress::direct(3),
            size_address: MemoryAddress::direct(100),
            offset_address: MemoryAddress::direct(101),
        },
        // input = 3
        Opcode::Const {
            destination: r_input_a,
            value: (3u128).into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // input = 7
        Opcode::Const {
            destination: r_input_b,
            value: (7u128).into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // output = 0
        Opcode::Const {
            destination: r_output,
            value: (0u128).into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // *output = matrix_2x2_transpose(*input)
        Opcode::ForeignCall {
            function: "matrix_2x2_transpose".into(),
            destinations: vec![ValueOrArray::HeapArray(HeapArray {
                pointer: r_output,
                size: matrix_a.len(),
            })],
            destination_value_types: vec![HeapValueType::Array {
                size: matrix_a.len(),
                value_types: vec![HeapValueType::field()],
            }],
            inputs: vec![
                ValueOrArray::HeapArray(HeapArray { pointer: r_input_a, size: matrix_a.len() }),
                ValueOrArray::HeapArray(HeapArray { pointer: r_input_b, size: matrix_b.len() }),
            ],
            input_value_types: vec![
                HeapValueType::Array {
                    size: matrix_a.len(),
                    value_types: vec![HeapValueType::field()],
                },
                HeapValueType::Array {
                    size: matrix_b.len(),
                    value_types: vec![HeapValueType::field()],
                },
            ],
        },
    ];

    let mut initial_memory = matrix_a.clone();
    initial_memory.extend(matrix_b.clone());

    let (memory, counter) = run_foreign_call_test(
        initial_memory.clone(),
        &matrix_mul_program,
        VMStatus::ForeignCallWait {
            function: "matrix_2x2_transpose".into(),
            inputs: vec![matrix_a.into(), matrix_b.into()],
        },
        vec![expected_result.clone().into()],
        VMStatus::Finished { return_data_offset: 0, return_data_size: 0 },
    );

    // Check result in memory
    let result_values: Vec<_> = memory
        .read_slice(MemoryAddress::direct(0), 4)
        .iter()
        .map(|mem_val| mem_val.clone().to_field())
        .collect();
    assert_eq!(result_values, expected_result);

    // Ensure the foreign call counter has been incremented
    assert_eq!(counter, 1);
}

#[test]
fn foreign_call_opcode_nested_arrays_and_slices_input() {
    // This is the data we want to pass:
    // [(Field, [Field], [Field; 1]); 2]
    // [(1, <2,3>, [4]), (5, <6,7,8>, [9])]

    let v2: Vec<MemoryValue<FieldElement>> = vec![
        MemoryValue::new_field(FieldElement::from(2u128)),
        MemoryValue::new_field(FieldElement::from(3u128)),
    ];
    let a4: Vec<MemoryValue<FieldElement>> =
        vec![MemoryValue::new_field(FieldElement::from(4u128))];
    let v6: Vec<MemoryValue<FieldElement>> = vec![
        MemoryValue::new_field(FieldElement::from(6u128)),
        MemoryValue::new_field(FieldElement::from(7u128)),
        MemoryValue::new_field(FieldElement::from(8u128)),
    ];
    let a9: Vec<MemoryValue<FieldElement>> =
        vec![MemoryValue::new_field(FieldElement::from(9u128))];

    // construct memory by declaring all inner arrays/vectors first
    // Declare v2: [RC, size, capacity, ...items]
    let v2_ptr: usize = 0usize;
    let mut memory = vec![MemoryValue::from(1_u32), v2.len().into(), v2.len().into()];
    memory.extend(v2.clone());

    // Declare a4: [RC, ...items]
    let a4_ptr = memory.len();
    memory.extend(vec![MemoryValue::from(1_u32)]);
    memory.extend(a4.clone());

    // Declare v6: [RC, size, capacity, ...items]
    let v6_ptr = memory.len();
    memory.extend(vec![MemoryValue::from(1_u32), v6.len().into(), v6.len().into()]);
    memory.extend(v6.clone());

    // Declare a9: [RC, ...items]
    let a9_ptr = memory.len();
    memory.extend(vec![MemoryValue::from(1_u32)]);
    memory.extend(a9.clone());

    // finally we add the contents of the outer array
    // RC of the outer array
    memory.extend(vec![MemoryValue::from(1_u32)]);
    // Start of outer array items
    let outer_start = memory.len();
    let outer_array = vec![
        MemoryValue::new_field(FieldElement::from(1u128)),
        MemoryValue::from(v2.len() as u32), // semantic length
        MemoryValue::from(v2_ptr),
        MemoryValue::from(a4_ptr),
        MemoryValue::new_field(FieldElement::from(5u128)),
        MemoryValue::from(v6.len() as u32), // semantic length
        MemoryValue::from(v6_ptr),
        MemoryValue::from(a9_ptr),
    ];
    memory.extend(outer_array.clone());

    let input_array_value_types: Vec<HeapValueType> = vec![
        HeapValueType::field(),
        HeapValueType::Simple(BitSize::Integer(IntegerBitSize::U64)), // size of following vector
        HeapValueType::Vector { value_types: vec![HeapValueType::field()] },
        HeapValueType::Array { value_types: vec![HeapValueType::field()], size: 1 },
    ];

    // memory addresses for input and output
    let r_input = MemoryAddress::direct(memory.len());
    let r_output = MemoryAddress::direct(memory.len() + 1);

    let program: Vec<_> = vec![
        Opcode::Const {
            destination: MemoryAddress::direct(100),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(memory.len()),
        },
        Opcode::Const {
            destination: MemoryAddress::direct(101),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        Opcode::CalldataCopy {
            destination_address: MemoryAddress::direct(0),
            size_address: MemoryAddress::direct(100),
            offset_address: MemoryAddress::direct(101),
        },
    ]
    .into_iter()
    .chain(memory.iter().enumerate().map(|(index, mem_value)| Opcode::Cast {
        destination: MemoryAddress::direct(index),
        source: MemoryAddress::direct(index),
        bit_size: mem_value.bit_size(),
    }))
    .chain(vec![
        // input = 0
        Opcode::Const {
            destination: r_input,
            value: (outer_start).into(),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
        },
        // some_function(input)
        Opcode::ForeignCall {
            function: "flat_sum".into(),
            destinations: vec![ValueOrArray::MemoryAddress(r_output)],
            destination_value_types: vec![HeapValueType::field()],
            inputs: vec![ValueOrArray::HeapArray(HeapArray {
                pointer: r_input,
                size: outer_array.len(),
            })],
            input_value_types: vec![HeapValueType::Array {
                value_types: input_array_value_types,
                size: outer_array.len(),
            }],
        },
    ])
    .collect();

    let calldata: Vec<FieldElement> = memory.iter().map(|v| v.to_field()).collect();
    let (memory, counter) = run_foreign_call_test(
        calldata,
        &program,
        VMStatus::ForeignCallWait {
            function: "flat_sum".into(),
            inputs: vec![ForeignCallParam::Array(vec![
                (1u128).into(),
                (2u128).into(), // size of following vector
                (2u128).into(),
                (3u128).into(),
                (4u128).into(),
                (5u128).into(),
                (3u128).into(), // size of following vector
                (6u128).into(),
                (7u128).into(),
                (8u128).into(),
                (9u128).into(),
            ])],
        },
        vec![FieldElement::from(45u128).into()],
        VMStatus::Finished { return_data_offset: 0, return_data_size: 0 },
    );

    // Check result
    let result_value = memory.read(r_output);
    assert_eq!(result_value, MemoryValue::new_field(FieldElement::from(45u128)));

    // Ensure the foreign call counter has been incremented
    assert_eq!(counter, 1);
}

#[test]
fn handles_foreign_calls_returning_empty_arrays() {
    let opcodes = &[
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(1u64),
        },
        Opcode::ForeignCall {
            function: "foo".to_string(),
            destinations: vec![ValueOrArray::HeapArray(HeapArray {
                pointer: MemoryAddress::Direct(0),
                size: 0,
            })],
            destination_value_types: vec![HeapValueType::Array {
                value_types: vec![HeapValueType::Simple(BitSize::Field)],
                size: 0,
            }],
            inputs: Vec::new(),
            input_value_types: Vec::new(),
        },
    ];

    run_foreign_call_test(
        vec![],
        opcodes,
        VMStatus::ForeignCallWait { function: "foo".to_string(), inputs: Vec::new() },
        vec![ForeignCallParam::Array(vec![])],
        VMStatus::Finished { return_data_offset: 0, return_data_size: 0 },
    );
}

#[test]
fn aborts_when_foreign_call_returns_too_much_data() {
    let opcodes = &[
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(1u64),
        },
        Opcode::ForeignCall {
            function: "foo".to_string(),
            destinations: vec![ValueOrArray::HeapArray(HeapArray {
                pointer: MemoryAddress::Direct(0),
                size: 3,
            })],
            destination_value_types: vec![HeapValueType::Array {
                value_types: vec![HeapValueType::Simple(BitSize::Field)],
                size: 3,
            }],
            inputs: Vec::new(),
            input_value_types: Vec::new(),
        },
    ];

    run_foreign_call_test(
        vec![],
        opcodes,
        VMStatus::ForeignCallWait { function: "foo".to_string(), inputs: Vec::new() },
        vec![ForeignCallParam::Array(vec![
            FieldElement::from(1u128),
            FieldElement::from(2u128),
            FieldElement::from(3u128),
            FieldElement::from(4u128), // Extra value that exceeds the expected size
        ])],
        VMStatus::Failure {
            reason: FailureReason::RuntimeError {
                message:
                    "Foreign call return value does not match expected size. Expected 3 but got 4"
                        .to_string(),
            },
            call_stack: vec![1],
        },
    );
}

#[test]
fn aborts_when_foreign_call_returns_not_enough_much_data() {
    let opcodes = &[
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(1u64),
        },
        Opcode::ForeignCall {
            function: "foo".to_string(),
            destinations: vec![ValueOrArray::HeapArray(HeapArray {
                pointer: MemoryAddress::Direct(0),
                size: 3,
            })],
            destination_value_types: vec![HeapValueType::Array {
                value_types: vec![HeapValueType::Simple(BitSize::Field)],
                size: 3,
            }],
            inputs: Vec::new(),
            input_value_types: Vec::new(),
        },
    ];

    run_foreign_call_test(
        vec![],
        opcodes,
        VMStatus::ForeignCallWait { function: "foo".to_string(), inputs: Vec::new() },
        vec![ForeignCallParam::Array(vec![
            FieldElement::from(1u128),
            FieldElement::from(2u128),
            // We're missing a value here
        ])],
        VMStatus::Failure {
            reason: FailureReason::RuntimeError {
                message:
                    "Foreign call return value does not match expected size. Expected 3 but got 2"
                        .to_string(),
            },
            call_stack: vec![1],
        },
    );
}

#[test]
fn aborts_when_foreign_call_returns_data_which_does_not_match_vector_elements() {
    let opcodes = &[
        // Set the free memory to start at slot 3
        Opcode::Const {
            destination: FREE_MEMORY_POINTER_ADDRESS,
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(3u64),
        },
        Opcode::ForeignCall {
            function: "foo".to_string(),
            // Pretend that we will point to the the destination addresses
            // using pointers in slot @1 and @2
            destinations: vec![ValueOrArray::HeapVector(HeapVector {
                pointer: MemoryAddress::Direct(1),
                size: MemoryAddress::Direct(2),
            })],
            destination_value_types: vec![HeapValueType::Vector {
                value_types: vec![
                    HeapValueType::Simple(BitSize::Field),
                    HeapValueType::Simple(BitSize::Field),
                ],
            }],
            inputs: Vec::new(),
            input_value_types: Vec::new(),
        },
    ];

    // Here we're returning an array of 3 elements, however the vector expects 2 fields per element
    // (see `value_types` above), so the returned data does not match the expected vector element size
    let foreign_call_result = vec![ForeignCallParam::Array(vec![
        FieldElement::from(1u128),
        FieldElement::from(2u128),
        FieldElement::from(3u128),
        // We're missing a value here
    ])];

    run_foreign_call_test(
        vec![],
        opcodes,
        VMStatus::ForeignCallWait { function: "foo".to_string(), inputs: Vec::new() },
        foreign_call_result,
        VMStatus::Failure {
            reason: FailureReason::RuntimeError {
                message: "Returned data does not match vector element size".to_string(),
            },
            call_stack: vec![1],
        },
    );
}
