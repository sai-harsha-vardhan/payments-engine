use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use payments_engine::run;
use payments_engine::types::errors::AppError;

fn main() -> ExitCode {
    if let Some(path) = env::args_os().nth(1) {
        match execute(PathBuf::from(path)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                report(error);
                ExitCode::FAILURE
            }
        }
    } else {
        report(AppError::MissingArgument);
        ExitCode::from(2)
    }
}

// Ignore a failed write: if stderr is gone there is nowhere left to report, and eprintln! would panic.
fn report(error: AppError) {
    let _ = writeln!(io::stderr(), "{error}");
}

fn execute(path: PathBuf) -> Result<(), AppError> {
    let input = BufReader::new(File::open(path)?);
    let output = BufWriter::new(io::stdout().lock());
    let stderr = BufWriter::new(io::stderr().lock());
    run(input, output, stderr)
}
