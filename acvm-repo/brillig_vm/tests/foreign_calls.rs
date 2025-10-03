use acir::{
    AcirField, FieldElement,
    brillig::{
        BitSize, ForeignCallParam, ForeignCallResult, HeapArray, HeapValueType, HeapVector,
        IntegerBitSize, MemoryAddress, Opcode, ValueOrArray,
    },
};
use acvm_blackbox_solver::StubbedBlackBoxSolver;
use brillig_vm::{FailureReason, VM, VMStatus};

fn run_foreign_call_test<F: AcirField>(
    opcodes: &[Opcode<F>],
    expected_foreign_call_status: VMStatus<F>,
    foreign_call_result: Vec<ForeignCallParam<F>>,
    expected_final_status: VMStatus<F>,
) {
    let calldata: Vec<F> = vec![];
    let solver = StubbedBlackBoxSolver::default();
    let mut vm = VM::new(calldata, opcodes, &[], &solver, false, None);

    let status = vm.process_opcodes();
    assert_eq!(status, expected_foreign_call_status);

    vm.resolve_foreign_call(ForeignCallResult { values: foreign_call_result });
    let status = vm.process_opcode();
    assert_eq!(status, expected_final_status);
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
        Opcode::Const {
            destination: MemoryAddress::direct(0),
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(2u64),
        },
        Opcode::ForeignCall {
            function: "foo".to_string(),
            destinations: vec![ValueOrArray::HeapVector(HeapVector {
                pointer: MemoryAddress::Direct(0),
                size: MemoryAddress::Direct(1),
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
