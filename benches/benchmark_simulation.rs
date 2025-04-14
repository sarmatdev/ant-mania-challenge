use ant_mania::Hiveum;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_simulation(c: &mut Criterion) {
    c.bench_function("simulate", |b| {
        b.iter(|| {
            let mut hiveum = Hiveum::new("hiveum_map_small.txt", 100, 420);

            hiveum.simulate()
        })
    });
}

criterion_group!(benches, bench_simulation);
criterion_main!(benches);
