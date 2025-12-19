#[cfg(test)]
mod foreign_calls;

use acir::{
    AcirField, FieldElement,
    brillig::{
        BinaryFieldOp, BinaryIntOp, BitSize, HeapVector, IntegerBitSize, MemoryAddress, Opcode,
    },
};
use acvm_blackbox_solver::StubbedBlackBoxSolver;
use brillig_vm::{
    FREE_MEMORY_POINTER_ADDRESS, FailureReason, MEMORY_ADDRESSING_BIT_SIZE, MemoryValue, VM,
    VMStatus,
};

/// Helper to execute brillig code and return the VM
fn brillig_execute_and_get_vm<'a, F: AcirField>(
    calldata: Vec<F>,
    opcodes: &'a [Opcode<F>],
    solver: &'a StubbedBlackBoxSolver,
) -> VM<'a, F, StubbedBlackBoxSolver> {
    let mut vm = VM::new(calldata, opcodes, solver, false, None);
    vm.process_opcodes();
    assert!(vm.get_call_stack_no_current_counter().is_empty());
    vm
}

#[test]
fn add_single_step_smoke() {
    let calldata = vec![];

    let opcodes = [Opcode::Const {
        destination: MemoryAddress::direct(0),
        bit_size: BitSize::Integer(IntegerBitSize::U32),
        value: FieldElement::from(27u128),
    }];

    // Start VM
    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, &opcodes, &solver, false, None);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

    // The address at index `2` should have the value of 3 since we had an
    // add opcode
    let memory = vm.take_memory();
    let output_value = memory.read(MemoryAddress::direct(0));

    assert_eq!(output_value.to_field(), FieldElement::from(27u128));
}

#[test]
fn jmpif_opcode() {
    let mut calldata: Vec<FieldElement> = vec![];

    let lhs = {
        calldata.push(2u128.into());
        MemoryAddress::direct(calldata.len() - 1)
    };

    let rhs = {
        calldata.push(2u128.into());
        MemoryAddress::direct(calldata.len() - 1)
    };

    let destination = MemoryAddress::direct(calldata.len());

    let opcodes = vec![
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(2u64),
        },
        Opcode::Const {
            destination: MemoryAddress::direct(1),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        Opcode::CalldataCopy {
            destination_address: MemoryAddress::direct(0),
            size_address: MemoryAddress::direct(0),
            offset_address: MemoryAddress::direct(1),
        },
        Opcode::BinaryFieldOp { destination, op: BinaryFieldOp::Equals, lhs, rhs },
        Opcode::Jump { location: 5 },
        Opcode::JumpIf { condition: destination, location: 6 },
    ];

    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, &opcodes, &solver, false, None);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

    let memory = vm.take_memory();
    let output_cmp_value = memory.read(destination);
    assert_eq!(output_cmp_value.to_field(), true.into());
}

#[test]
fn jump_forward_and_backward() {
    let calldata: Vec<FieldElement> = vec![];
    let bit_size = IntegerBitSize::U32;

    let a = MemoryAddress::direct(0);
    let one = MemoryAddress::direct(1);
    let condition = MemoryAddress::direct(4);

    // Opcodes:
    // 0: Set a = 1
    // 1: Initialize `1` constant for incrementing
    // 2: Jump forward to 5
    // 3: Should be skipped (a = 10)
    // 4: Should be skipped (other_val = 10)
    // 5: Increment a by 1
    // 6: Initialize `2` constant
    // 7: a < 2
    // 8: If a < 2, jump back to 5
    let opcodes = vec![
        Opcode::Const {
            destination: a,
            bit_size: BitSize::Integer(bit_size),
            value: FieldElement::from(1u32),
        },
        Opcode::Const {
            destination: one,
            bit_size: BitSize::Integer(bit_size),
            value: FieldElement::from(1u32),
        },
        Opcode::Jump { location: 5 },
        // Skip this assignment
        Opcode::Const {
            destination: a,
            bit_size: BitSize::Integer(bit_size),
            value: FieldElement::from(10u32),
        },
        // Skip this assignment
        Opcode::Const {
            destination: MemoryAddress::Direct(3),
            bit_size: BitSize::Integer(bit_size),
            value: FieldElement::from(10u32),
        },
        Opcode::BinaryIntOp { destination: a, lhs: a, rhs: one, op: BinaryIntOp::Add, bit_size },
        Opcode::Const {
            destination: MemoryAddress::direct(2),
            bit_size: BitSize::Integer(bit_size),
            value: FieldElement::from(5u32),
        },
        Opcode::BinaryIntOp {
            destination: condition,
            lhs: a,
            rhs: MemoryAddress::direct(2),
            op: BinaryIntOp::LessThan,
            bit_size,
        },
        Opcode::JumpIf { condition, location: 5 },
    ];

    let solver = StubbedBlackBoxSolver::default();
    let vm = brillig_execute_and_get_vm(calldata, &opcodes, &solver);

    let memory = vm.take_memory();

    let a_val = memory.read(a).to_field();
    // a should have incremented correctly
    assert_eq!(a_val.to_u128(), 5);

    // memory[3] should never have been set
    let skipped_val = memory.read(MemoryAddress::direct(3)).to_field();
    assert_eq!(skipped_val.to_u128(), 0);
}

