use std::io::Read;

use csv::{ErrorKind, StringRecord, Trim};
use serde::Deserialize;

use crate::types::Command;
use crate::types::account::ClientId;
use crate::types::amount::Amount;
use crate::types::errors::{AppError, DomainError};
use crate::types::transaction::TransactionId;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum RowType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Deserialize)]
struct RawRow {
    #[serde(rename = "type")]
    kind: RowType,
    client: ClientId,
    #[serde(rename = "tx")]
    transaction_id: TransactionId,
    amount: Option<Amount>,
}

impl TryFrom<RawRow> for Command {
    type Error = DomainError;

    fn try_from(raw: RawRow) -> Result<Self, DomainError> {
        let (client, transaction_id) = (raw.client, raw.transaction_id);
        match raw.kind {
            RowType::Deposit => Ok(Command::Deposit {
                client,
                transaction_id,
                amount: raw.amount.ok_or(DomainError::MissingAmount)?,
            }),
            RowType::Withdrawal => Ok(Command::Withdrawal {
                client,
                transaction_id,
                amount: raw.amount.ok_or(DomainError::MissingAmount)?,
            }),
            RowType::Dispute => Ok(Command::Dispute {
                client,
                transaction_id,
            }),
            RowType::Resolve => Ok(Command::Resolve {
                client,
                transaction_id,
            }),
            RowType::Chargeback => Ok(Command::Chargeback {
                client,
                transaction_id,
            }),
        }
    }
}

pub enum RowError {
    Invalid { row: u64, reason: String },
    Fatal(AppError),
}

const REQUIRED_COLUMNS: [&str; 4] = ["type", "client", "tx", "amount"];

/// Streams validated commands, one CSV record at a time
pub fn read_commands<R: Read>(
    input: R,
) -> Result<impl Iterator<Item = Result<(u64, Command), RowError>>, AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(input);

    let headers: StringRecord = reader
        .headers()?
        .iter()
        .map(str::to_ascii_lowercase)
        .collect();
    // An empty file has no header and no rows, which is valid
    if !headers.is_empty() {
        if let Some(name) = REQUIRED_COLUMNS
            .into_iter()
            .find(|name| !headers.iter().any(|header| header == *name))
        {
            return Err(AppError::MissingColumn(name));
        }
        reader.set_headers(headers);
    }

    Ok(reader
        .into_deserialize::<RawRow>()
        .zip(1..)
        .map(|(raw, row)| parse_row(raw, row)))
}

fn parse_row(raw: csv::Result<RawRow>, row: u64) -> Result<(u64, Command), RowError> {
    let raw = raw.map_err(|error| match error.kind() {
        ErrorKind::Io(_) => RowError::Fatal(error.into()),
        _ => RowError::Invalid {
            row,
            reason: error.to_string(),
        },
    })?;
    let command = Command::try_from(raw).map_err(|error| RowError::Invalid {
        row,
        reason: error.to_string(),
    })?;
    Ok((row, command))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "type,client,tx,amount\n";

    fn read(input: &str) -> Vec<Result<(u64, Command), RowError>> {
        read_commands(input.as_bytes()).unwrap().collect()
    }

    #[test]
    fn bad_rows_are_skipped_and_reading_continues() {
        let bad = [
            "deposit,1,2",
            "deposit,1,2,",
            "deposit,1,2,1.23456",
            "dispute,1,2,abc",
            "bogus,1,2,1",
            "deposit,abc,2,1",
            "deposit,65536,2,1",
            "deposit,1,4294967296,1",
        ];
        for row in bad {
            let rows = read(&format!("{HEADER}{row}\ndispute,1,2\n"));
            assert!(
                matches!(rows[0], Err(RowError::Invalid { row: 1, .. })),
                "{row}"
            );
            assert!(matches!(rows[1], Ok((2, Command::Dispute { .. }))), "{row}");
        }
    }

    #[test]
    fn unreadable_or_incomplete_header_is_an_error() {
        let inputs: [&[u8]; 3] = [
            b"type,client,tx,amount\xff\ndeposit,1,1,5\n",
            b"deposit,1,1,5\n",
            b"type,client,tx\ndeposit,1,1\n",
        ];
        for input in inputs {
            assert!(read_commands(input).is_err());
        }
    }

    #[test]
    fn empty_input_has_no_rows() {
        assert!(read("").is_empty());
        assert!(read(HEADER).is_empty());
    }
}
