#[macro_use]
extern crate error_chain;
pub mod errors {

    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        errors {
            // Interpreter errors
            ScanError(message: String) {
                description("Scan Error")
                display("Scan Error: {}", message)
            }
            ParseError(message: String) {
                description("Parse Error")
                display("Parse Error: {}", message)
            }
            ResolutionError(message: String) {
                description("Resolution Error")
                display("Resolution Error: {}", message)
            }
            RuntimeError(message: String) {
                description("Runtime Error")
                display("Runtime Error: {}", message)
            }
        }

        foreign_links {
            Io(::std::io::Error) #[cfg(unix)];
        }
    }
}
pub use env_logger;
pub use error_chain::bail;
pub use errors::*;
pub use log::*;
use std::io::Write;
pub type Writer<'a> = &'a mut dyn Write;
pub type ByteUnit = u8;

pub fn report_error(message: String, error_writer: Writer) {
    writeln!(error_writer, "{}", message).expect("Write failed");
}

pub fn report_error_with_line(line: usize, message: String, error_writer: Writer) {
    report_error_with_line_and_location(line, "".into(), message, error_writer);
}

pub fn report_error_with_line_and_location(
    line: usize,
    location: String,
    message: String,
    error_writer: Writer,
) {
    report_error(
        format!("[line: {}] Error {}: message: {}", line, location, message),
        error_writer,
    );
}
pub fn utf8_to_string(bytes: &[u8]) -> String {
    match String::from_utf8(bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => String::new(),
    }
}

pub fn print_error(e: Error, error_writer: &mut dyn Write) {
    match e.0 {
        ErrorKind::ScanError(i) => print_error_kind_message("[Scan Error]", &i, error_writer),
        ErrorKind::ParseError(i) => print_error_kind_message("[Parse Error]", &i, error_writer),
        ErrorKind::ResolutionError(i) => {
            print_error_kind_message("[Resolution Error]", &i, error_writer)
        }
        ErrorKind::RuntimeError(i) => print_error_kind_message("[Runtime Error]", &i, error_writer),
        _ => print_error_kind_message("Unknown", &e.to_string(), error_writer),
    };
}

fn print_error_kind_message(kind: &str, message: &str, error_writer: &mut dyn Write) {
    writeln!(error_writer, "{} {}", kind, message).expect("Write failed");
}
