use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{collections::BTreeSet, time::Duration};

use acir::{
    circuit::{Circuit, ExpressionWidth, Opcode, Program, PublicInputs},
    native_types::{Expression, Witness},
    FieldElement,
};

use pprof::criterion::{Output, PProfProfiler};

const SIZES: [usize; 9] = [10, 50, 100, 500, 1000, 5000, 10000, 50000, 100000];

fn sample_program(num_opcodes: usize) -> Program<FieldElement> {
    let assert_zero_opcodes: Vec<Opcode<_>> = (0..num_opcodes)
        .map(|i| {
            Opcode::AssertZero(Expression {
                mul_terms: vec![(
                    FieldElement::from(2 * i),
                    Witness(i as u32),
                    Witness(i as u32 + 10),
                )],
                linear_combinations: vec![
                    (FieldElement::from(2 * i), Witness(i as u32)),
                    (FieldElement::from(3 * i), Witness(i as u32 + 1)),
                ],
                q_c: FieldElement::from(i),
            })
        })
        .collect();

    Program {
        functions: vec![Circuit {
            current_witness_index: 4000,
            opcodes: assert_zero_opcodes.to_vec(),
            expression_width: ExpressionWidth::Bounded { width: 4 },
            private_parameters: BTreeSet::from([Witness(1), Witness(2), Witness(3), Witness(4)]),
            public_parameters: PublicInputs(BTreeSet::from([Witness(5)])),
            return_values: PublicInputs(BTreeSet::from([Witness(6)])),
            assert_messages: Vec::new(),
            recursive: false,
        }],
        unconstrained_functions: Vec::new(),
    }
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_program");
    for size in SIZES.iter() {
        let program = sample_program(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &program, |b, program| {
            b.iter(|| Program::serialize_program(program));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("serialize_program_json");
    for size in SIZES.iter() {
        let program = sample_program(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &program, |b, program| {
            b.iter(|| {
                let mut bytes = Vec::new();
                let mut serializer = serde_json::Serializer::new(&mut bytes);
                Program::serialize_program_base64(program, &mut serializer)
            });
        });
    }
    group.finish();
}

fn bench_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize_program");
    for size in SIZES.iter() {
        let program = sample_program(*size);
        let serialized_program = Program::serialize_program(&program);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &serialized_program,
            |b, program| {
                b.iter(|| Program::<FieldElement>::deserialize_program(program));
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("deserialize_program_json");
    for size in SIZES.iter() {
        let program = sample_program(*size);

        let serialized_program = {
            let mut bytes = Vec::new();
            let mut serializer = serde_json::Serializer::new(&mut bytes);
            Program::serialize_program_base64(&program, &mut serializer).expect("should succeed");
            bytes
        };

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &serialized_program,
            |b, program| {
                b.iter(|| {
                    let mut deserializer = serde_json::Deserializer::from_slice(program);
                    Program::<FieldElement>::deserialize_program_base64(&mut deserializer)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(40).measurement_time(Duration::from_secs(20)).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_serialization, bench_deserialization
);

criterion_main!(benches);
