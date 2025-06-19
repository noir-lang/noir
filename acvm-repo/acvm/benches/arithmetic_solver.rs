use std::{collections::BTreeMap, time::Duration};

use acir::{
    AcirField, FieldElement,
    circuit::Opcode,
    native_types::{Expression, Witness, WitnessMap},
};
use acvm::pwg::{ACVM, ACVMStatus};
use acvm_blackbox_solver::StubbedBlackBoxSolver;
use criterion::{Criterion, criterion_group, criterion_main};
use pprof::criterion::{Output, PProfProfiler};

fn purely_sequential_opcodes(c: &mut Criterion) {
    // This bytecode defines a stack of constraints such that `w_{i+1} = w_i + 1`.
    // This prevents any batch inversions as all opcodes require the result of the previous opcode as an input.
    let bytecode: Vec<Opcode<FieldElement>> = (0..1000)
        .map(|witness_index| {
            Opcode::AssertZero(Expression {
                mul_terms: Vec::new(),
                linear_combinations: vec![
                    (FieldElement::one(), Witness(witness_index)),
                    (-FieldElement::one(), Witness(witness_index + 1)),
                ],
                q_c: FieldElement::one(),
            })
        })
        .collect();

    bench_bytecode(c, "purely_sequential_opcodes", &bytecode);
}

fn perfectly_parallel_opcodes(c: &mut Criterion) {
    // This bytecode defines a set of constraints such that `w_{i+1} = w_0 + i`.
    // This allows all opcodes to be solved with a single field inversion assuming perfect batching.
    let bytecode: Vec<Opcode<FieldElement>> = (1..1000)
        .map(|witness_index| {
            Opcode::AssertZero(Expression {
                mul_terms: Vec::new(),
                linear_combinations: vec![
                    (FieldElement::one(), Witness(0)),
                    (-FieldElement::one(), Witness(witness_index)),
                ],
                q_c: FieldElement::from(witness_index),
            })
        })
        .collect();

    bench_bytecode(c, "perfectly_parallel_opcodes", &bytecode);
}

fn perfectly_parallel_batch_inversion_opcodes(c: &mut Criterion) {
    // This bytecode defines a set of constraints such that `w_{i+1} = 2*w_0 + i`.
    // This allows all opcodes to be solved with a single field inversion assuming perfect batching.
    let bytecode: Vec<Opcode<FieldElement>> = (1..1000)
        .map(|witness_index| {
            Opcode::AssertZero(Expression {
                mul_terms: Vec::new(),
                linear_combinations: vec![
                    (FieldElement::one(), Witness(0)),
                    (-FieldElement::from(2_u32), Witness(witness_index)),
                ],
                q_c: FieldElement::from(witness_index),
            })
        })
        .collect();

    bench_bytecode(c, "perfectly_parallel_batch_inversion_opcodes", &bytecode);
}

fn bench_bytecode<F: AcirField>(c: &mut Criterion, benchmark_name: &str, bytecode: &[Opcode<F>]) {
    c.bench_function(benchmark_name, |b| {
        b.iter_batched(
            || {
                let initial_witness = WitnessMap::from(BTreeMap::from([(Witness(0), F::one())]));
                ACVM::new(&StubbedBlackBoxSolver(true), bytecode, initial_witness, &[], &[])
            },
            |mut vm| {
                let status = vm.solve();
                assert!(matches!(status, ACVMStatus::Solved));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group! {
    name = execution_benches;
    config = Criterion::default().sample_size(20).measurement_time(Duration::from_secs(20)).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = purely_sequential_opcodes, perfectly_parallel_opcodes, perfectly_parallel_batch_inversion_opcodes
}
criterion_main!(execution_benches);
