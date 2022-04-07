#![feature(let_chains)]
extern crate clap;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate serde;

mod codegen;
mod parser;

use clap::Parser;
use inkwell::context::Context;
use std::{fs::File, io::Read, path::Path};

use crate::{codegen::CodeBuilder, parser::AST};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// file name to compile
    #[clap(short, long)]
    file: String,
}

fn main() {
    let args: Args = Args::parse();
    let mut source_file = File::open(&args.file).expect("Unable to open source file!");
    let mut source_code = String::new();
    source_file
        .read_to_string(&mut source_code)
        .expect("Unable to read the file!");
    let prefix = args.file.strip_suffix(".c").unwrap_or(args.file.as_str());

    let ast = AST::parse(source_code);
    println!("{:?}", &ast);
    println!("{:?}", serde_json::to_string(&ast));
    // A Context is a container for all LLVM entities including Modules.
    let context = Context::create();
    let codegen = CodeBuilder::new(&context, args.file.as_str(), &ast).unwrap();
    codegen.build_asm(Path::new(&format!("{}.s", prefix)));
    codegen.build_llvmir(Path::new(&format!("{}.ll", prefix)));
}
