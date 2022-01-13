mod runtime_memory;
pub mod vm;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    evie_common::env_logger::init();
}
