# `transactions-engine`

[Github Repository](https://github.com/fredmorcos/transactions-engine)

## Licensing

This software is licensed under [the GPLv3
license](https://choosealicense.com/licenses/gpl-3.0/) (see the LICENSE
file). Dependencies of this software are licensed under [the MIT
license](https://choosealicense.com/licenses/mit/) (see the LICENSE.dependencies file).

## Running

`transactions-engine` uses subcommands.

The `license` subcommand (using `./transactions-engine license` or `cargo run -- license`)
will print out the different licenses related to the projects.

The `process` subcommand (using `./transactions-engine process` or `cargo run -- process`)
is the main command, used to process the input CSV file containing transactions.

## Error Handling

The library ignores a transaction and throws an error when an invalid case is detected and
leaves it up to user code to handle. The executable in this particular implementation
prints out an error message and continues to operate normally. The assumption here is that
the engine should not shut down in case of faulty input (e.g. invalid transactions).

The specification is unclear on how to handle errors. One example is what the behavior
should be around insufficient funds during withdrawal: The specification mentions "If a
client does not have sufficient available funds the withdrawal should fail" but does not
define what failing here means. It goes on to mention "and the total amount of funds
should not change" which leads one to understand that failing here means silently ignoring
the transaction (or perhaps only print a warning message).

### Printing Errors and the Verbosity Flag

Since it is requested that the tool not have any other output than the result account
statuses and balances, printing out errors and ignored transactions is disabled by default
and will only be enabled when at least one level of `-v` (verbose) flags is passed. This
is also why most of this document uses the sentence "invalid transactions throw an error
and are silently ignored", which might be confusing at first.

### Malformed Transactions

Malformed transactions are silently ignored and will cause the executable to print an
error but continue to operate. Examples of malformed transactions are deposits/withdrawals
without a specified amount, or disputes/resolves/chargebacks with a specified amount.

### Overflows

Large deposits which would overflow an account balance print an error and are silently
ignored.

## Assumptions

### Non-deposit operations on non-existing Accounts

The assumption is that - along with most other cases - such transactions should be
silently ignored (and perhaps print out a warning message). That is as opposed to
e.g. creating an account with a negative balance in the case of a withdrawal on a
non-existing account.

### Resolves and Chargebacks

Resolves and chargebacks are treated as different ways to end a dispute. The specification
is not clear on the relationship between the two. In this implementation, resolves are
used to end a dispute when the dispute is unfounded and the state of the account should go
back to normal operation, while chargebacks are used when the dispute is in fact founded
and the client should receive a payout - reversing the deposit and locking their account.

## Known shortcomings

### The `Tx` Type

The `Tx` type is not ideal but is necessary. I could not figure out how to get the `csv`
crate to work with `serde`'s tagged enums, see [this
issue](https://github.com/BurntSushi/rust-csv/issues/211).

This, unfortunately, requires us to do some manual type-checking: one example is to make
sure deposits and withdrawals come with an specified amount, and other types of
transactions don't. With tagged enums, such a check would be done by serde.

The following would have been possible with internally tagged enums:

```rust
#[derive(Serialize, Deserialize, Debug)]
struct CommonTransactionInfo {
  client: u16,
  tx: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase", tag = "type")]
enum Transaction {
  Deposit {
    #[serde(flatten)]
    info: CommonTransactionInfo,
    amount: Decimal,
  },
  Withdrawal {
    #[serde(flatten)]
    info: CommonTransactionInfo,
    amount: Decimal,
  },
  Dispute(CommonTransactionInfo),
  Resolve(CommonTransactionInfo),
  Chargeback(CommonTransactionInfo),
}
```

Notice how `Deposit` and `Withdrawal` have the `amount` field without requiring an
`Option<Decimal>` and the other variants can safely avoid having the field altogether.

### Disputes

Disputes only refer to deposits. I do not see how disputes could work for e.g. withdrawals
without human intervention (as the specification mentions: think of an ATM withdrawal).

### Assertions

Assertions are used to ensure programming errors (e.g. invalid state) end the execution of
the program. However, due to Rust's semantics around assertions in multi-threaded
contexts, this might not actually happen and might instead just leave a program running
with invalid state. That means this library is not fit to be used as part of a
multi-threaded application in case a bug triggers an assertion failure.

### Documentation and Testing

There is unfortunately not much in the way of code documentation and extensive tests. The
different ways an operation can fail - along with some notes - are documented with their
respective structs. Small unit and integration tests are available for simple scenarios.

### Operation structs are more bloated than necessary

Operation structs (e.g. Deposit, Withdraw, etc...) don't really need to contain the client
ID and the transaction ID and can just be thin wrappers around their values and
states. This would simplify quite a few things, and would also reduce the number of
required assertions. However, I am leaving them the way they currently are for good
measure.
