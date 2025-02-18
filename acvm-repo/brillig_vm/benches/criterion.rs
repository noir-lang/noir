use acir::{
    brillig::{self, MemoryAddress},
    AcirField, FieldElement,
};
use brillig_vm::VMStatus;
use criterion::{criterion_group, criterion_main, Criterion};

use pprof::criterion::{Output, PProfProfiler};
use rand::Rng;

use std::time::Duration;

fn byte_decomposition(c: &mut Criterion) {
    let mut bytecode = vec![
        // Radix
        brillig::Opcode::Const {
            destination: MemoryAddress::Direct(2),
            bit_size: brillig::BitSize::Integer(brillig::IntegerBitSize::U32),
            value: FieldElement::from(2u128),
        },
        // Output pointer
        brillig::Opcode::Const {
            destination: MemoryAddress::Direct(3),
            bit_size: brillig::BitSize::Integer(brillig::IntegerBitSize::U32),
            value: FieldElement::from(5u128),
        },
        // num_limbs
        brillig::Opcode::Const {
            destination: MemoryAddress::Direct(4),
            bit_size: brillig::BitSize::Integer(brillig::IntegerBitSize::U32),
            value: FieldElement::from(32u128),
        },
        // output_bits
        brillig::Opcode::Const {
            destination: MemoryAddress::Direct(5),
            bit_size: brillig::BitSize::Integer(brillig::IntegerBitSize::U1),
            value: FieldElement::from(false),
        },
        // calldata offset
        brillig::Opcode::Const {
            destination: MemoryAddress::Direct(0),
            bit_size: brillig::BitSize::Integer(brillig::IntegerBitSize::U32),
            value: FieldElement::from(0u128),
        },
        // calldata size
        brillig::Opcode::Const {
            destination: MemoryAddress::Direct(1),
            bit_size: brillig::BitSize::Integer(brillig::IntegerBitSize::U32),
            value: FieldElement::from(1u128),
        },
        brillig::Opcode::CalldataCopy {
            destination_address: MemoryAddress::Direct(0),
            size_address: MemoryAddress::Direct(1),
            offset_address: MemoryAddress::Direct(0),
        },
    ];

    bytecode.push(brillig::Opcode::BlackBox(brillig::BlackBoxOp::ToRadix {
        input: MemoryAddress::Direct(0),
        radix: MemoryAddress::Direct(2),
        output_pointer: MemoryAddress::Direct(3),
        num_limbs: MemoryAddress::Direct(4),
        output_bits: MemoryAddress::Direct(5),
    }));

    bench_bytecode(c, "byte_decomposition", &bytecode);
}

fn bench_bytecode<F: AcirField>(
    c: &mut Criterion,
    benchmark_name: &str,
    bytecode: &[brillig::Opcode<F>],
) {
    c.bench_function(benchmark_name, |b| {
        b.iter_batched(
            || {
                let input = rand::thread_rng().gen::<u128>();
                let input_field = F::from(input);
                let mut vm = brillig_vm::VM::new(
                    vec![input_field],
                    bytecode,
                    &acvm_blackbox_solver::StubbedBlackBoxSolver(false),
                    false,
                );

                // Process up to the last opcode which we want to benchmark
                for _ in 0..bytecode.len() - 1 {
                    vm.process_opcode();
                }
                vm
            },
            |mut vm| {
                let status = vm.process_opcodes();
                assert!(matches!(status, VMStatus::Finished { .. }))
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group! {
    name = execution_benches;
    config = Criterion::default().sample_size(20).measurement_time(Duration::from_secs(20)).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = byte_decomposition
}
criterion_main!(execution_benches);