#[test]
fn stop() {
    // Create a vector in memory
    let vector_size: u32 = 100;
    let calldata: Vec<FieldElement> = (0..vector_size).map(FieldElement::from).collect();

    let calldata_pointer = MemoryAddress::direct(calldata.len());

    // Simply immediately return the call data
    let opcodes = vec![
        // The pointer for the call data will come after the calldata itself
        Opcode::Const {
            destination: calldata_pointer,
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u32),
        },
        // Place the size register after all the call data
        Opcode::Const {
            destination: calldata_pointer.offset(1),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(100u32),
        },
        Opcode::CalldataCopy {
            destination_address: MemoryAddress::direct(0),
            size_address: calldata_pointer.offset(1),
            offset_address: calldata_pointer,
        },
        // Stop and return the vector starting at memory[0]
        Opcode::Stop {
            return_data: HeapVector { pointer: calldata_pointer, size: calldata_pointer.offset(1) },
        },
    ];

    let solver = StubbedBlackBoxSolver::default();
    let vm = brillig_execute_and_get_vm(calldata.clone(), &opcodes, &solver);

    let VMStatus::Finished { return_data_offset, return_data_size } = vm.get_status() else {
        panic!("Expected the program to finish execution")
    };

    let memory = vm.take_memory();
    let returned: Vec<_> = (return_data_offset..return_data_size)
        .map(|i| {
            memory
                .read(MemoryAddress::direct(
                    i.try_into().expect("Failed conversion from u32 to usize"),
                ))
                .to_field()
        })
        .collect();
    assert_eq!(returned, calldata);
}

#[test]
fn cast_opcode() {
    let calldata: Vec<FieldElement> = vec![((2_u128.pow(32)) - 1).into()];

    let value_address = MemoryAddress::direct(1);
    let one_usize = MemoryAddress::direct(2);
    let zero_usize = MemoryAddress::direct(3);

    let opcodes = &[
        Opcode::Const {
            destination: one_usize,
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(1u64),
        },
        Opcode::Const {
            destination: zero_usize,
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        Opcode::CalldataCopy {
            destination_address: value_address,
            size_address: one_usize,
            offset_address: zero_usize,
        },
        Opcode::Cast {
            destination: value_address,
            source: value_address,
            bit_size: BitSize::Integer(IntegerBitSize::U8),
        },
        Opcode::Stop {
            return_data: HeapVector {
                pointer: one_usize, // Since value_address is direct(1)
                size: one_usize,
            },
        },
    ];
    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, opcodes, &solver, false, None);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::Finished { return_data_offset: 1, return_data_size: 1 });

    let memory = vm.take_memory();

    let casted_value = memory.read(MemoryAddress::direct(1));
    assert_eq!(casted_value.to_field(), (2_u128.pow(8) - 1).into());
}

