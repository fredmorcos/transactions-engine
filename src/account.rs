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

use crate::{ClientId, Deposit, DepositHeld, TxErr, TxId, TxResult, Withdraw};
use derive_more::Display;
use derive_new::new;
use rust_decimal::Decimal;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashMap;

/// A client's account.
#[derive(Debug, Display, PartialEq, Eq, new)]
#[display(fmt = "Account {} Available={}", id, available)]
pub struct Account {
  id: ClientId,

  #[new(default)]
  available: Decimal,

  #[new(default)]
  held: Decimal,

  #[new(default)]
  deposits: HashMap<TxId, Deposit>,

  #[new(default)]
  withdraws: HashMap<TxId, Withdraw>,

  #[new(default)]
  deposits_held: HashMap<TxId, Deposit<DepositHeld>>,
}

impl Account {
  pub fn id(&self) -> ClientId {
    self.id
  }

  pub fn available(&self) -> Decimal {
    self.available
  }

  pub fn held(&self) -> Decimal {
    self.held
  }

  pub fn total(&self) -> Decimal {
    self.available + self.held
  }

  pub(crate) fn deposit(&mut self, tx: Deposit) -> TxResult {
    assert_eq!(self.id, tx.client());

    if self.total().checked_add(tx.amount()).is_none() {
      // Depositing *amount* would overflow the total.
      return Err(TxErr::Overflow);
    }

    if let Some(sum) = self.available.checked_add(tx.amount()) {
      self.available = sum;
      // The database ensures that the transaction ID is not a duplicate.
      self.deposits.insert(tx.id(), tx);
      return Ok(());
    }

    // Depositing *amount* would overflow the available.
    Err(TxErr::Overflow)
  }

  pub(crate) fn withdraw(&mut self, tx: Withdraw) -> TxResult {
    assert_eq!(self.id, tx.client());

    if tx.amount() > self.available {
      return Err(TxErr::Insufficient);
    }

    // The database ensures that the transaction ID is not a duplicate.
    self.withdraws.insert(tx.id(), tx);
    self.available -= tx.amount();

    Ok(())
  }

  pub(crate) fn dispute(&mut self, tx: crate::Dispute) -> TxResult {
    assert_eq!(self.id, tx.client());

    let id = tx.id();

    let deposit = match self.deposits.remove(&id) {
      Some(deposit) => deposit,
      None => return Err(TxErr::MissingTxForClient),
    };

    assert!(!self.deposits_held.contains_key(&id));

    if deposit.amount() > self.available {
      self.deposits.insert(id, deposit);
      return Err(TxErr::Insufficient);
    }

    self.available -= deposit.amount();
    self.held += deposit.amount();

    self.deposits_held.insert(id, deposit.hold());

    Ok(())
  }
}

impl Serialize for Account {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("Account", 5)?;
    state.serialize_field("client", &self.id())?;
    state.serialize_field("available", &self.available())?;
    state.serialize_field("held", &self.held())?;
    state.serialize_field("total", &self.total())?;
    state.serialize_field("locked", &false)?;
    state.end()
  }
}

#[cfg(test)]
mod account_tests {
  use crate::{Account, ClientId, Deposit, TxId, Withdraw};

  #[test]
  fn deposits_withdraws() {
    let client = ClientId::new(1);
    let mut account = Account::new(client);

    let tx = Deposit::new(TxId::new(1), client, 5.into()).unwrap();
    assert_eq!(account.deposit(tx), Ok(()));
    assert_eq!(account.total(), 5.into());
    assert_eq!(account.available(), 5.into());
    assert_eq!(account.held(), 0.into());

    let tx = Deposit::new(TxId::new(2), client, 5.into()).unwrap();
    assert_eq!(account.deposit(tx), Ok(()));
    assert_eq!(account.total(), 10.into());
    assert_eq!(account.available(), 10.into());
    assert_eq!(account.held(), 0.into());

    let tx = Withdraw::new(TxId::new(3), client, 6.into()).unwrap();
    assert_eq!(account.withdraw(tx), Ok(()));
    assert_eq!(account.total(), 4.into());
    assert_eq!(account.available(), 4.into());
    assert_eq!(account.held(), 0.into());
  }

  #[test]
  #[should_panic]
  fn deposit_invalid_client() {
    let client1 = ClientId::new(1);
    let client2 = ClientId::new(2);
    let mut account = Account::new(client1);
    let tx = Deposit::new(TxId::new(1), client2, 5.into()).unwrap();

    // The call to deposit() should fail the client ID assertion.
    assert_eq!(account.deposit(tx), Ok(()));
  }

  #[test]
  #[should_panic]
  fn withdraw_invalid_client() {
    let client1 = ClientId::new(1);
    let client2 = ClientId::new(2);
    let mut account = Account::new(client1);
    let tx = Withdraw::new(TxId::new(1), client2, 5.into()).unwrap();

    // The call to withdraw() should fail the client ID assertion.
    assert_eq!(account.withdraw(tx), Ok(()));
  }
}
