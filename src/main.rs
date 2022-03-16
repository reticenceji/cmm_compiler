extern crate pest;
#[macro_use]
extern crate pest_derive;
mod parser;
use pest::Parser;

fn main() {
    let parse = parser::CParser::parse(parser::Rule::program, "1112");
    println!("{:?}", parse);
}
