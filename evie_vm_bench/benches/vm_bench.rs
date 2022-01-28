use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use evie_native::clock;
use evie_vm::vm::VirtualMachine;

struct Iteration(usize, fn(usize) -> String);

impl Iteration {
    fn build(&self) -> (usize, String) {
        (self.0, self.1(self.0))
    }
}

fn vm() -> VirtualMachine<'static> {
    let mut vm = VirtualMachine::new();
    evie_vm::vm::define_native_fn("clock", 0, &mut vm, clock);
    vm
}

pub fn equality(c: &mut Criterion) {
    let mut group = c.benchmark_group("Equality");
    let mut vm = vm();
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
    let mut vm = vm();
    for i in [
        Iteration(10, evie_vm_bench::fib::src).build(),
        Iteration(15, evie_vm_bench::fib::src).build(),
        Iteration(20, evie_vm_bench::fib::src).build(),
        Iteration(25, evie_vm_bench::fib::src).build(),
        Iteration(30, evie_vm_bench::fib::src).build(),
        Iteration(35, evie_vm_bench::fib::src).build(),
        Iteration(36, evie_vm_bench::fib::src).build(),
        Iteration(37, evie_vm_bench::fib::src).build(),
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
    let mut vm = vm();
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

pub fn binary_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("Binary_Tree");
    let mut vm = vm();
    for i in [
        Iteration(2, evie_vm_bench::binary_tree::src).build(),
        Iteration(4, evie_vm_bench::binary_tree::src).build(),
        Iteration(6, evie_vm_bench::binary_tree::src).build(),
        Iteration(8, evie_vm_bench::binary_tree::src).build(),
        Iteration(10, evie_vm_bench::binary_tree::src).build(),
    ]
    .into_iter()
    {
        group.bench_with_input(BenchmarkId::new("Iteration_count", i.0), &i, |b, i| {
            b.iter(|| vm.interpret(i.1.clone(), None));
        });
    }
}

pub fn instantiation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Instantiation");
    let mut vm = vm();
    for i in [
        Iteration(100, evie_vm_bench::instantiation::src).build(),
        Iteration(1000, evie_vm_bench::instantiation::src).build(),
        Iteration(10000, evie_vm_bench::instantiation::src).build(),
        Iteration(100000, evie_vm_bench::instantiation::src).build(),
    ]
    .into_iter()
    {
        group.bench_with_input(BenchmarkId::new("Iteration_count", i.0), &i, |b, i| {
            b.iter(|| vm.interpret(i.1.clone(), None));
        });
    }
}

pub fn invocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Invocation");
    let mut vm = vm();
    for i in [
        Iteration(100, evie_vm_bench::invocation::src).build(),
        Iteration(1000, evie_vm_bench::invocation::src).build(),
        Iteration(10000, evie_vm_bench::invocation::src).build(),
        Iteration(100000, evie_vm_bench::invocation::src).build(),
    ]
    .into_iter()
    {
        group.bench_with_input(BenchmarkId::new("Iteration_count", i.0), &i, |b, i| {
            b.iter(|| vm.interpret(i.1.clone(), None));
        });
    }
}

pub fn properties(c: &mut Criterion) {
    let mut group = c.benchmark_group("Properties");
    let mut vm = vm();
    for i in [
        Iteration(100, evie_vm_bench::invocation::src).build(),
        Iteration(1000, evie_vm_bench::invocation::src).build(),
        Iteration(10000, evie_vm_bench::invocation::src).build(),
        Iteration(100000, evie_vm_bench::invocation::src).build(),
    ]
    .into_iter()
    {
        group.bench_with_input(BenchmarkId::new("Iteration_count", i.0), &i, |b, i| {
            b.iter(|| vm.interpret(i.1.clone(), None));
        });
    }
}

pub fn trees(c: &mut Criterion) {
    let mut group = c.benchmark_group("trees");
    let mut vm = vm();
    for i in [
        Iteration(10, evie_vm_bench::trees::src).build(),
        Iteration(20, evie_vm_bench::trees::src).build(),
        Iteration(30, evie_vm_bench::trees::src).build(),
        Iteration(40, evie_vm_bench::trees::src).build(),
        Iteration(50, evie_vm_bench::trees::src).build(),
    ]
    .into_iter()
    {
        group.bench_with_input(BenchmarkId::new("Iteration_count", i.0), &i, |b, i| {
            b.iter(|| vm.interpret(i.1.clone(), None));
        });
    }
}

pub fn zoo(c: &mut Criterion) {
    let mut group = c.benchmark_group("Zoo");
    let mut vm = vm();
    for i in [
        Iteration(100, evie_vm_bench::zoo::src).build(),
        Iteration(1000, evie_vm_bench::zoo::src).build(),
        Iteration(10000, evie_vm_bench::zoo::src).build(),
        Iteration(100000, evie_vm_bench::zoo::src).build(),
        Iteration(1000000, evie_vm_bench::zoo::src).build(),
    ]
    .into_iter()
    {
        group.bench_with_input(BenchmarkId::new("Iteration_count", i.0), &i, |b, i| {
            b.iter(|| vm.interpret(i.1.clone(), None));
        });
    }
}

criterion_group!(
    benches,
    equality,
    recursion,
    string_equality,
    binary_tree,
    instantiation,
    invocation,
    properties,
    trees,
    zoo
);
criterion_main!(benches);
