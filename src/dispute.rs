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

/// A dispute is a claim of an erroneous transaction by the client.
///
/// A deposit must decrease the available funds and increase held funds.
///
/// # Errors
///
/// * An error is thrown if the [client ID](ClientId) and account do not already exist.
///
/// * An error is thrown if the [transaction ID](TxId) has already been used.
///
/// * An error is thrown if the amount being disputed is larger than the available balance
/// in the client's account.
///
/// # Notes
///
/// * The amount being disputed cannot overflow the held funds since it refers to a
/// pre-existing transaction and it was checked that the available and total funds cannot
/// overflow during the entrance of said transaction.
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, new)]
#[display(fmt = "Dispute {} {}", id, client)]
pub struct Dispute {
  id: TxId,
  client: ClientId,
}

impl Dispute {
  /// Get the dispute's id.
  pub fn id(&self) -> TxId {
    self.id
  }

  /// Get the dispute's client.
  pub fn client(&self) -> ClientId {
    self.client
  }
}
