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

pub fn recursion(c: &mut Criterion) {
    let mut group = c.benchmark_group("Recursion");
    let mut vm = VirtualMachine::new();
    for i in [
        Iteration(10, evie_vm_bench::fib::src).build(),
        Iteration(15, evie_vm_bench::fib::src).build(),
        Iteration(20, evie_vm_bench::fib::src).build(),
        Iteration(25, evie_vm_bench::fib::src).build(),
        Iteration(30, evie_vm_bench::fib::src).build(),
        Iteration(35, evie_vm_bench::fib::src).build(),
    ]
    .into_iter()
    {
        group.bench_with_input(BenchmarkId::new("Recursion_count", i.0), &i, |b, i| {
            b.iter(|| vm.interpret(i.1.clone(), None));
        });
    }
}

pub fn string_equality(c: &mut Criterion) {
    let mut group = c.benchmark_group("String_Equality");
    let mut vm = VirtualMachine::new();
    for i in [
        Iteration(100, evie_vm_bench::string_equality::src).build(),
        Iteration(1000, evie_vm_bench::string_equality::src).build(),
        Iteration(10000, evie_vm_bench::string_equality::src).build(),
        Iteration(100000, evie_vm_bench::string_equality::src).build(),
        Iteration(1000000, evie_vm_bench::string_equality::src).build(),
    ]
    .into_iter()
    {
        group.bench_with_input(BenchmarkId::new("Iteration_count", i.0), &i, |b, i| {
            b.iter(|| vm.interpret(i.1.clone(), None));
        });
    }
}

criterion_group!(benches, equality, recursion, string_equality);
criterion_main!(benches);
