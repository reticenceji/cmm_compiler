#![feature(let_chains)]
#![feature(box_patterns)]
#![feature(path_try_exists)]
#![feature(is_some_with)]

use clap::Parser;
use std::fs;
use std::io::Write;
use std::process;
use std::{fs::File, path::Path};

use cmm::{Ast, CodeBuilder, Context, DiGraph};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File name of generated file
    file: String,
    /// Visualize ast as dot file
    #[clap(short, long)]
    dotfile: Option<String>,
    /// Output file name
    #[clap(short = 'o', long)]
    output: Option<String>,
    /// Optimize the code
    #[clap(short = 'O', long)]
    opt: bool,
    /// Generate assembly file
    #[clap(short = 's', long)]
    asm: bool,
    /// Generate llvm-ir
    #[clap(long)]
    llvmir: bool,
}

fn main() {
    let args: Args = Args::parse();
    let source_code = fs::read_to_string(&args.file).expect("Unable to open source file!");
    let filename = match args.output {
        Some(name) => name,
        None => {
            let prefix = args.file.strip_suffix(".c").unwrap_or(args.file.as_str());
            match (args.asm, args.llvmir) {
                (true, _) => format!("{}.s", prefix),
                (false, true) => format!("{}.ll", prefix),
                (false, false) => String::from("a.out"),
            }
        }
    };

    let context = Context::create();
    match Ast::parse(source_code) {
        Ok(ast) => match CodeBuilder::new(&context, args.file.as_str(), &ast, args.opt) {
            Ok(codebuilder) => {
                match (args.asm, args.llvmir) {
                    (true, _) => codebuilder.build_asm(Path::new(&filename)),
                    (false, true) => codebuilder.build_llvmir(Path::new(&filename)),
                    (false, false) => {
                        let tmpfile = format!("{}.s", filename);
                        let io_c = if fs::try_exists("/usr/lib/cmm/io.c").is_ok_and(|b| *b) {
                            "/usr/lib/cmm/io.c"
                        } else if fs::try_exists("./io.c").is_ok_and(|b| *b) {
                            "./io.c"
                        } else {
                            eprintln!("Cannot find io.c in /usr/lib/cmm or current directory");
                            process::exit(1);
                        };
                        codebuilder.build_asm(Path::new(&tmpfile));
                        process::Command::new("clang")
                            .args([tmpfile.as_str(), io_c, "-o", filename.as_str()])
                            .spawn()
                            .expect("Fail to start clang")
                            .wait()
                            .expect("Fail to link io.c with clang");
                        fs::remove_file(Path::new(&tmpfile)).expect("Fail to remove temp file");
                    }
                };

                // Generate dot file
                if let Some(dotfile) = args.dotfile {
                    let dot_cont = DiGraph::new(&args.file, &ast).write_dot();
                    let mut file = File::create(&dotfile).expect("Unable to create a dot file!");
                    file.write_all(dot_cont.as_bytes())
                        .expect("Unable to write dot file!");
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
