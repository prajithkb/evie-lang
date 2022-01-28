pub mod binary_tree;
pub mod equality;
pub mod fib;
pub mod instantiation;
pub mod invocation;
pub mod properties;
pub mod string_equality;
pub mod trees;
pub mod zoo;
#[cfg(test)]
#[ctor::ctor]
fn init() {
    evie_common::env_logger::init();
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use evie_common::errors::*;
    use evie_native::clock;
    use evie_vm::vm::VirtualMachine;

    #[test]
    fn it_works() -> Result<()> {
        let mut vm = VirtualMachine::new();
        let start = Instant::now();
        evie_vm::vm::define_native_fn("clock", 0, &mut vm, clock);
        vm.interpret(crate::binary_tree::src(10), None)?;
        vm.interpret(crate::equality::src(10), None)?;
        vm.interpret(crate::invocation::src(10), None)?;
        vm.interpret(crate::instantiation::src(10), None)?;
        vm.interpret(crate::properties::src(10), None)?;
        vm.interpret(crate::string_equality::src(10), None)?;
        vm.interpret(crate::trees::src(10), None)?;
        vm.interpret(crate::zoo::src(10), None)?;
        println!("Elapsed: {} ms", start.elapsed().as_millis());
        Ok(())
    }
}
