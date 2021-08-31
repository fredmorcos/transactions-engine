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

/// A withdrawal is a debit to the client's account.
///
/// A withdrawal must decrease the available (and total) funds in the account.
///
/// # Errors
///
/// * An error is thrown if the [client ID](ClientId) and account do not already exist.
///
/// * An error is thrown if the [transaction ID](TxId) has already been used.
///
/// * An error is thrown if the amount is negative.
///
/// * An error is thrown if the amount being withdrawn is more than the available balance
/// in the client's account.
#[derive(Debug, Display, PartialEq, Eq, Clone, Copy)]
#[display(fmt = "Withdrawal {} {} Amount={}", id, client, amount)]
pub struct Withdraw {
  id: TxId,
  client: ClientId,
  amount: Decimal,
}

impl Withdraw {
  pub fn new(id: TxId, client: ClientId, amount: Decimal) -> Result<Self, TxErr> {
    if amount.is_sign_negative() {
      Err(TxErr::NegativeAmount)
    } else {
      Ok(Self { id, client, amount })
    }
  }

  /// Get the withdraw's id.
  pub fn id(&self) -> TxId {
    self.id
  }

  /// Get the withdraw's client.
  pub fn client(&self) -> ClientId {
    self.client
  }

  /// Get the withdraw's amount.
  pub fn amount(&self) -> Decimal {
    self.amount
  }
}

#[cfg(test)]
mod withdraw_tests {
  use crate::{ClientId, TxErr, TxId, Withdraw};
  use rust_decimal::Decimal;

  #[test]
  fn positive_amount() {
    let tx_id = TxId::new(1);
    let client_id = ClientId::new(1);
    let amount = Decimal::from(5);

    assert_eq!(
      Withdraw::new(tx_id, client_id, amount),
      Ok(Withdraw { id: tx_id, client: client_id, amount })
    );
  }

  #[test]
  fn negative_amount() {
    assert_eq!(
      Withdraw::new(TxId::new(1), ClientId::new(1), Decimal::from(-5)),
      Err(TxErr::NegativeAmount)
    );
  }
}
