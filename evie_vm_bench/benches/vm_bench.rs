use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use evie_vm::vm::VirtualMachine;

struct Iteration(usize, fn(usize) -> String);

impl Iteration {
    fn build(&self) -> (usize, String) {
        (self.0, self.1(self.0))
    }
}

pub fn equality(c: &mut Criterion) {
    let mut group = c.benchmark_group("Equality");
    let mut vm = VirtualMachine::new();
    for i in [
        Iteration(100, evie_vm_bench::equality::src).build(),
        Iteration(1000, evie_vm_bench::equality::src).build(),
        Iteration(10000, evie_vm_bench::equality::src).build(),
        Iteration(100000, evie_vm_bench::equality::src).build(),
        Iteration(1000000, evie_vm_bench::equality::src).build(),
    ]
    .into_iter()
    {
        group.bench_with_input(BenchmarkId::new("Iteration_count", i.0), &i, |b, i| {
            b.iter(|| vm.interpret(i.1.clone(), None));
        });
    }
}

criterion_group!(benches, equality);
criterion_main!(benches);