#[test]
fn not_opcode() {
    let calldata: Vec<FieldElement> = vec![(1_usize).into()];

    let value_address = MemoryAddress::direct(1);
    let one_usize = MemoryAddress::direct(2);
    let zero_usize = MemoryAddress::direct(3);

    let opcodes = &[
        Opcode::Const {
            destination: one_usize,
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(1u64),
        },
        Opcode::Const {
            destination: zero_usize,
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        Opcode::CalldataCopy {
            destination_address: value_address,
            size_address: one_usize,
            offset_address: zero_usize,
        },
        Opcode::Cast {
            destination: value_address,
            source: value_address,
            bit_size: BitSize::Integer(IntegerBitSize::U128),
        },
        Opcode::Not {
            destination: value_address,
            source: value_address,
            bit_size: IntegerBitSize::U128,
        },
        Opcode::Stop {
            return_data: HeapVector {
                pointer: one_usize, // Since value_address is direct(1)
                size: one_usize,
            },
        },
    ];
    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, opcodes, &solver, false, None);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::Finished { return_data_offset: 1, return_data_size: 1 });

    let memory = vm.take_memory();
    let MemoryValue::U128(negated_value) = memory.read(MemoryAddress::direct(1)) else {
        panic!("Expected integer as the output of Not");
    };
    assert_eq!(negated_value, !1_u128);
}

#[test]
fn mov_opcode() {
    let calldata: Vec<FieldElement> = vec![(1u128).into(), (2u128).into(), (3u128).into()];

    let opcodes = &[
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(3u64),
        },
        Opcode::Const {
            destination: MemoryAddress::direct(1),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        Opcode::CalldataCopy {
            destination_address: MemoryAddress::direct(0),
            size_address: MemoryAddress::direct(0),
            offset_address: MemoryAddress::direct(1),
        },
        Opcode::Mov { destination: MemoryAddress::direct(2), source: MemoryAddress::direct(0) },
    ];
    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, opcodes, &solver, false, None);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();

    assert_eq!(*status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

    let memory = vm.take_memory();

    let destination_value = memory.read(MemoryAddress::direct(2));
    assert_eq!(destination_value.to_field(), (1u128).into());

    let source_value = memory.read(MemoryAddress::direct(0));
    assert_eq!(source_value.to_field(), (1u128).into());
}

#[test]
fn cmov_opcode() {
    let calldata: Vec<FieldElement> =
        vec![(0u128).into(), (1u128).into(), (2u128).into(), (3u128).into()];

    let opcodes = &[
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(4u64),
        },
        Opcode::Const {
            destination: MemoryAddress::direct(1),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        Opcode::CalldataCopy {
            destination_address: MemoryAddress::direct(0),
            size_address: MemoryAddress::direct(0),
            offset_address: MemoryAddress::direct(1),
        },
        Opcode::Cast {
            destination: MemoryAddress::direct(0),
            source: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U1),
        },
        Opcode::Cast {
            destination: MemoryAddress::direct(1),
            source: MemoryAddress::direct(1),
            bit_size: BitSize::Integer(IntegerBitSize::U1),
        },
        Opcode::ConditionalMov {
            destination: MemoryAddress::direct(4), // Sets 3_u128 to memory address 4
            source_a: MemoryAddress::direct(2),
            source_b: MemoryAddress::direct(3),
            condition: MemoryAddress::direct(0),
        },
        Opcode::ConditionalMov {
            destination: MemoryAddress::direct(5), // Sets 2_u128 to memory address 5
            source_a: MemoryAddress::direct(2),
            source_b: MemoryAddress::direct(3),
            condition: MemoryAddress::direct(1),
        },
    ];
    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, opcodes, &solver, false, None);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

    let memory = vm.take_memory();

    let destination_value = memory.read(MemoryAddress::direct(4));
    assert_eq!(destination_value.to_field(), (3_u128).into());

    let source_value = memory.read(MemoryAddress::direct(5));
    assert_eq!(source_value.to_field(), (2_u128).into());
}

