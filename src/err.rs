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

#[derive(Display, Debug, PartialEq, Eq)]
#[display(fmt = "{}")]
pub enum TxErr {
  #[display(fmt = "Transaction must provide an amount")]
  MissingAmount,

  #[display(fmt = "Invalid negative amount")]
  NegativeAmount,

  #[display(fmt = "Insufficient funds")]
  Insufficient,

  #[display(fmt = "Attempt to access a non-existing account")]
  AccessUnavailable,

  #[display(fmt = "Transaction would overflow account")]
  Overflow,

  #[display(fmt = "Duplicate transaction ID")]
  Duplicate,
}

pub type TxResult = Result<(), TxErr>;
