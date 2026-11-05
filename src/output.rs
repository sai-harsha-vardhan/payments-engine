use std::io::Write;

use crate::types::account::{Account, ClientId};
use crate::types::errors::AppError;

pub fn write_accounts<'a, W: Write>(
    output: W,
    accounts: impl Iterator<Item = (&'a ClientId, &'a Account)>,
    stderr: &mut impl Write,
) -> Result<(), AppError> {
    let mut writer = csv::Writer::from_writer(output);
    writer.write_record(["client", "available", "held", "total", "locked"])?;

    for (client, account) in accounts {
        // compute the total amount for every client account, if it overflows skip the record
        if let Some(total) = account.total() {
            writer.write_record([
                client.0.to_string(),
                account.available.to_string(),
                account.held.to_string(),
                total.to_string(),
                account.locked.to_string(),
            ])?;
        } else {
            let _ = writeln!(stderr, "client {}: total overflows, row skipped", client.0);
        }
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_map_writes_header_only() {
        let mut output = Vec::new();
        write_accounts(&mut output, std::iter::empty(), &mut std::io::sink()).unwrap();
        assert_eq!(output, b"client,available,held,total,locked\n");
    }
}
