use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::time::Duration;

use acir::FieldElement;

use pprof::criterion::{Output, PProfProfiler};

use acvm_blackbox_solver::{bit_and, bit_xor};

fn bench_logic_ops(c: &mut Criterion) {
    let lhs = FieldElement::from(0xABCDu128);
    let rhs = FieldElement::from(0x1234u128);

    let mut group = c.benchmark_group("logic_ops");

    for &bits in &[8u32, 32u32, 64u32] {
        group.bench_function(format!("bit_and_{bits}bits"), |b| {
            b.iter(|| {
                let _ = bit_and(black_box(lhs), black_box(rhs), black_box(bits));
            });
        });

        group.bench_function(format!("bit_xor_{bits}bits"), |b| {
            b.iter(|| {
                let _ = bit_xor(black_box(lhs), black_box(rhs), black_box(bits));
            });
        });
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(15))
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_logic_ops
);

criterion_main!(benches);
