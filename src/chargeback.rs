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

use crate::{ClientId, TxId};
use derive_more::Display;
use derive_new::new;

/// A chargeback is a resolution of a dispute by reversing a transaction.
///
/// A chargeback must decrease the held funds without increasing the available funds.
///
/// # Errors
///
/// * An error is thrown if the [client ID](ClientId) and account do not already exist.
///
/// * An error is thrown if the [transaction ID](TxId) does not already exist.
///
/// * An error is thrown if the [transaction ID](TxId) does not refer to a held deposit
/// transaction associated with [the corresponding client](ClientId).
///
/// # Notes
///
/// * The amount being charged back can only be less than or equal to the held funds.
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, new)]
#[display(fmt = "Chargeback {} {}", id, client)]
pub struct Chargeback {
  id: TxId,
  client: ClientId,
}

impl Chargeback {
  /// Get the chargeback's id.
  pub fn id(&self) -> TxId {
    self.id
  }

  /// Get the chargeback's client.
  pub fn client(&self) -> ClientId {
    self.client
  }
}
