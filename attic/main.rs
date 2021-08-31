// This file is part of transactions-engine.
//
// transactions-engine is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later version.
//
// transactions-engine is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE.  See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// transactions-engine.  If not, see <https://www.gnu.org/licenses/>.

#![warn(clippy::all)]
#![allow(clippy::upper_case_acronyms)]

use derive_more::{Display, From, Into};
use derive_new::new;
use indexmap::IndexSet;
use log::{debug, error, info, trace, warn};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, HashMap};
use std::fmt;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Display)]
#[display(fmt = "{}")]
enum TxError {
  #[display(fmt = "Invalid negative amount")]
  NegativeAmount,

  #[display(fmt = "Insufficient funds")]
  InsufficientFunds,

  #[display(fmt = "Attempt to access a locked account")]
  AccessLocked,

  #[display(fmt = "Attempt to access a non-existing account")]
  AccessUnavailable,

  #[display(fmt = "Transaction would overflow account")]
  Overflow,

  #[display(fmt = "Duplicate transaction ID")]
  Duplicate,

  #[display(fmt = "Referenced transaction either does not exist or is not a deposit")]
  InvalidDepositRef,

  #[display(fmt = "Referenced deposit transaction is already in dispute")]
  DepositAlreadyInDispute,
}

impl fmt::Debug for TxError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    (self as &dyn fmt::Display).fmt(f)
  }
}

type TxResult = Result<(), TxError>;

/// A client ID is a u16 as defined by the spec. We create a newtype to make it harder to
/// use as a normal u16 value.
#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy)]
#[display(fmt = "Client={}", _0)]
struct ClientID(u16);

/// A transaction ID is a u32 as defined by the spec. We create a newtype to make it
/// harder to use as a normal u32 value.
#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy)]
#[display(fmt = "Tx={}", _0)]
struct TxID(u32);

/// The "normal" state of a deposit transaction.
#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy)]
#[display(fmt = "Released")]
struct DepositReleased;

/// Used when a deposit is being held.
#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy)]
#[display(fmt = "Held")]
struct DepositHeld;

trait DepositTxState {}
impl DepositTxState for DepositReleased {}
impl DepositTxState for DepositHeld {}

#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy)]
#[display(fmt = "Deposit {} <{}> {} Amount={}", id, state, client, amount)]
struct Deposit<State: DepositTxState = DepositReleased> {
  id: TxID,
  client: ClientID,
  amount: Decimal,
  state: State,
}

impl<State: DepositTxState> Deposit<State> {
  fn amount(&self) -> Decimal {
    self.amount
  }
}

impl Deposit<DepositReleased> {
  fn new(id: TxID, client: ClientID, amount: Decimal) -> Self {
    Self { id, client, amount, state: DepositReleased }
  }

  fn dispute(&self) -> Deposit<DepositHeld> {
    Deposit::<DepositHeld> {
      id: self.id,
      client: self.client,
      amount: self.amount,
      state: DepositHeld,
    }
  }
}

impl Deposit<DepositHeld> {
  fn resolve(&self) -> Deposit<DepositReleased> {
    Deposit::<DepositReleased> {
      id: self.id,
      client: self.client,
      amount: self.amount,
      state: DepositReleased,
    }
  }
}

#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy, new)]
#[display(fmt = "Withdrawal {} {} Amount={}", id, client, amount)]
struct Withdrawal {
  id: TxID,
  client: ClientID,
  amount: Decimal,
}

impl Withdrawal {
  fn amount(&self) -> Decimal {
    self.amount
  }
}

#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy, new)]
#[display(fmt = "Dispute {} {}", id, client)]
struct Dispute {
  id: TxID,
  client: ClientID,
}

#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy, new)]
#[display(fmt = "Resolve {} {}", id, client)]
struct Resolve {
  id: TxID,
  client: ClientID,
}

#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy, new)]
#[display(fmt = "Chargeback {} {}", id, client)]
struct Chargeback {
  id: TxID,
  client: ClientID,
}