#[test]
fn cmp_binary_ops() {
    let bit_size = MEMORY_ADDRESSING_BIT_SIZE;
    let calldata: Vec<FieldElement> =
        vec![(2u128).into(), (2u128).into(), (0u128).into(), (5u128).into(), (6u128).into()];
    let calldata_size = calldata.len();

    let calldata_copy_opcodes = vec![
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(5u64),
        },
        Opcode::Const {
            destination: MemoryAddress::direct(1),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(0u64),
        },
        Opcode::CalldataCopy {
            destination_address: MemoryAddress::direct(0),
            size_address: MemoryAddress::direct(0),
            offset_address: MemoryAddress::direct(1),
        },
    ];

    let cast_opcodes: Vec<_> = (0..calldata_size)
        .map(|index| Opcode::Cast {
            destination: MemoryAddress::direct(index),
            source: MemoryAddress::direct(index),
            bit_size: BitSize::Integer(bit_size),
        })
        .collect();

    let destination = MemoryAddress::direct(2);
    let equal_opcode = Opcode::BinaryIntOp {
        bit_size,
        op: BinaryIntOp::Equals,
        lhs: MemoryAddress::direct(0),
        rhs: MemoryAddress::direct(1),
        destination,
    };

    let not_equal_opcode = Opcode::BinaryIntOp {
        bit_size,
        op: BinaryIntOp::Equals,
        lhs: MemoryAddress::direct(0),
        rhs: MemoryAddress::direct(3),
        destination,
    };

    let less_than_opcode = Opcode::BinaryIntOp {
        bit_size,
        op: BinaryIntOp::LessThan,
        lhs: MemoryAddress::direct(3),
        rhs: MemoryAddress::direct(4),
        destination,
    };

    let less_than_equal_opcode = Opcode::BinaryIntOp {
        bit_size,
        op: BinaryIntOp::LessThanEquals,
        lhs: MemoryAddress::direct(3),
        rhs: MemoryAddress::direct(4),
        destination,
    };

    let opcodes: Vec<_> = calldata_copy_opcodes
        .into_iter()
        .chain(cast_opcodes)
        .chain([equal_opcode, not_equal_opcode, less_than_opcode, less_than_equal_opcode])
        .collect();
    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, &opcodes, &solver, false, None);

    // Calldata copy
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);

    for _ in 0..calldata_size {
        let status = vm.process_opcode();
        assert_eq!(*status, VMStatus::InProgress);
    }

    // Equals
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);

    let output_eq_value = vm.get_memory()[destination.unwrap_direct()];
    assert_eq!(output_eq_value, true.into());

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);

    let output_neq_value = vm.get_memory()[destination.unwrap_direct()];
    assert_eq!(output_neq_value, false.into());

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);

    let lt_value = vm.get_memory()[destination.unwrap_direct()];
    assert_eq!(lt_value, true.into());

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

    let lte_value = vm.get_memory()[destination.unwrap_direct()];
    assert_eq!(lte_value, true.into());
}

#[test]
fn store_opcode() {
    /// Brillig code for the following:
    ///     let mut i = 0;
    ///     let len = memory.len();
    ///     while i < len {
    ///         memory[i] = i as Value;
    ///         i += 1;
    ///     }
    fn brillig_write_memory(item_count: usize) -> Vec<MemoryValue<FieldElement>> {
        let integer_bit_size = MEMORY_ADDRESSING_BIT_SIZE;
        let bit_size = BitSize::Integer(integer_bit_size);
        let r_i = MemoryAddress::direct(0);
        let r_len = MemoryAddress::direct(1);
        let r_tmp = MemoryAddress::direct(2);
        let r_pointer = MemoryAddress::direct(3);

        let start: [Opcode<FieldElement>; 3] = [
            // i = 0
            Opcode::Const { destination: r_i, value: 0u128.into(), bit_size },
            // len = memory.len() (approximation)
            Opcode::Const { destination: r_len, value: item_count.into(), bit_size },
            // pointer = free_memory_ptr
            Opcode::Const { destination: r_pointer, value: 4u128.into(), bit_size },
        ];
        let loop_body = [
            // *i = i
            Opcode::Store { destination_pointer: r_pointer, source: r_i },
            // tmp = 1
            Opcode::Const { destination: r_tmp, value: 1u128.into(), bit_size },
            // i = i + 1 (tmp)
            Opcode::BinaryIntOp {
                destination: r_i,
                lhs: r_i,
                op: BinaryIntOp::Add,
                rhs: r_tmp,
                bit_size: integer_bit_size,
            },
            // pointer = pointer + 1
            Opcode::BinaryIntOp {
                destination: r_pointer,
                lhs: r_pointer,
                op: BinaryIntOp::Add,
                rhs: r_tmp,
                bit_size: integer_bit_size,
            },
            // tmp = i < len
            Opcode::BinaryIntOp {
                destination: r_tmp,
                lhs: r_i,
                op: BinaryIntOp::LessThan,
                rhs: r_len,
                bit_size: integer_bit_size,
            },
            // if tmp != 0 goto loop_body
            Opcode::JumpIf { condition: r_tmp, location: start.len() },
        ];

        let opcodes = [&start[..], &loop_body[..]].concat();
        let solver = StubbedBlackBoxSolver::default();
        let vm = brillig_execute_and_get_vm(vec![], &opcodes, &solver);
        vm.get_memory()[4..].to_vec()
    }

    let memory = brillig_write_memory(5);
    let expected = vec![(0u32).into(), (1u32).into(), (2u32).into(), (3u32).into(), (4u32).into()];
    assert_eq!(memory, expected);

    let memory = brillig_write_memory(1024);
    let expected: Vec<_> = (0..1024).map(|i: u32| i.into()).collect();
    assert_eq!(memory, expected);
}

