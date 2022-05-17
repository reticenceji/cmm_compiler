#![feature(let_chains)]
#![feature(box_patterns)]
#![feature(path_try_exists)]
#![feature(is_some_with)]
#[macro_use]
extern crate pest_derive;

mod ast_viz;
mod codegen;
mod error;
mod parser;

pub use ast_viz::DiGraph;
pub use codegen::CodeBuilder;
pub use inkwell::context::Context;
pub use parser::Ast;
