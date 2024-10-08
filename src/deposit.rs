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

use crate::{ClientId, TxErr, TxId};
use derive_more::Display;
use rust_decimal::Decimal;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DepositHeld;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DepositReleased;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DepositReversed;

pub trait DepositState {}
impl DepositState for DepositHeld {}
impl DepositState for DepositReleased {}
impl DepositState for DepositReversed {}

/// A deposit is a credit to the client's account.
///
/// A deposit must increase the available (and total) funds in the account.
///
/// # Notes
///
/// * If the [client ID](ClientId) and account do not already exist, they must be created.
///
/// # Errors
///
/// * An error is thrown if the [transaction ID](TxId) has already been used.
///
/// * An error is thrown if the amount is negative.
///
/// * An error is thrown if the amount being deposited would overflow the account's total
///   or available balance.
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy)]
#[display(fmt = "Deposit {} {} Amount={}", id, client, amount)]
pub struct Deposit<State: DepositState = DepositReleased> {
  id: TxId,
  client: ClientId,
  amount: Decimal,
  state: State,
}

impl<State: DepositState> Deposit<State> {
  /// Get the deposit's id.
  pub fn id(&self) -> TxId {
    self.id
  }

  /// Get the deposit's client.
  pub fn client(&self) -> ClientId {
    self.client
  }

  /// Get the deposit's amount.
  pub fn amount(&self) -> Decimal {
    self.amount
  }
}

impl Deposit<DepositReleased> {
  pub fn new(id: TxId, client: ClientId, amount: Decimal) -> Result<Self, TxErr> {
    if amount.is_sign_negative() {
      Err(TxErr::NegativeAmount)
    } else {
      Ok(Self { id, client, amount, state: DepositReleased })
    }
  }

  pub fn hold(self) -> Deposit<DepositHeld> {
    Deposit::<DepositHeld> {
      id: self.id,
      client: self.client,
      amount: self.amount,
      state: DepositHeld,
    }
  }
}

impl Deposit<DepositHeld> {
  pub fn release(self) -> Deposit<DepositReleased> {
    Deposit::<DepositReleased> {
      id: self.id,
      client: self.client,
      amount: self.amount,
      state: DepositReleased,
    }
  }

  pub fn reverse(self) -> Deposit<DepositReversed> {
    Deposit::<DepositReversed> {
      id: self.id,
      client: self.client,
      amount: self.amount,
      state: DepositReversed,
    }
  }
}

#[cfg(test)]
mod deposit_tests {
  use crate::{deposit::DepositReleased, ClientId, Deposit, TxErr, TxId};
  use rust_decimal::Decimal;

  #[test]
  fn positive_amount() {
    let tx_id = TxId::new(1);
    let client_id = ClientId::new(1);
    let amount = Decimal::from(5);

    assert_eq!(
      Deposit::new(tx_id, client_id, amount),
      Ok(Deposit { id: tx_id, client: client_id, amount, state: DepositReleased })
    );
  }

  #[test]
  fn negative_amount() {
    assert_eq!(
      Deposit::new(TxId::new(1), ClientId::new(1), Decimal::from(-5)),
      Err(TxErr::NegativeAmount)
    );
  }
}