#[test]
fn iconst_opcode() {
    let opcodes = &[
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            value: FieldElement::from(8_usize),
        },
        Opcode::IndirectConst {
            destination_pointer: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
            value: FieldElement::from(27_usize),
        },
    ];
    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(vec![], opcodes, &solver, false, None);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

    let memory = vm.take_memory();

    let destination_value = memory.read(MemoryAddress::direct(8));
    assert_eq!(destination_value.to_field(), (27_usize).into());
}

#[test]
fn load_opcode() {
    /// Brillig code for the following:
    ///     let mut sum = 0;
    ///     let mut i = 0;
    ///     let len = memory.len();
    ///     while i < len {
    ///         sum += memory[i];
    ///         i += 1;
    ///     }
    fn brillig_sum_memory(memory: Vec<FieldElement>) -> FieldElement {
        let bit_size = IntegerBitSize::U32;
        let r_i = MemoryAddress::direct(0);
        let r_len = MemoryAddress::direct(1);
        let r_sum = MemoryAddress::direct(2);
        let r_tmp = MemoryAddress::direct(3);
        let r_pointer = MemoryAddress::direct(4);

        let start = [
            // sum = 0
            Opcode::Const { destination: r_sum, value: 0u128.into(), bit_size: BitSize::Field },
            // i = 0
            Opcode::Const {
                destination: r_i,
                value: 0u128.into(),
                bit_size: BitSize::Integer(bit_size),
            },
            // len = array.len() (approximation)
            Opcode::Const {
                destination: r_len,
                value: memory.len().into(),
                bit_size: BitSize::Integer(bit_size),
            },
            // pointer = array_ptr
            Opcode::Const {
                destination: r_pointer,
                value: 5u128.into(),
                bit_size: BitSize::Integer(bit_size),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(100),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(memory.len() as u32),
            },
            Opcode::Const {
                destination: MemoryAddress::direct(101),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(5),
                size_address: MemoryAddress::direct(100),
                offset_address: MemoryAddress::direct(101),
            },
        ];
        let loop_body = [
            // tmp = *i
            Opcode::Load { destination: r_tmp, source_pointer: r_pointer },
            // sum = sum + tmp
            Opcode::BinaryFieldOp {
                destination: r_sum,
                lhs: r_sum,
                op: BinaryFieldOp::Add,
                rhs: r_tmp,
            },
            // tmp = 1
            Opcode::Const {
                destination: r_tmp,
                value: 1u128.into(),
                bit_size: BitSize::Integer(bit_size),
            },
            // i = i + 1 (tmp)
            Opcode::BinaryIntOp {
                destination: r_i,
                lhs: r_i,
                op: BinaryIntOp::Add,
                rhs: r_tmp,
                bit_size,
            },
            // pointer = pointer + 1
            Opcode::BinaryIntOp {
                destination: r_pointer,
                lhs: r_pointer,
                op: BinaryIntOp::Add,
                rhs: r_tmp,
                bit_size,
            },
            // tmp = i < len
            Opcode::BinaryIntOp {
                destination: r_tmp,
                lhs: r_i,
                op: BinaryIntOp::LessThan,
                rhs: r_len,
                bit_size,
            },
            // if tmp != 0 goto loop_body
            Opcode::JumpIf { condition: r_tmp, location: start.len() },
        ];

        let opcodes = [&start[..], &loop_body[..]].concat();
        let solver = StubbedBlackBoxSolver::default();
        let vm = brillig_execute_and_get_vm(memory, &opcodes, &solver);
        vm.take_memory().read(r_sum).to_field()
    }

    assert_eq!(
        brillig_sum_memory(vec![
            (1u128).into(),
            (2u128).into(),
            (3u128).into(),
            (4u128).into(),
            (5u128).into(),
        ]),
        (15u128).into()
    );
    assert_eq!(brillig_sum_memory(vec![(1u128).into(); 1024]), (1024u128).into());
}

