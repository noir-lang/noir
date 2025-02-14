use acir_field::{AcirField, FieldElement};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    let field_element = FieldElement::from(123456789_u128);
    c.bench_function("FieldElement::num_bits", |b| b.iter(|| black_box(field_element).num_bits()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
