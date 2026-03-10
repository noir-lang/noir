use criterion::{Criterion, criterion_group, criterion_main};
use std::{hint::black_box, time::Duration};

use acir::AcirField;
use bn254_blackbox_solver::poseidon2_permutation;

use acir::acir_field::GenericFieldElement;
use pprof::criterion::{Output, PProfProfiler};

fn bench_poseidon2(c: &mut Criterion) {
    let inputs = [GenericFieldElement::<ark_bn254::Fr>::zero(); 4];

    c.bench_function("poseidon2", |b| b.iter(|| poseidon2_permutation(black_box(&inputs))));
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(40).measurement_time(Duration::from_secs(20)).with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_poseidon2
);

criterion_main!(benches);