#[test]
fn call_and_return_opcodes() {
    /// Brillig code for the following recursive function:
    ///     fn recursive_write(i: u128, len: u128) {
    ///         if len <= i {
    ///             return;
    ///         }
    ///         memory[i as usize] = i as Value;
    ///         recursive_write(memory, i + 1, len);
    ///     }
    /// Note we represent a 100% in-stack optimized form in brillig
    fn brillig_recursive_write_memory<F: AcirField>(size: usize) -> Vec<MemoryValue<F>> {
        let integer_bit_size = MEMORY_ADDRESSING_BIT_SIZE;
        let bit_size = BitSize::Integer(integer_bit_size);
        let r_i = MemoryAddress::direct(0);
        let r_len = MemoryAddress::direct(1);
        let r_tmp = MemoryAddress::direct(2);
        let r_pointer = MemoryAddress::direct(3);

        let start: [Opcode<F>; 5] = [
            // i = 0
            Opcode::Const { destination: r_i, value: 0u128.into(), bit_size },
            // len = size
            Opcode::Const { destination: r_len, value: (size as u128).into(), bit_size },
            // pointer = free_memory_ptr
            Opcode::Const { destination: r_pointer, value: 4u128.into(), bit_size },
            // call recursive_fn
            Opcode::Call {
                            location: 5, // Call after 'start'
                        },
            // end program by jumping to end
            Opcode::Jump { location: 100 },
        ];

        let recursive_fn = [
            // tmp = len <= i
            Opcode::BinaryIntOp {
                destination: r_tmp,
                lhs: r_len,
                op: BinaryIntOp::LessThanEquals,
                rhs: r_i,
                bit_size: integer_bit_size,
            },
            // if !tmp, goto end
            Opcode::JumpIf {
                condition: r_tmp,
                location: start.len() + 7, // 8 ops in recursive_fn, go to 'Return'
            },
            // *i = i
            Opcode::Store { destination_pointer: r_pointer, source: r_i },
            // tmp = 1
            Opcode::Const { destination: r_tmp, value: 1u128.into(), bit_size },
            // i = i + 1 (tmp)
            Opcode::BinaryIntOp {
                destination: r_i,
                lhs: r_i,
                op: BinaryIntOp::Add,
                rhs: r_tmp,
                bit_size: integer_bit_size,
            },
            // pointer = pointer + 1
            Opcode::BinaryIntOp {
                destination: r_pointer,
                lhs: r_pointer,
                op: BinaryIntOp::Add,
                rhs: r_tmp,
                bit_size: integer_bit_size,
            },
            // call recursive_fn
            Opcode::Call { location: start.len() },
            Opcode::Return {},
        ];

        let opcodes = [&start[..], &recursive_fn[..]].concat();
        let solver = StubbedBlackBoxSolver::default();
        let vm = brillig_execute_and_get_vm(vec![], &opcodes, &solver);
        vm.get_memory()[4..].to_vec()
    }

    let memory = brillig_recursive_write_memory::<FieldElement>(5);
    let expected = vec![(0u32).into(), (1u32).into(), (2u32).into(), (3u32).into(), (4u32).into()];
    assert_eq!(memory, expected);

    let memory = brillig_recursive_write_memory::<FieldElement>(1024);
    let expected: Vec<_> = (0..1024).map(|i: u32| i.into()).collect();
    assert_eq!(memory, expected);
}

