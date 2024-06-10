use criterion::{criterion_group, criterion_main, Criterion};
use std::{hint::black_box, time::Duration};

use acir::{AcirField, FieldElement};
use acvm_blackbox_solver::BlackBoxFunctionSolver;
use bn254_blackbox_solver::{poseidon2_permutation, Bn254BlackBoxSolver};

use pprof::criterion::{Output, PProfProfiler};

fn bench_poseidon2(c: &mut Criterion) {
    let inputs = [FieldElement::zero(); 4];

    c.bench_function("poseidon2", |b| b.iter(|| poseidon2_permutation(black_box(&inputs), 4)));
}

fn bench_pedersen_commitment(c: &mut Criterion) {
    let inputs = [FieldElement::one(); 2];

    c.bench_function("pedersen_commitment", |b| {
        b.iter(|| Bn254BlackBoxSolver.pedersen_commitment(black_box(&inputs), 0))
    });
}

fn bench_pedersen_hash(c: &mut Criterion) {
    let inputs = [FieldElement::one(); 2];

    c.bench_function("pedersen_hash", |b| {
        b.iter(|| Bn254BlackBoxSolver.pedersen_hash(black_box(&inputs), 0))
    });
}

fn bench_schnorr_verify(c: &mut Criterion) {
    let pub_key_x = FieldElement::from_hex(
        "0x04b260954662e97f00cab9adb773a259097f7a274b83b113532bce27fa3fb96a",
    )
    .unwrap();
    let pub_key_y = FieldElement::from_hex(
        "0x2fd51571db6c08666b0edfbfbc57d432068bccd0110a39b166ab243da0037197",
    )
    .unwrap();
    let sig_bytes: [u8; 64] = [
        1, 13, 119, 112, 212, 39, 233, 41, 84, 235, 255, 93, 245, 172, 186, 83, 157, 253, 76, 77,
        33, 128, 178, 15, 214, 67, 105, 107, 177, 234, 77, 48, 27, 237, 155, 84, 39, 84, 247, 27,
        22, 8, 176, 230, 24, 115, 145, 220, 254, 122, 135, 179, 171, 4, 214, 202, 64, 199, 19, 84,
        239, 138, 124, 12,
    ];

    let message: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    c.bench_function("schnorr_verify", |b| {
        b.iter(|| {
            Bn254BlackBoxSolver.schnorr_verify(
                black_box(&pub_key_x),
                black_box(&pub_key_y),
                black_box(&sig_bytes),
                black_box(message),
            )
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(40).measurement_time(Duration::from_secs(20)).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_poseidon2, bench_pedersen_commitment, bench_pedersen_hash, bench_schnorr_verify
);

criterion_main!(benches);