trait TxType {
  fn id(&self) -> TxID;
  fn client(&self) -> ClientID;
}

impl TxType for Deposit {
  fn id(&self) -> TxID {
    self.id
  }

  fn client(&self) -> ClientID {
    self.client
  }
}

impl TxType for Withdrawal {
  fn id(&self) -> TxID {
    self.id
  }

  fn client(&self) -> ClientID {
    self.client
  }
}

impl TxType for Dispute {
  fn id(&self) -> TxID {
    self.id
  }

  fn client(&self) -> ClientID {
    self.client
  }
}

impl TxType for Resolve {
  fn id(&self) -> TxID {
    self.id
  }

  fn client(&self) -> ClientID {
    self.client
  }
}

impl TxType for Chargeback {
  fn id(&self) -> TxID {
    self.id
  }

  fn client(&self) -> ClientID {
    self.client
  }
}

#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy)]
#[display(fmt = "Active")]
struct AccountActive;

#[derive(Debug, Display, PartialEq, Eq, Hash, From, Into, Clone, Copy)]
#[display(fmt = "Locked")]
struct AccountLocked;

trait AccountState {}
impl AccountState for AccountActive {}
impl AccountState for AccountLocked {}

#[derive(Debug, Display, From, Into, Clone)]
#[display(fmt = "Account ({}) {} Available={} Held={}", state, id, available, held)]
struct Account<State: AccountState = AccountActive> {
  id: ClientID,
  available: Decimal,
  held: Decimal,

  deposits: HashMap<TxID, Deposit>,
  held_deposits: HashMap<TxID, Deposit<DepositHeld>>,
  withdrawals: HashMap<TxID, Withdrawal>,

  state: State,
}

impl Account<AccountActive> {
  fn new(id: ClientID) -> Self {
    Self {
      id,
      available: Decimal::ZERO,
      held: Decimal::ZERO,
      deposits: HashMap::new(),
      held_deposits: HashMap::new(),
      withdrawals: HashMap::new(),
      state: AccountActive,
    }
  }

  fn ensure_valid_amount(amount: Decimal) -> TxResult {
    if amount.is_sign_negative() {
      Err(TxError::NegativeAmount)
    } else {
      Ok(())
    }
  }

  fn deposit(&mut self, id: TxID, client: ClientID, amount: Decimal) -> TxResult {
    Self::ensure_valid_amount(amount)?;

    if self.held_deposits.contains_key(&id) || self.deposits.contains_key(&id) {
      Err(TxError::Duplicate)
    } else {
      if self.total().checked_add(amount).is_none() {
        // Depositing *amount* would overflow the total.
        return Err(TxError::Overflow);
      }

      if let Some(sum) = self.available.checked_add(amount) {
        self.available = sum;
        self.deposits.insert(id, Deposit::new(id, client, amount));
        return Ok(());
      }

      // Depositing *amount* would overflow the available.
      Err(TxError::Overflow)
    }
  }

  fn withdraw(&mut self, id: TxID, client: ClientID, amount: Decimal) -> TxResult {
    Self::ensure_valid_amount(amount)?;

    if amount > self.available {
      Err(TxError::InsufficientFunds)
    } else if let Entry::Vacant(entry) = self.withdrawals.entry(id) {
      self.available -= amount;
      entry.insert(Withdrawal::new(id, client, amount));
      Ok(())
    } else {
      Err(TxError::Duplicate)
    }
  }

  fn ensure_valid_deposit_ref(&self, id: TxID) -> TxResult {
    if self.deposits.contains_key(&id) {
      Ok(())
    } else if self.held_deposits.contains_key(&id) {
      Err(TxError::DepositAlreadyInDispute)
    } else {
      Err(TxError::InvalidDepositRef)
    }
  }

