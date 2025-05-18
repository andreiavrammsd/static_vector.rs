use criterion::{Criterion, criterion_group, criterion_main};
use static_vector::Vec;

fn bench_static_vector(c: &mut Criterion) {
    let mut vec = Vec::<i32, 10>::new();

    c.bench_function("push and clear", |b| {
        b.iter(|| {
            vec.push(&1).unwrap();
            vec.push(&2).unwrap();
            vec.push(&3).unwrap();
            vec.push(&4).unwrap();
            vec.push(&5).unwrap();
            vec.push(&6).unwrap();
            vec.clear();
        })
    });
}

criterion_group!(benches, bench_static_vector);
criterion_main!(benches);
