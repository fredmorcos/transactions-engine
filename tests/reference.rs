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

use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use tx_engine::Db;

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
struct Account {
  client: u16,
  available: Decimal,
  held: Decimal,
  total: Decimal,
  locked: bool,
}

fn parse_csv<R: Read>(r: &mut BufReader<R>) -> HashSet<Account> {
  let mut reader = csv::ReaderBuilder::new().from_reader(r);
  let mut res: HashSet<Account> = HashSet::with_capacity(20);
  for account in reader.deserialize() {
    let account = account.unwrap();
    res.insert(account);
  }
  res
}

#[test]
fn reference() {
  for entry in fs::read_dir("tests/data").unwrap() {
    let entry = entry.unwrap();
    let mut file_path = entry.path();
    let file_ext = file_path.extension().unwrap().to_str().unwrap();

    if file_ext == "csv" {
      eprintln!("Testing with {}", file_path.display());

      let db = {
        let input_file = File::open(&file_path).unwrap();
        let mut reader = csv::ReaderBuilder::new()
          .flexible(true)
          .trim(csv::Trim::All)
          .from_reader(input_file);

        let mut db = Db::new();
        for tx in reader.deserialize() {
          let tx = tx.unwrap();
          let _ = db.process(&tx);
        }

        db
      };

      let actual = {
        let mut actual = Vec::with_capacity(1024);
        {
          let writer = BufWriter::new(&mut actual);
          let mut csv_writer = csv::Writer::from_writer(writer);

          for account in db.accounts() {
            csv_writer.serialize(account).unwrap();
          }

          for account in db.accounts_locked() {
            csv_writer.serialize(account).unwrap();
          }

          csv_writer.flush().unwrap();
        }

        let mut reader = BufReader::new(actual.as_slice());
        parse_csv(&mut reader)
      };

      let expected = {
        file_path.set_extension("out");
        let file = File::open(&file_path).unwrap();
        let mut reader = BufReader::new(file);
        parse_csv(&mut reader)
      };

      assert_eq!(expected, actual);
    }
  }
}
