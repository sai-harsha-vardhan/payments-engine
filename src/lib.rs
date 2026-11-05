mod engine;
mod input;
mod output;
mod store;
pub mod types;

use std::io::{Read, Write};

use engine::PaymentsEngine;
use input::{RowError, read_commands};
use output::write_accounts;
use types::errors::AppError;

pub fn run<R: Read, W: Write, D: Write>(input: R, output: W, mut diag: D) -> Result<(), AppError> {
    let mut payments_engine = PaymentsEngine::new();
    for item in read_commands(input)? {
        match item {
            Ok((row, command)) => {
                // Try to apply the row command, in case of any failure write to std err and progress further
                if let Err(error) = payments_engine.apply(command) {
                    let _ = writeln!(diag, "row {row}: {error}");
                }
            }
            // Invalid rows are skipped, and errors are not propagated
            Err(RowError::Invalid { row, reason }) => {
                let _ = writeln!(diag, "row {row}: {reason}");
            }
            Err(RowError::Fatal(error)) => return Err(error),
        }
    }
    write_accounts(output, payments_engine.accounts(), &mut diag)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_str(input: &str) -> (Result<(), AppError>, String, String) {
        let (mut output, mut stderr) = (Vec::new(), Vec::new());
        let result = run(input.as_bytes(), &mut output, &mut stderr);
        (
            result,
            String::from_utf8(output).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    }

    #[test]
    fn run_skips_and_reports_bad_rows() {
        let input = "type,client,tx,amount\n\
                     deposit,1,1,2\n\
                     bogus,1,2,1\n\
                     withdrawal,1,3,5\n\
                     dispute,1,99\n";
        let (result, output, stderr) = run_str(input);
        assert!(result.is_ok());
        assert_eq!(
            output,
            "client,available,held,total,locked\n1,2,0,2,false\n"
        );
        let lines: Vec<_> = stderr.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("row 2: "));
        assert_eq!(lines[1], "row 3: insufficient available funds");
        assert_eq!(lines[2], "row 4: unknown transaction id");
    }
}
