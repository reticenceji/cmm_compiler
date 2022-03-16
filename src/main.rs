extern crate clap;
extern crate pest;
#[macro_use]
extern crate pest_derive;

mod parser;

use clap::Parser;
use parser::parse;
use std::{fs::File, io::Read};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// file name to compile
    #[clap(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();
    let mut source_file = File::open(&args.file).expect("Unable to open source file!");
    let mut source_code = String::new();
    source_file
        .read_to_string(&mut source_code)
        .expect("Unable to read the file!");
    parse(source_code);
    todo!("ast, check, generate code...");
}
