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
use derive_new::new;
use serde::Serialize;

/// A client ID is a u16 as defined by the spec.
///
/// We use a newtype to make it harder to use as a normal u16 value.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Display, new)]
#[display(fmt = "Client={}", _0)]
pub struct ClientId(u16);

/// A transaction ID is a u32 as defined by the spec.
///
/// We use a newtype to make it harder to use as a normal u32 value.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Display, new)]
#[display(fmt = "Tx={}", _0)]
pub struct TxId(u32);
