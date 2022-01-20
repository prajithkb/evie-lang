pub mod equality;
pub mod fib;
pub mod string_equality;
#[cfg(test)]
#[ctor::ctor]
fn init() {
    evie_common::env_logger::init();
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use evie_common::errors::*;
    use evie_vm::vm::VirtualMachine;

    #[test]
    fn it_works() -> Result<()> {
        let mut vm = VirtualMachine::new();
        let source = crate::equality::src(10000);
        let start = Instant::now();
        vm.interpret(source, None)?;
        println!("Elapsed: {} ms", start.elapsed().as_millis());
        Ok(())
    }
}
