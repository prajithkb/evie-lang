/**
 * This test runs the bench mark tests and prints the timing information between clox and vm.
 * It does not assert on anything yet.
 */
#[cfg(test)]
mod tests {
    const TEST_CASE_PATH: &str = "/Users/kprajith/workspace/rust/evie-lang/evie_bench/files";
    const CLOX_PATH: &str =
        "/Users/kprajith/workspace/crafting-interpretors/craftinginterpreters/clox";
    const VM_PATH: &str = "/Users/kprajith/workspace/rust/evie-lang/target/release/evie";
    const WS_PATH: &str = "/Users/kprajith/workspace/rust/evie-lang/Cargo.toml";
    use cli_table::{print_stdout, Cell, Color, Style, Table};
    use evie_common::{bail, errors::*};
    use std::{ffi::OsStr, fs, path::Path, process::Command, time::Instant};

    #[test]
    fn perf_timings() -> Result<()> {
        println!("This test runs the bench mark tests and compares the timing (performance) between clox and vm.\nIt DOES NOT  assert on anything!\n");
        println!("Building release...");
        cargo_build_release()?;
        if !binary_path_exists() {
            eprint!("Binary path {} does not exist, exiting!", VM_PATH);
            // Exit early if there is nothing to run
            return Ok(());
        }
        println!("Built release, starting test...");
        let dir_path = Path::new(TEST_CASE_PATH);
        let mut entries: Vec<_> = fs::read_dir(dir_path)?.collect();
        entries.sort_by(|a, b| {
            let a = a.as_ref().unwrap();
            let b = b.as_ref().unwrap();
            a.file_name().cmp(&b.file_name())
        });
        let mut table = vec![];
        let allow_listed_entries = entries;
        // let allow_listed_entries = entries.into_iter().filter(|e| {
        //     [OsStr::new("zoo_batch.lox").to_os_string()].contains(&e.as_ref().unwrap().file_name())
        // });
        for entry in allow_listed_entries {
            let e = entry?;
            if e.file_type()?.is_file() {
                let file_name = String::from(e.file_name().to_string_lossy());
                let path = e.path();
                println!("Benchmark for {:?}", path.as_os_str());
                let timed_taken_by_vm = run_vm(path.as_os_str())?;
                let timed_taken_by_clox = run_clox(path.as_os_str())?;
                let percentage_difference =
                    ((timed_taken_by_vm / timed_taken_by_clox) * 100f64) - 100f64;
                let percentage_difference_styled = if percentage_difference < 0f64 {
                    percentage_difference
                        .cell()
                        .background_color(Some(Color::Green))
                } else {
                    percentage_difference.cell().bold(true)
                };
                println!("Timing for test = {}, time taken by clox ={}, time taken by vm = {}, difference = {} %", file_name, timed_taken_by_clox, timed_taken_by_vm, percentage_difference);
                table.push(vec![
                    file_name.cell(),
                    timed_taken_by_clox.cell(),
                    timed_taken_by_vm.cell(),
                    percentage_difference_styled,
                ]);
            }
        }
        let table = table
            .table()
            .title(vec![
                "Test".cell().bold(true),
                "Clox time in seconds".cell().bold(true),
                "Vm time in seconds".cell().bold(true),
                "Percentage difference".cell().bold(true),
            ])
            .bold(true);

        println!("\nFinal results:");
        print_stdout(table)?;
        Ok(())
    }

    fn binary_path_exists() -> bool {
        Path::new(CLOX_PATH).exists() && Path::new(VM_PATH).exists()
    }

    fn run_clox(path: &OsStr) -> Result<f64> {
        run(OsStr::new(CLOX_PATH), path)
    }

    fn run_vm(path: &OsStr) -> Result<f64> {
        run(OsStr::new(VM_PATH), path)
    }

    fn run(path_to_executable: &OsStr, path_to_file: &OsStr) -> Result<f64> {
        let command = &mut Command::new(path_to_executable);
        command.arg(path_to_file);
        let start_time = Instant::now();
        let output = command.output()?;
        let stdout =
            std::str::from_utf8(&output.stdout).map_err(|e| ErrorKind::Msg(e.to_string()))?;
        let stderr =
            std::str::from_utf8(&output.stderr).map_err(|e| ErrorKind::Msg(e.to_string()))?;
        println!("RAN  {:?}{:?}", path_to_executable, path_to_file);
        println!("STDOUT:{}", stdout);
        println!("STDERR:{}", stderr);
        println!("---------------------------------");
        if !output.status.success() {
            bail!("Error")
        } else {
            Ok(start_time.elapsed().as_secs_f64())
        }
    }

    fn cargo_build_release() -> Result<()> {
        let cargo_run_release = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--manifest-path")
            .arg(WS_PATH)
            .output()?;
        if !cargo_run_release.status.success() {
            println!(
                "{}",
                std::str::from_utf8(&cargo_run_release.stderr)
                    .map_err(|e| ErrorKind::Msg(e.to_string()))?
            );
            bail!("Error running cargo")
        } else {
            println!(
                "{}",
                std::str::from_utf8(&cargo_run_release.stdout)
                    .map_err(|e| ErrorKind::Msg(e.to_string()))?
            );
            Ok(())
        }
    }
}
