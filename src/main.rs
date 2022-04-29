#![feature(let_chains)]
#![feature(box_patterns)]
#[macro_use]
extern crate pest_derive;

// mod ast_viz;
mod codegen;
mod error;
mod parser;

use crate::codegen::CodeBuilder;
use clap::Parser;
use inkwell::context::Context;
use parser::AST;
use std::{fs::File, io::Read, path::Path};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// file name to compile
    #[clap(short, long)]
    file: String,
    /// visualize ast as dot file
    #[clap(short, long)]
    dotfile: Option<String>,
}

fn main() {
    let args: Args = Args::parse();
    let mut source_file = File::open(&args.file).expect("Unable to open source file!");
    let mut source_code = String::new();

    // Read source file
    source_file
        .read_to_string(&mut source_code)
        .expect("Unable to read the file!");

    let prefix = args.file.strip_suffix(".c").unwrap_or(args.file.as_str());
    let context = Context::create();

    match AST::parse(source_code) {
        Ok(ast) => match CodeBuilder::new(&context, args.file.as_str(), &ast) {
            Ok(codebuilder) => {
                codebuilder.build_llvmir(Path::new(&format!("{}.ll", prefix)));
                codebuilder.build_asm(Path::new(&format!("{}.s", prefix)));
            }
            Err(e) => eprintln!("Error: {}", e),
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    // Generate dot file
    // if let Some(dotfile) = args.dotfile {
    //     let dot_cont = DiGraph::new(&args.file, &ast).write_dot();
    //     let mut file = File::create(&dotfile).expect("Unable to create a dot file!");
    //     file.write_all(dot_cont.as_bytes())
    //         .expect("Unable to write dot file!");
    // }

    // A Context is a container for all LLVM entities including Modules.

    // Now we build asm file, llvm-ir file and print json AST.
    // After we will make it chosable.
}
