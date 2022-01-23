///! The runner for evie. This is invoked from the cmd line
/// Evie supports both executing a file and repl mode
use std::{
    fs::File,
    io::{self, stderr, Read, Write},
};

use evie_common::{errors::*, print_error};
use evie_native::{clock, to_string};
use evie_vm::vm::VirtualMachine;

/// The runner is responsible for streaming code into the [VirtualMachine] via repl or  reading from a file
pub struct Runner<'a> {
    vm: VirtualMachine<'a>,
}

impl<'a> Runner<'a> {
    /// Creates a new instance of Runner.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut vm = VirtualMachine::new();
        // Define native functions
        evie_vm::vm::define_native_fn("clock", 0, &mut vm, clock);
        evie_vm::vm::define_native_fn("to_string", 1, &mut vm, to_string);
        Runner { vm }
    }

    /// Run the given script
    pub fn run_script(&mut self, path: &str) -> Result<()> {
        let mut script = File::open(path).chain_err(|| "Unable to create file")?;
        let mut script_contents = String::new();
        if script
            .read_to_string(&mut script_contents)
            .chain_err(|| "Unable to read file")?
            > 0
        {
            self.run_vm(script_contents)?;
        }
        self.vm.free();
        Ok(())
    }
    /// REPL mode
    pub fn repl(&mut self) -> Result<()> {
        println!("####### REPL mode (evie) ########");
        loop {
            print!("evie> ");
            io::stdout().flush().chain_err(|| "")?;
            let mut line = String::new();
            let bytes = io::stdin()
                .read_line(&mut line)
                .chain_err(|| "Unable to read stdin")?;
            let result = self.run_vm(with_semi_colon(line.trim().to_string()));
            match result {
                Ok(_) => continue,
                Err(e) => {
                    print_error(e, &mut stderr());
                }
            };
            if bytes == 0 {
                break;
            }
        }
        self.vm.free();
        Ok(())
    }

    fn run_vm(&mut self, source: String) -> Result<()> {
        self.vm.interpret(source, None)?;
        Ok(())
    }
}

pub fn with_semi_colon(mut line: String) -> String {
    if !line.ends_with(';') {
        line.push(';');
    }
    line
}
