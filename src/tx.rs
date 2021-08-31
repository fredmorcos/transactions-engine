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

use derive_more::Display;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Display)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
  Deposit,
  Withdrawal,
}

#[derive(Serialize, Deserialize, Debug, Display, Clone, Copy)]
#[display(fmt = "{} ID={} Client={} Amount={:?}", typ, tx, client, amount)]
pub struct Tx {
  #[serde(rename = "type")]
  pub typ: TxType,
  pub client: u16,
  pub tx: u32,
  pub amount: Option<Decimal>,
}

impl Tx {
  pub fn new_deposit(tx: u32, client: u16, amount: Decimal) -> Self {
    Self { typ: TxType::Deposit, client, tx, amount: Some(amount) }
  }

  pub fn new_withdraw(tx: u32, client: u16, amount: Decimal) -> Self {
    Self { typ: TxType::Withdrawal, client, tx, amount: Some(amount) }
  }
}
