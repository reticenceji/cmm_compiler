//! Abstract Syntax Tree Visualizer
use crate::parser::AST;
use lazy_static::lazy_static;

#[derive(Debug)]
struct DiGraph {
    name: String,
    conts: Vec<Content>,
}

#[derive(Debug)]
enum Node {
    Symbol(String),
    Subgraph(),
}

#[derive(Debug)]
struct Edge {
    from: Node,
    to: Node,
}

#[derive(Debug)]
enum Content {
    Node(Node),
    Edge(Edge),
}

impl DiGraph {
    pub fn new(name: String) -> Self {
        Self {
            name,
            conts: Vec::new(),
        }
    }

    pub fn parse_ast(ast: &AST) {
        match ast {
            AST::FunctionDec(ftype, name, params, box ast) => {}
            AST::VariableDec(vtype, name) => {}
            AST::BlockStmt(ast1, ast2) => {}
            AST::SelectionStmt(box ast1, box ast2, ast3) => {}
            AST::IterationStmt(box ast1, box ast2) => {}
            AST::ReturnStmt(ast) => {}
            AST::AssignmentExpr(box ast1, box ast2) => {}
            AST::BinaryExpr(oprand, box ast1, box ast2) => {}
            AST::CallExpr(name, ast) => {}
            AST::Variable(name, ast) => {}
            AST::IntLiteral(val) => {}
        }
    }
}

struct IDAllocator {
    current_id: usize,
}

impl IDAllocator {
    pub fn alloc(&mut self) -> usize {
        self.current_id += 1;
        self.current_id - 1
    }
}

lazy_static! {
    static ref IDALLOCATOR: IDAllocator = IDAllocator { current_id: 1 };
}

#[test]
fn test_ast() {
    use crate::{codegen::CodeBuilder, parser::AST};
    use inkwell::context::Context;
    use std::{fs::File, io::Read, path::Path};
    let file = "test/test_ast.c";
    let mut source_file = File::open(file).expect("Unable to open source file!");
    let mut source_code = String::new();

    // Read source file
    source_file
        .read_to_string(&mut source_code)
        .expect("Unable to read the file!");

    let prefix = file.strip_suffix(".c").unwrap_or(file);
    let ast = AST::parse(source_code);

    println!("{:#?}", ast);

    // A Context is a container for all LLVM entities including Modules.
    let context = Context::create();
    let codegen = CodeBuilder::new(&context, file, &ast).unwrap();

    // Now we build asm file, llvm-ir file and print json AST.
    // After we will make it chosable.
    codegen.build_asm(Path::new(&format!("{}.s", prefix)));
    codegen.build_llvmir(Path::new(&format!("{}.ll", prefix)));

    println!("{:#?}", serde_json::to_string(&ast));
}