#[test]
fn relative_addressing() {
    let calldata = vec![];
    let bit_size = BitSize::Integer(IntegerBitSize::U32);
    let value = FieldElement::from(3u128);

    let opcodes = [
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size,
            value: FieldElement::from(27u128),
        },
        Opcode::Const {
            destination: MemoryAddress::relative(1), // Resolved address 28 value 3
            bit_size,
            value,
        },
        Opcode::Const {
            destination: MemoryAddress::direct(1), // Address 1 value 3
            bit_size,
            value,
        },
        Opcode::BinaryIntOp {
            destination: MemoryAddress::direct(1),
            op: BinaryIntOp::Equals,
            bit_size: IntegerBitSize::U32,
            lhs: MemoryAddress::direct(1),
            rhs: MemoryAddress::direct(28),
        },
    ];

    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, &opcodes, &solver, false, None);

    vm.process_opcode();
    vm.process_opcode();
    vm.process_opcode();
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });

    let memory = vm.take_memory();
    let output_value = memory.read(MemoryAddress::direct(1));

    assert_eq!(output_value.to_field(), FieldElement::from(1u128));
}

#[test]
fn field_zero_division_regression() {
    let calldata: Vec<FieldElement> = vec![];

    let opcodes = &[
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Field,
            value: FieldElement::from(1u64),
        },
        Opcode::Const {
            destination: MemoryAddress::direct(1),
            bit_size: BitSize::Field,
            value: FieldElement::from(0u64),
        },
        Opcode::BinaryFieldOp {
            destination: MemoryAddress::direct(2),
            op: BinaryFieldOp::Div,
            lhs: MemoryAddress::direct(0),
            rhs: MemoryAddress::direct(1),
        },
    ];
    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, opcodes, &solver, false, None);

    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(*status, VMStatus::InProgress);
    let status = vm.process_opcode();
    assert_eq!(
        *status,
        VMStatus::Failure {
            reason: FailureReason::RuntimeError { message: "Attempted to divide by zero".into() },
            call_stack: vec![2]
        }
    );
}

#[test]
fn free_memory_pointer_out_of_memory() {
    let calldata: Vec<FieldElement> = vec![];

    // Addresses 0,1,2 are reserved.
    let big_array_size_addr = MemoryAddress::Direct(3);

    let free_memory_starting_slot = 4;

    let opcodes = &[
        // Set the free memory pointer to 4.
        Opcode::Const {
            destination: FREE_MEMORY_POINTER_ADDRESS,
            value: free_memory_starting_slot.into(),
            bit_size: BitSize::Integer(MEMORY_ADDRESSING_BIT_SIZE),
        },
        // Create a constant that, if added to the free memory pointer,
        // would wrap around to equal 0.
        Opcode::Const {
            destination: big_array_size_addr,
            value: FieldElement::from(u32::MAX - free_memory_starting_slot as u32 + 1),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
        },
        // Increase the free memory pointer by the size of the hypothetical big array.
        Opcode::BinaryIntOp {
            destination: FREE_MEMORY_POINTER_ADDRESS,
            op: BinaryIntOp::Add,
            bit_size: MEMORY_ADDRESSING_BIT_SIZE,
            lhs: FREE_MEMORY_POINTER_ADDRESS,
            rhs: big_array_size_addr,
        },
        // Load 0 into slot 0, so the size pointer has somewhere to point.
        Opcode::Const {
            destination: MemoryAddress::Direct(0),
            value: FieldElement::zero(),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
        },
        // We should not get to the stop opcode, but it's here to make sure `process_opcodes` exits.
        Opcode::Stop {
            return_data: HeapVector {
                pointer: MemoryAddress::Direct(0),
                size: MemoryAddress::Direct(0),
            },
        },
    ];
    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, opcodes, &solver, false, None);

    let status = vm.process_opcodes();
    let memory = vm.take_memory();

    // Check that the free memory pointer did not wrap around.
    let free_memory_start = memory.read_ref(FREE_MEMORY_POINTER_ADDRESS);
    assert!(free_memory_start.to_usize() >= free_memory_starting_slot);

    assert_eq!(
        status,
        VMStatus::Failure {
            reason: FailureReason::RuntimeError { message: "Out of memory".into() },
            call_stack: vec![2]
        }
    );
}