  fn dispute(&mut self, id: TxID, client: ClientID) -> TxResult {
    self.ensure_valid_deposit_ref(id)?;

    let deposit = if let Some(deposit) = self.deposits.remove(&id) {
      deposit
    } else if self.held_deposits.contains_key(&id) {
      return Err(TxError::DepositAlreadyInDispute);
    } else {
      return Err(TxError::InvalidDepositRef);
    };
  }
}

impl<State: AccountState> Account<State> {
  fn id(&self) -> ClientID {
    self.id
  }

  fn available(&self) -> Decimal {
    self.available
  }

  fn held(&self) -> Decimal {
    self.held
  }

  fn total(&self) -> Decimal {
    self.available + self.held
  }
}

#[derive(Debug, new)]
struct Database {
  #[new(default)]
  active_accounts: HashMap<ClientID, Account>,

  #[new(default)]
  locked_accounts: HashMap<ClientID, Account<AccountLocked>>,

  // Ordered-set to preserve the chronological order of transactions.
  #[new(default)]
  tx_ids: IndexSet<TxID>,
}

impl Database {
  fn active_accounts(&self) -> impl Iterator<Item = &Account> {
    self.active_accounts.values()
  }

  fn locked_accounts(&self) -> impl Iterator<Item = &Account<AccountLocked>> {
    self.locked_accounts.values()
  }

  fn get_active_account(&self, id: ClientID) -> Option<&Account> {
    self.active_accounts.get(&id)
  }

  fn get_locked_account(&self, id: ClientID) -> Option<&Account<AccountLocked>> {
    self.locked_accounts.get(&id)
  }

  fn ensure_txid_available(&self, id: TxID) -> TxResult {
    if self.tx_ids.contains(&id) {
      Err(TxError::Duplicate)
    } else {
      Ok(())
    }
  }

  fn ensure_account_not_locked(&self, id: ClientID) -> TxResult {
    if self.locked_accounts.contains_key(&id) {
      Err(TxError::AccessLocked)
    } else {
      Ok(())
    }
  }

  fn deposit(&mut self, id: TxID, client: ClientID, amount: Decimal) -> TxResult {
    self.ensure_txid_available(id)?;
    self.ensure_account_not_locked(client)?;

    if let Some(account) = self.active_accounts.get_mut(&client) {
      account.deposit(id, client, amount)
    } else {
      let mut account = Account::new(client);
      account.deposit(id, client, amount)?;
      self.active_accounts.insert(client, account);
      Ok(())
    }
  }

  fn withdraw(&mut self, id: TxID, client: ClientID, amount: Decimal) -> TxResult {
    self.ensure_txid_available(id)?;
    self.ensure_account_not_locked(client)?;

    if let Some(account) = self.active_accounts.get_mut(&client) {
      account.withdraw(id, client, amount)
    } else {
      Err(TxError::AccessUnavailable)
    }
  }

  fn dispute(&mut self, id: TxID, client: ClientID) -> TxResult {
    self.ensure_account_not_locked(client)?;

    if let Some(account) = self.active_accounts.get_mut(&client) {
      account.dispute(id, client)
    } else {
      Err(TxError::AccessUnavailable)
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Display)]
#[serde(rename_all = "lowercase")]
enum CSVTxType {
  Deposit,
  Withdrawal,
  Dispute,
  Resolve,
  Chargeback,
}

#[derive(Serialize, Deserialize, Debug, Display, Clone, Copy)]
#[display(fmt = "{} ID={} Client={} Amount={:?}", r#type, tx, client, amount)]
struct CSVTx {
  r#type: CSVTxType,
  client: u16,
  tx: u32,
  amount: Option<Decimal>,
}

impl CSVTx {
  fn kind(&self) -> CSVTxType {
    self.r#type
  }
}

#[derive(From, Display)]
#[display(fmt = "{}")]
enum CSVTxError {
  #[display(fmt = "Transaction must provide an amount")]
  MissingAmount,

