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

use crate::{
  Account, ClientId, Deposit, Dispute, Tx, TxErr, TxId, TxResult, TxType, Withdraw,
};
use derive_new::new;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

/// Database of accounts.
#[derive(Debug, new)]
pub struct Db {
  #[new(default)]
  accounts: HashMap<ClientId, Account>,

  #[new(default)]
  tx_ids: HashSet<TxId>,
}

impl Db {
  pub fn accounts(&self) -> impl Iterator<Item = &Account> {
    self.accounts.values()
  }

  pub fn get_account(&self, id: ClientId) -> Option<&Account> {
    self.accounts.get(&id)
  }

  pub fn process(&mut self, tx: &Tx) -> TxResult {
    fn ensure_amount(tx: &Tx) -> Result<Decimal, TxErr> {
      match tx.amount {
        Some(amount) => Ok(amount),
        None => Err(TxErr::MissingAmount),
      }
    }

    fn ensure_no_amount(tx: &Tx) -> TxResult {
      match tx.amount {
        Some(_) => Err(TxErr::ExtraneousAmount),
        None => Ok(()),
      }
    }

    let id = TxId::new(tx.tx);
    let client = ClientId::new(tx.client);

    match tx.typ {
      TxType::Deposit => {
        let amount = ensure_amount(tx)?;
        self.deposit(id, client, amount)
      }
      TxType::Withdrawal => {
        let amount = ensure_amount(tx)?;
        self.withdraw(id, client, amount)
      }
      TxType::Dispute => {
        ensure_no_amount(tx)?;
        self.dispute(id, client)
      }
    }
  }

  fn deposit(&mut self, id: TxId, client: ClientId, amount: Decimal) -> TxResult {
    let tx = Deposit::new(id, client, amount)?;

    if self.tx_ids.contains(&id) {
      return Err(TxErr::Duplicate);
    }

    if let Some(account) = self.accounts.get_mut(&client) {
      account.deposit(tx)?;
      self.tx_ids.insert(id);
    } else {
      let mut account = Account::new(client);
      account.deposit(tx)?;
      self.tx_ids.insert(id);
      self.accounts.insert(client, account);
    }

    Ok(())
  }

  fn withdraw(&mut self, id: TxId, client: ClientId, amount: Decimal) -> TxResult {
    let tx = Withdraw::new(id, client, amount)?;

    if self.tx_ids.contains(&id) {
      return Err(TxErr::Duplicate);
    }

    if let Some(account) = self.accounts.get_mut(&client) {
      account.withdraw(tx)?;
      self.tx_ids.insert(id);
      Ok(())
    } else {
      Err(TxErr::AccessUnavailable)
    }
  }

  fn dispute(&mut self, id: TxId, client: ClientId) -> TxResult {
    let tx = Dispute::new(id, client);

    if !self.tx_ids.contains(&id) {
      return Err(TxErr::MissingTx);
    }

    if let Some(account) = self.accounts.get_mut(&client) {
      account.dispute(tx)
    } else {
      Err(TxErr::AccessUnavailable)
    }
  }
}

#[cfg(test)]
mod db_tests {
  use crate::{Db, Tx, TxErr};
  use rust_decimal::Decimal;

  #[test]
  fn valid_transactions() {
    let mut db = Db::new();
    assert_eq!(db.process(&Tx::new_deposit(5, 1, Decimal::from(5))), Ok(()));
    assert_eq!(db.process(&Tx::new_deposit(4, 1, Decimal::from(5))), Ok(()));
    assert_eq!(db.process(&Tx::new_deposit(3, 2, Decimal::from(5))), Ok(()));
    assert_eq!(db.process(&Tx::new_withdraw(2, 1, Decimal::from(10))), Ok(()));
    assert_eq!(db.process(&Tx::new_withdraw(1, 2, Decimal::from(5))), Ok(()));
  }

  #[test]
  fn duplicate_tx_id() {
    let mut db = Db::new();
    assert_eq!(db.process(&Tx::new_deposit(4, 1, Decimal::from(5))), Ok(()));
    assert_eq!(
      db.process(&Tx::new_withdraw(4, 1, Decimal::from(5))),
      Err(TxErr::Duplicate)
    );
  }

  #[test]
  fn invalid_withdraw() {
    let mut db = Db::new();
    assert_eq!(db.process(&Tx::new_deposit(5, 1, Decimal::from(5))), Ok(()));
    assert_eq!(db.process(&Tx::new_deposit(4, 1, Decimal::from(5))), Ok(()));
    assert_eq!(db.process(&Tx::new_deposit(3, 2, Decimal::from(5))), Ok(()));
    assert_eq!(
      db.process(&Tx::new_withdraw(2, 1, Decimal::from(15))),
      Err(TxErr::Insufficient)
    );
    assert_eq!(db.process(&Tx::new_withdraw(1, 2, Decimal::from(5))), Ok(()));
  }
}
