use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::str::FromStr;

use rust_decimal::Decimal;

type Accounts = BTreeMap<u16, (Decimal, Decimal, Decimal, bool)>;

fn data(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(file)
}

fn run_binary(args: &[PathBuf]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_payments-engine"))
        .args(args)
        .output()
        .unwrap()
}

/// Parses account CSV, comparing money as Decimal values. `#` lines are comments.
fn parse_accounts(text: &str) -> Accounts {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .comment(Some(b'#'))
        .from_reader(text.as_bytes());
    let headers: Vec<_> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_owned)
        .collect();
    assert_eq!(headers, ["client", "available", "held", "total", "locked"]);

    let mut accounts = Accounts::new();
    for record in reader.records() {
        let fields = record.unwrap();
        let money = |index: usize| Decimal::from_str(&fields[index]).unwrap();
        let row = (
            money(1),
            money(2),
            money(3),
            fields[4].parse::<bool>().unwrap(),
        );
        let client = fields[0].parse().unwrap();
        assert!(
            accounts.insert(client, row).is_none(),
            "client {client} repeated"
        );
    }
    accounts
}

fn check(name: &str) {
    let output = run_binary(&[data(&format!("{name}.csv"))]);
    assert!(output.status.success(), "{name}: {:?}", output.status);
    let expected = std::fs::read_to_string(data(&format!("{name}.expected.csv"))).unwrap();
    assert_eq!(
        parse_accounts(&String::from_utf8(output.stdout).unwrap()),
        parse_accounts(&expected),
        "{name}"
    );
}

macro_rules! fixtures {
    ($($test:ident => $name:literal),* $(,)?) => {
        $(#[test] fn $test() { check($name); })*
    };
}

fixtures! {
    basic => "01_basic",
    insufficient_funds => "02_insufficient_funds",
    duplicate_ids => "03_duplicate_ids",
    dispute_cycle => "04_dispute_cycle",
    after_chargeback => "05_after_chargeback",
    ignored_disputes => "06_ignored_disputes",
    negative_available => "07_negative_available",
    whitespace_and_decimals => "08_whitespace_decimals",
    bad_rows => "09_bad_rows",
    rejected_withdrawal_only => "10_rejected_withdrawal_only",
}

#[test]
fn no_argument_prints_usage_and_exits_2() {
    let output = run_binary(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage"));
}

#[test]
fn extra_arguments_are_ignored() {
    let output = run_binary(&[data("01_basic.csv"), data("does_not_exist.csv")]);
    assert!(output.status.success());
    let expected = std::fs::read_to_string(data("01_basic.expected.csv")).unwrap();
    assert_eq!(
        parse_accounts(&String::from_utf8(output.stdout).unwrap()),
        parse_accounts(&expected)
    );
}

#[test]
fn missing_file_exits_1() {
    let output = run_binary(&[data("does_not_exist.csv")]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
}

#[test]
fn bad_rows_are_reported_on_stderr_and_stdout_stays_csv() {
    let output = run_binary(&[data("09_bad_rows.csv")]);
    let stderr = String::from_utf8(output.stderr).unwrap();
    let lines: Vec<_> = stderr.lines().collect();
    // abc client, unknown type, missing amount (empty and short), 5 decimals, zero, two negatives
    assert_eq!(lines.len(), 8, "{stderr}");
    assert!(lines.iter().all(|l| l.starts_with("row ")), "{stderr}");
    parse_accounts(&String::from_utf8(output.stdout).unwrap());
}

#[test]
fn ignored_disputes_are_reported_with_their_reason() {
    // Rows of 06_ignored_disputes.csv that do nothing, and why (row N = Nth data row).
    let expected = "\
row 4: unknown transaction id
row 5: transaction cannot be disputed
row 6: transaction belongs to another client
row 7: transaction is not under dispute
row 8: transaction is not under dispute
row 12: account is locked
row 13: transaction cannot be disputed
";
    let output = run_binary(&[data("06_ignored_disputes.csv")]);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), expected);
}

/// Generates `total` rows on demand so the input never sits in memory.
struct GeneratedRows {
    next: u64,
    total: u64,
    header_sent: bool,
    pending: Vec<u8>,
    pos: usize,
}

impl Read for GeneratedRows {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pos == self.pending.len() {
            self.pending.clear();
            self.pos = 0;
            if !self.header_sent {
                self.header_sent = true;
                self.pending.extend_from_slice(b"type,client,tx,amount\n");
            } else if self.next < self.total {
                let index = self.next;
                self.next += 1;
                let (kind, amount) = if index % 2 == 0 {
                    ("deposit", "1.0001")
                } else {
                    ("withdrawal", "0.5")
                };
                self.pending.extend_from_slice(
                    format!("{kind},{},{index},{amount}\n", index % 1000).as_bytes(),
                );
            } else {
                return Ok(0);
            }
        }
        let copied = buf.len().min(self.pending.len() - self.pos);
        buf[..copied].copy_from_slice(&self.pending[self.pos..self.pos + copied]);
        self.pos += copied;
        Ok(copied)
    }
}

/// Run with `cargo test -- --ignored`.
#[test]
#[ignore]
fn streams_a_million_rows() {
    let rows = GeneratedRows {
        next: 0,
        total: 1_000_000,
        header_sent: false,
        pending: Vec::new(),
        pos: 0,
    };
    let mut output = Vec::new();
    payments_engine::run(rows, &mut output, std::io::sink()).unwrap();
    assert_eq!(output.iter().filter(|&&b| b == b'\n').count(), 1001);
}
