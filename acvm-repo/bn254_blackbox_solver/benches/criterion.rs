use criterion::{criterion_group, criterion_main, Criterion};
use std::{hint::black_box, time::Duration};

use acir::{AcirField, FieldElement};
use bn254_blackbox_solver::poseidon2_permutation;

use pprof::criterion::{Output, PProfProfiler};

fn bench_poseidon2(c: &mut Criterion) {
    let inputs = [FieldElement::zero(); 4];

    c.bench_function("poseidon2", |b| b.iter(|| poseidon2_permutation(black_box(&inputs), 4)));
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(40).measurement_time(Duration::from_secs(20)).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_poseidon2
);

criterion_main!(benches);
