# payments-engine

## Overview

Applies a stream of transactions from a CSV file to client accounts and prints the balances as CSV.

```sh
cargo run -- input.csv > accounts.csv
```

| Column | Used in | Meaning |
|---|---|---|
| `type` | input | Deposit, withdrawal, dispute, resolve or chargeback. |
| `client` | input, output | Client id, a `u16`. |
| `tx` | input | Transaction id, a `u32`. |
| `amount` | input | Decimal with up to four places, needed for deposit and withdrawal. |
| `available` | output | Funds the client can use. |
| `held` | output | Funds held by disputes that are still open. |
| `total` | output | The sum of `available` and `held`, computed when printing. |
| `locked` | output | True after a chargeback, otherwise false. |

- The first argument is the input path and further arguments are ignored; without one the program prints usage on stderr and exits with code 2.
- Only the CSV goes to stdout, one row per client in no particular order, with amounts printed as exact decimals without padding.
- A skipped row is reported on stderr with its row number and the reason, and the run continues; row numbers count data rows from 1, so blank lines or quoted newlines shift them.
- Exit codes: 0 for success, 1 for a fatal error such as an unreadable file, 2 for no path; a fatal error leaves stdout empty.

## Assumptions

- Deposits and withdrawals share one transaction id space, and the first row with an id wins.
- A deposit or withdrawal with a valid amount claims its id before the account checks; if it is then skipped, for example for a locked account or low funds, the id still cannot be reused, because a reusable id could apply one transaction twice. Rows with an invalid amount claim nothing.
- Only a settled deposit of the disputing client can be disputed, resolve and chargeback apply only to a disputed deposit, and any other case is skipped; a resolved deposit can be disputed again.
- A chargeback is final: it locks the account, and that deposit cannot be disputed again.
- A locked account skips deposits and withdrawals, while dispute, resolve and chargeback rows still work, because locked means no new money in or out and disputes describe events that already happened.
- The balances `available` and `total` can go negative, for example when spent funds are disputed, because the deposit was credited and refusing the dispute would hide the exposure.
- A deposit or withdrawal creates its client if it is new; other commands never do.
- Amounts accept digits and one decimal point; more than four decimals skips the row rather than being rounded.
- A deposit or withdrawal needs an amount greater than zero.

## Design

```text
src/
├── main.rs     arguments and exit codes
├── lib.rs      run: read, apply, print
├── input.rs    CSV rows to Command values
├── output.rs   accounts to CSV
├── engine/     PaymentsEngine and its flows
├── store/      AccountStore, TransactionStore
└── types/      Command, Amount, Account, transaction records, errors
```

`PaymentsEngine` matches on each `Command` and calls one plain function per command in these steps.

- Deposit and withdrawal check the amount, claim the transaction id and fetch or create the account; if a later check fails, the transaction stays in `Created`.
- Every flow then checks the state (account unlocked with enough funds, or the disputed transaction fits), computes the new balances and commits the new status and account, the only place balances change.

```text
deposit     Created -> Settled -> Disputed -> Settled | ChargedBack
withdrawal  Created -> Settled
```

## Design decisions

- Chose `Decimal` over integer minor units because it gives exact base-10 arithmetic and a tested parser, at the cost of 16 bytes per amount instead of 8.
- Chose separate `RawRow` and `Command` types over deserializing into the domain type because `RawRow` mirrors the CSV and `Command` holds validated data, so the file format can change without touching the engine or its API.
- Chose one transaction map with a status per record over separate maps because duplicate checks, dispute lookups and the lifecycle share one structure.
- Chose a single owner with no locks over a shared lock-protected map because one writer is simple and needs no synchronization.
- Chose two error types over one, with a `DomainError` variant per rejection reason, because a bad row is then never mistaken for a fatal `AppError` and each skipped row reports its exact cause, which the tests assert.
- Chose store types over raw hash maps in the flows because the storage can change, for example to disk, by editing only `AccountStore` and `TransactionStore`.
- Chose stderr for diagnostics over a log file because stdout must stay clean CSV and a file adds paths and side effects.

## Completeness

- All five transaction types are handled, and fixtures run a full dispute, resolve and chargeback cycle.
- Each ignored dispute case has a test that checks its reason: unknown transaction, another client's transaction, a withdrawal, and a deposit not under dispute.

## Correctness

- 8 unit tests and 15 CLI tests pass.
- The CLI tests run ten fixtures in `tests/data/` with expected outputs, compared numerically in any row order.
- The types enforce the rules: `Amount` cannot hold more than four decimals, and `ClientId` and `TransactionId` cannot be mixed up.

## Safety

- No unsafe code, no unwrap or expect outside tests, and no floats; precision formatting of `Decimal` is avoided because it panics near the 96-bit limit.
- Money arithmetic is checked: an overflow skips the row, including a deposit that would overflow the total; at print time an overflowing total would be left out and logged on stderr.

## Maintainability

- Each flow is a plain function on `PaymentsEngine`, in a file each for deposit, withdrawal and the dispute family.
- The `match` on `Command` is exhaustive with no wildcard, so a new command does not compile until the engine handles it.

## Efficiency

- Input is streamed one record at a time, so the file is never loaded into memory.
- Ids are `u32` and `ClientId` is a `u16`, so memory grows with the transaction count, claimed ids are never released, and there are at most 65,536 accounts.