  #[display(fmt = "Transaction processing error: {}", _0)]
  TxProcessing(TxError),
}

impl fmt::Debug for CSVTxError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    (self as &dyn fmt::Display).fmt(f)
  }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "transactions-engine")]
struct Opt {
  /// Verbose output (can be specified multiple times).
  #[structopt(short, long, parse(from_occurrences))]
  verbose: u8,

  /// Input CSV file.
  #[structopt(name = "FILE", parse(from_os_str))]
  file: PathBuf,
}

#[derive(From, Display)]
#[display(fmt = "{}")]
enum Err {
  #[display(fmt = "IO Error: {}", _0)]
  Io(io::Error),

  #[display(fmt = "CSV Error: {}", _0)]
  CSV(csv::Error),

  #[display(fmt = "Transaction Read Error: {}", _0)]
  CSVTx(CSVTxError),
}

impl fmt::Debug for Err {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    (self as &dyn fmt::Display).fmt(f)
  }
}

fn ensure_amount(tx: &CSVTx) -> Result<Decimal, CSVTxError> {
  match tx.amount {
    Some(amount) => Ok(amount),
    None => Err(CSVTxError::MissingAmount),
  }
}

fn process(db: &mut Database, csv_tx: CSVTx) -> Result<(), CSVTxError> {
  let id = csv_tx.tx.into();
  let client = csv_tx.client.into();

  match csv_tx.kind() {
    CSVTxType::Deposit => {
      let amount = ensure_amount(&csv_tx)?;
      db.deposit(id, client, amount)?;
    }
    CSVTxType::Withdrawal => {
      let amount = ensure_amount(&csv_tx)?;
      db.withdraw(id, client, amount)?;
    }
    CSVTxType::Dispute => {
      db.dispute(id, client)?;
    }
    CSVTxType::Resolve => todo!(),
    CSVTxType::Chargeback => todo!(),
  }

  Ok(())
}

fn main() -> Result<(), Err> {
  let opt = Opt::from_args();

  let log_level = match opt.verbose {
    0 => log::LevelFilter::Off,
    1 => log::LevelFilter::Error,
    2 => log::LevelFilter::Warn,
    3 => log::LevelFilter::Info,
    4 => log::LevelFilter::Debug,
    _ => log::LevelFilter::Trace,
  };

  env_logger::Builder::new().filter_level(log_level).try_init().unwrap_or_else(|e| {
    eprintln!("Error initializing logger: {}", e);
  });

  warn!("Warning output enabled.");
  info!("Info output enabled.");
  debug!("Debug output enabled.");
  trace!("Trace output enabled.");

  let input_file = File::open(opt.file)?;
  let mut csv_reader =
    csv::ReaderBuilder::new().flexible(true).trim(csv::Trim::All).from_reader(input_file);

  let mut db = Database::new();

  // Process transactions.
  'NEXT_TX: for tx in csv_reader.deserialize() {
    let tx: CSVTx = match tx {
      Ok(tx) => tx,
      Err(error) => {
        error!("Transaction reading error: {}", error);
        continue 'NEXT_TX;
      }
    };

    debug!("CSV Transaction {}", tx);

    match process(&mut db, tx) {
      Ok(_) => {}
      Err(err) => {
        error!("Error (transaction skipped): {}", tx);
        error!("  Reason: {}", err);

        if let Some(account) = db.get_active_account(tx.client.into()) {
          error!("  Related Account: {}", account)
        } else if let Some(account) = db.get_locked_account(tx.client.into()) {
          error!("  Related Account: {}", account)
        }
      }
    }
  }

  // Print output header.
  println!("client,available,held,total,locked");

  // Print active accounts.
  for account in db.active_accounts() {
    println!(
      "{},{},{},{},false",
      account.id(),
      account.available(),
      account.held(),
      account.total()
    );
  }

  // Print locked accounts.
  for account in db.locked_accounts() {
    println!(
      "{},{},{},{},true",
      account.id(),
      account.available(),
      account.held(),
      account.total(),
    );
  }

  Ok(())
}
