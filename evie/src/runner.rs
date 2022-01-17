use std::{
    fs::File,
    io::{self, stderr, Read, Write},
};

use evie_common::{errors::*, print_error};
use evie_native::clock;
use evie_vm::vm::VirtualMachine;

pub struct Runner<'a> {
    vm: VirtualMachine<'a>,
}

impl<'a> Runner<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut vm = VirtualMachine::new();
        // Define native functions
        evie_vm::vm::define_native_fn("clock", 0, &mut vm, clock);
        Runner { vm }
    }

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

    pub fn run_script_with_exit_code(&mut self, script: &str) -> Result<()> {
        self.run_script(script)
    }

    pub fn run_vm(&mut self, source: String) -> Result<()> {
        self.vm.interpret(source, None)?;
        Ok(())
    }
    pub fn run_prompt(&mut self) -> Result<()> {
        loop {
            print!("cmd> ");
            io::stdout().flush().chain_err(|| "")?;
            let mut line = String::new();
            let bytes = io::stdin()
                .read_line(&mut line)
                .chain_err(|| "Unable to read stdin")?;
            let result = self.run_vm(line.trim().to_string());
            match result {
                Ok(_) => continue,
                Err(e) => {
                    eprintln!("Command encountered error, to exit press Ctrl + C");
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
}
