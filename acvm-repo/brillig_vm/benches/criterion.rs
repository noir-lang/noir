use acir::{brillig, AcirField, FieldElement};
use criterion::{criterion_group, criterion_main, Criterion};

use pprof::criterion::{Output, PProfProfiler};

use std::time::Duration;

fn field_addition(c: &mut Criterion) {
    let mut bytecode = vec![
        brillig::Opcode::Const {
            destination: brillig::MemoryAddress::Direct(0),
            bit_size: brillig::BitSize::Field,
            value: FieldElement::from(1u128),
        },
        brillig::Opcode::Const {
            destination: brillig::MemoryAddress::Direct(1),
            bit_size: brillig::BitSize::Field,
            value: FieldElement::from(2u128),
        },
    ];

    for _ in 0..100 {
        bytecode.push(brillig::Opcode::BinaryFieldOp {
            destination: brillig::MemoryAddress::Direct(0),
            op: brillig::BinaryFieldOp::Add,
            lhs: brillig::MemoryAddress::Direct(0),
            rhs: brillig::MemoryAddress::Direct(1),
        });
    }

    bench_bytecode(c, "field_addition", &bytecode);
}

fn byte_addition(c: &mut Criterion) {
    let mut bytecode = vec![
        brillig::Opcode::Const {
            destination: brillig::MemoryAddress::Direct(0),
            bit_size: brillig::BitSize::Field,
            value: FieldElement::from(1u128),
        },
        brillig::Opcode::Const {
            destination: brillig::MemoryAddress::Direct(1),
            bit_size: brillig::BitSize::Field,
            value: FieldElement::from(2u128),
        },
    ];

    for _ in 0..100 {
        bytecode.push(brillig::Opcode::BinaryIntOp {
            destination: brillig::MemoryAddress::Direct(2),
            op: brillig::BinaryIntOp::Add,
            bit_size: brillig::IntegerBitSize::U8,
            lhs: brillig::MemoryAddress::Direct(0),
            rhs: brillig::MemoryAddress::Direct(1),
        });
    }

    bench_bytecode(c, "byte_addition", &bytecode);
}

fn bench_bytecode<F: AcirField>(
    c: &mut Criterion,
    benchmark_name: &str,
    bytecode: &[brillig::Opcode<F>],
) {
    c.bench_function(benchmark_name, |b| {
        b.iter_batched(
            || {
                let vm = brillig_vm::VM::new(
                    Vec::new(),
                    bytecode,
                    Vec::new(),
                    &acvm_blackbox_solver::StubbedBlackBoxSolver(false),
                    false,
                );
                vm
            },
            |mut vm| vm.process_opcodes(),
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group! {
    name = execution_benches;
    config = Criterion::default().sample_size(20).measurement_time(Duration::from_secs(20)).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = field_addition, byte_addition
}
criterion_main!(execution_benches);
