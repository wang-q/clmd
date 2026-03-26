use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(_c: &mut Criterion) {
    // Benchmarks will be added here
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
