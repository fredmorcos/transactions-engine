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

use clap::Parser;
use derive_more::{Display, From};
use log::{debug, error, info, trace, warn};
use std::fmt;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use tx_engine::{ClientId, Db, TxErr};

const LICENSE: &str = include_str!("../LICENSE");
const LICENSE_DEPS: &str = include_str!("../LICENSE.dependencies");

#[derive(Debug, clap::Parser)]
#[clap(author, version, about, long_about = None)]
struct Opt {
  /// Verbose output (can be specified multiple times)
  #[clap(short, long, action = clap::ArgAction::Count)]
  verbose: u8,

  /// Show licenses (stops program execution).
  #[clap(short, long)]
  license: bool,

  /// Input CSV file.
  #[clap(name = "FILE")]
  file: PathBuf,
}

#[derive(From, Display)]
#[display(fmt = "{}")]
enum Err {
  #[display(fmt = "IO Error: {}", _0)]
  Io(io::Error),

  #[display(fmt = "CSV Parsing Error: {}", _0)]
  Csv(csv::Error),

  #[display(fmt = "Transaction Processing Error: {}", _0)]
  Tx(TxErr),
}

impl fmt::Debug for Err {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    (self as &dyn fmt::Display).fmt(f)
  }
}

fn main() -> Result<(), Err> {
  let opt = Opt::parse();

  if opt.license {
    eprintln!("{}", LICENSE);
    eprintln!();
    eprintln!("{}", LICENSE_DEPS);
    return Ok(());
  }

  let log_level = match opt.verbose {
    0 => log::LevelFilter::Off,
    1 => log::LevelFilter::Error,
    2 => log::LevelFilter::Warn,
    3 => log::LevelFilter::Info,
    4 => log::LevelFilter::Debug,
    _ => log::LevelFilter::Trace,
  };

  env_logger::Builder::new().filter_level(log_level).try_init().unwrap_or_else(|e| {
    eprintln!("Error initializing logger: {}", e);
  });

  error!("Error output enabled.");
  warn!("Warning output enabled.");
  info!("Info output enabled.");
  debug!("Debug output enabled.");
  trace!("Trace output enabled.");

  let input_file = File::open(opt.file)?;
  let mut reader =
    csv::ReaderBuilder::new().flexible(true).trim(csv::Trim::All).from_reader(input_file);

  let mut db = Db::new();

  'NEXT_TX: for tx in reader.deserialize() {
    let tx = match tx {
      Ok(tx) => tx,
      Err(e) => {
        error!("{}", e);
        continue 'NEXT_TX;
      }
    };

    debug!("CSV Transaction: {}", tx);

    match db.process(&tx) {
      Ok(_) => {}
      Err(err) => {
        error!("Error: Transaction skipped: {}", tx);
        error!("  Reason: {}", err);

        if let Some(account) = db.get_account(ClientId::new(tx.client)) {
          error!("  Related Account: {}", account)
        }
      }
    }
  }

  let mut writer = csv::Writer::from_writer(io::stdout());

  for account in db.accounts() {
    writer.serialize(account)?;
  }

  for account in db.accounts_locked() {
    writer.serialize(account)?;
  }

  writer.flush()?;

  Ok(())
}
