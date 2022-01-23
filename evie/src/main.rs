use evie::runner::Runner;
use evie_common::{env_logger, errors::*, print_error};
use std::env;
use std::io::stderr;
fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let mut runner = Runner::new();
    let result = match args.len() {
        1 => runner.repl(),
        2 => runner.run_script(&args[1]),
        _ => print_help(),
    };
    match result {
        Ok(_) => {}
        Err(e) => print_error(e, &mut stderr()),
    };
    Ok(())
}

fn print_help() -> Result<()> {
    eprintln!("Usage: evie [path to evie script]\nNote: If you run without any arguments, you enter REPL mode");
    Ok(())
}
