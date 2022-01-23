//! THe virtual machine crate.
//! Implements the logic for all the instructions defined in [evie_instructions::opcodes]
mod runtime_memory;
pub mod vm;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    evie_common::env_logger::init();
}
