use evie::lox::Lox;
use evie_common::{env_logger, errors::*, print_error};
use std::env;
use std::io::stderr;
fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();
    let result = match args.len() {
        1 => lox.run_prompt(),
        2 => lox.run_script_with_exit_code(&args[1]),
        _ => print_help(),
    };
    match result {
        Ok(_) => {}
        Err(e) => print_error(e, &mut stderr()),
    };
    Ok(())
}

fn print_help() -> Result<()> {
    eprintln!("Usage: evie [type=interpreter|vm] [script=path to a file]\nNotes: Only values for type are 'interpreter' and 'vm");
    Ok(())
}
