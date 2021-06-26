use std::env;

mod transactions_reader;
mod enums;
mod structs;

fn print_usage() {
  println!("
USAGE:
  executable <transactions_filepath>
  ");
}

fn process_file(path: &str) {
  println!("Reading {}", path);
  let result = transactions_reader::read(path);
  match result {
    Ok(result) => {
      println!("Result: {:?}", result)
    },
    Err(err) => {
      println!("Error occured during processing: {}", err)
    }
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  match args.len() {
    1 => {
      print_usage();
    },
    _ => {
      process_file(&args[1]);
    }
  }
}
