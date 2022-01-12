//! The 'frontend' crate which parses the source code and produces [tokens].
#[macro_use(bail)]
extern crate evie_common;

pub mod scanner;
pub mod tokens;
