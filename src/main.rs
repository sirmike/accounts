use std::env;
use csv::{Error, Reader, ReaderBuilder, Trim};
use std::fs::File;
use std::collections::HashMap;
use std::io;

mod transactions;
mod enums;
mod structs;

use transactions::processor;
use structs::Account;

fn main() {
  let args: Vec<String> = env::args().collect();
  match args.len() {
    1 => {
      print_usage();
    },
    _ => {
      match process_file(&args[1]) {
        Err(err) => {
          println!("Error occured during processing input file: {}", err);
          std::process::exit(1);
        },
        _ => {}
      }
    }
  }
}

fn print_usage() {
  println!("
USAGE:
  executable <transactions_filepath>
  ");
}

fn csv_reader(path: &str) -> Result<Reader<File>, Error> {
  let mut reader = ReaderBuilder::new();
  reader.has_headers(true).trim(Trim::All).from_path(path)
}

fn print_result(result: HashMap<u16, Account>) -> Result<(), Error> {
  let mut writer = csv::Writer::from_writer(io::stdout());
  for (_, account) in result {
    writer.serialize(account)?;
  }
  writer.flush()?;
  Ok(())
}

fn process_file(path: &str) -> Result<(), Error> {
  let mut reader = csv_reader(path)?;
  let accounts = processor::process(&mut reader)?;
  print_result(accounts)?;
  Ok(())
}

