use inkwell::{
    context::Context,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
};
use pest::{iterators::Pair, Parser};
use serde::Serialize;
use std::borrow::Borrow;
use sugars::boxed;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CParser;

#[derive(Debug, Serialize)]
pub enum AST {
    /// type, name, params, block_statements: type name(params) {statements}
    FunctionDec(Type, String, Vec<(Type, String)>, Box<AST>),
    /// type, name
    VariableDec(Type, String),

    /// varibale_declarations, expressions
    BlockStmt(Vec<AST>, Vec<AST>),
    /// condition, if_statement, else_statement: if (condition) {if_statements} else {else_statement}
    SelectionStmt(Box<AST>, Box<AST>, Option<Box<AST>>),
    /// condition, expressions: while(condition) {expression}
    IterationStmt(Box<AST>, Box<AST>),
    /// return value
    ReturnStmt(Option<Box<AST>>),

    /// var, expression
    AssignmentExpr(Box<AST>, Box<AST>),
    /// operation, expression, expression: expression operation expression
    BinaryExpr(Oprand, Box<AST>, Box<AST>),
    /// name, args
    CallExpr(String, Vec<AST>),

    /// name, []: name[]
    Variable(String, Option<Box<AST>>),
    IntLiteral(i32),
}

#[derive(Debug, Serialize)]
pub enum Oprand {
    Add,
    Sub,
    Mul,
    Div,
    Ge,
    Le,
    Gt,
    Lt,
    Eq,
    Ne,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Int,
    Void,
    IntArray(usize),
    IntPtr,
}

impl<'ctx> Type {
    pub fn to_llvm_basic_type(&self, context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            Type::Int => context.i32_type().as_basic_type_enum(),
            Type::Void => panic!("Variable have void type"),
            Type::IntArray(size) => context
                .i32_type()
                .array_type(*size as u32)
                .as_basic_type_enum(),
            Type::IntPtr => context
                .i32_type()
                .ptr_type(inkwell::AddressSpace::Generic)
                .as_basic_type_enum(),
        }
    }

    pub fn to_llvm_basic_metadata_type(
        &self,
        context: &'ctx Context,
    ) -> BasicMetadataTypeEnum<'ctx> {
        match self {
            Type::Int => BasicMetadataTypeEnum::IntType(context.i32_type()),
            Type::Void => panic!("Variable have void type"),
            Type::IntArray(size) => {
                BasicMetadataTypeEnum::ArrayType(context.i32_type().array_type(*size as u32))
            }
            Type::IntPtr => BasicMetadataTypeEnum::PointerType(
                context.i32_type().ptr_type(inkwell::AddressSpace::Generic),
            ),
        }
    }
}

impl AST {
    /// Turn the source code to AST,
    /// which can be serialized to json.
    pub fn parse<T>(source_code: T) -> Vec<Self>
    where
        T: Borrow<str>,
    {
        let root = CParser::parse(Rule::program, source_code.borrow())
            .unwrap()
            .next()
            .unwrap();

        let mut ast = vec![];
        visit_program(root, &mut ast);
        return ast;
    }
}

fn visit_program(pair: Pair<'_, Rule>, ast: &mut Vec<AST>) {
    assert_eq!(pair.as_rule(), Rule::program);
    for node in pair.into_inner() {
        match node.as_rule() {
            Rule::func_declaration => {
                visit_func_declaration(node, ast);
            }
            Rule::var_declaration => visit_var_declaration(node, ast),
            _ => {}
        }
    }
}

fn visit_func_declaration(pair: Pair<'_, Rule>, ast: &mut Vec<AST>) {
    let mut children = pair.into_inner();
    let type_spec = visit_type_spec(children.next().unwrap());
    let id = visit_id(children.next().unwrap());
    let params = visit_params(children.next().unwrap());
    let block_stmt = visit_block_stmt(children.next().unwrap());

    ast.push(AST::FunctionDec(type_spec, id, params, boxed!(block_stmt)));
}

fn visit_var_declaration(pair: Pair<'_, Rule>, ast: &mut Vec<AST>) {
    let mut children = pair.into_inner();
    let mut type_spec = visit_type_spec(children.next().unwrap());
    let id = visit_id(children.next().unwrap());

    for child in children {
        match child.as_rule() {
            Rule::int_literal => {
                let size = visit_int_literal(child) as usize;
                type_spec = Type::IntArray(size);
            }
            _ => {}
        }
    }
    ast.push(AST::VariableDec(type_spec, id));
}

fn visit_int_literal(pair: Pair<'_, Rule>) -> i32 {
    let child = pair.into_inner().next().unwrap();
    match child.as_rule() {
        Rule::bin_literal => i32::from_str_radix(child.as_str(), 2).unwrap(),
        Rule::oct_literal => i32::from_str_radix(child.as_str(), 8).unwrap(),
        Rule::dec_literal => i32::from_str_radix(child.as_str(), 10).unwrap(),
        Rule::hex_literal => i32::from_str_radix(child.as_str(), 16).unwrap(),
        _ => 0,
    }
}
fn visit_type_spec(pair: Pair<'_, Rule>) -> Type {
    let child = pair.into_inner().next().unwrap();
    match child.as_rule() {
        Rule::int => Type::Int,
        Rule::void => Type::Void,
        _ => panic!(),
    }
}

fn visit_id(pair: Pair<'_, Rule>) -> String {
    pair.as_str().to_string()
}

fn visit_params(pair: Pair<'_, Rule>) -> Vec<(Type, String)> {
    let mut params = vec![];
    for node in pair.into_inner() {
        if Rule::param == node.as_rule() {
            params.push(visit_param(node));
        }
    }
    params
}

fn visit_param(pair: Pair<'_, Rule>) -> (Type, String) {
    let mut children = pair.into_inner();
    let mut type_spec = visit_type_spec(children.next().unwrap());
    let id = visit_id(children.next().unwrap());
    if let Some(x) = children.next() && x.as_rule() == Rule::pointer {
        type_spec = Type::IntPtr;
    }
    (type_spec, id)
}
fn visit_block_stmt(pair: Pair<'_, Rule>) -> AST {
    let children = pair.into_inner();
    let mut vars = vec![];
    let mut statements = vec![];
    for node in children {
        match node.as_rule() {
            Rule::var_declaration => visit_var_declaration(node, &mut vars),
            Rule::statement => visit_statement(node, &mut statements),
            _ => {}
        }
    }
    AST::BlockStmt(vars, statements)
}

fn visit_statement(pair: Pair<'_, Rule>, ast: &mut Vec<AST>) {
    let children = pair.into_inner().next().unwrap();
    match children.as_rule() {
        Rule::block_stmt => {
            ast.push(visit_block_stmt(children));
        }
        Rule::expression_stmt => {
            let children = children.into_inner();
            for node in children {
                match node.as_rule() {
                    Rule::expression => {
                        ast.push(visit_expression(node));
                    }
                    _ => {}
                }
            }
        }
        Rule::selection_stmt => {
            let children = children.into_inner();
            let mut is_if = true;
            let mut condition: Option<Box<AST>> = None;
            let mut if_statement: Vec<AST> = vec![];
            let mut else_statement: Vec<AST> = vec![];

            for node in children {
                match node.as_rule() {
                    Rule::expression => {
                        condition = Some(boxed!(visit_expression(node)));
                    }
                    Rule::statement if is_if => {
                        visit_statement(node, &mut if_statement);
                        is_if = false;
                    }
                    Rule::statement if !is_if => {
                        visit_statement(node, &mut else_statement);
                    }
                    _ => {}
                }
            }
            let statement = AST::SelectionStmt(
                condition.unwrap(),
                boxed!(if_statement.into_iter().next().unwrap()),
                else_statement.into_iter().next().map(|x| boxed!(x)),
            );

            ast.push(statement);
        }
        Rule::iteration_stmt => {
            let children = children.into_inner();
            let mut condition: Option<Box<AST>> = None;
            let mut loop_statement: Vec<AST> = vec![];

            for node in children {
                match node.as_rule() {
                    Rule::expression => condition = Some(boxed!(visit_expression(node))),
                    Rule::statement => {
                        visit_statement(node, &mut loop_statement);
                    }
                    _ => {}
                }
            }

            let statement = AST::IterationStmt(
                condition.unwrap(),
                boxed!(loop_statement.into_iter().next().unwrap()),
            );
            ast.push(statement);
        }
        Rule::return_stmt => {
            let children = children.into_inner();
            let mut expression: Option<Box<AST>> = None;
            for node in children {
                match node.as_rule() {
                    Rule::expression => expression = Some(boxed!(visit_expression(node))),
                    _ => {}
                }
            }

            let statement = AST::ReturnStmt(expression);
            ast.push(statement);
        }
        _ => {
            println!("{:?}", children.as_rule())
        }
    }
}

fn visit_expression(mut pair: Pair<'_, Rule>) -> AST {
    if pair.as_rule() == Rule::expression {
        pair = pair.into_inner().next().unwrap();
    }
    match pair.as_rule() {
        Rule::assignment_expr => visit_assignment_expr(pair),
        Rule::unary_expr => visit_unary_expr(pair),
        _ => visit_binary_expr(pair),
    }
}

fn visit_unary_expr(pair: Pair<'_, Rule>) -> AST {
    let child = pair.into_inner().next().unwrap();
    match child.as_rule() {
        Rule::var => visit_var(child),
        Rule::int_literal => AST::IntLiteral(visit_int_literal(child)),
        Rule::call_expr => visit_call_expr(child),
        Rule::bracket_expr => visit_bracket_expr(child),
        _ => panic!(),
    }
}

fn visit_bracket_expr(pair: Pair<'_, Rule>) -> AST {
    let mut children = pair.into_inner();
    loop {
        let child = children.next().unwrap();
        if child.as_rule() == Rule::expression {
            return visit_expression(child);
        }
    }
}

fn visit_call_expr(pair: Pair<'_, Rule>) -> AST {
    let mut children = pair.into_inner();
    let id = visit_id(children.next().unwrap());
    let mut args = vec![];
    visit_args(children.next().unwrap(), &mut args);
    AST::CallExpr(id, args)
}

fn visit_args(pair: Pair<'_, Rule>, args: &mut Vec<AST>) {
    let children = pair.into_inner();
    for node in children {
        if node.as_rule() == Rule::expression {
            args.push(visit_expression(node));
        }
    }
}

fn visit_assignment_expr(pair: Pair<'_, Rule>) -> AST {
    let mut children = pair.into_inner();
    let var = visit_var(children.next().unwrap());
    children.next();
    let expression = visit_expression(children.next().unwrap());
    AST::AssignmentExpr(boxed!(var), boxed!(expression))
}

fn visit_var(pair: Pair<'_, Rule>) -> AST {
    let mut children = pair.into_inner();
    let id = children.next().unwrap().as_str().to_string();
    let mut expression = None;
    for node in children {
        if node.as_rule() == Rule::expression {
            expression = Some(boxed!(visit_expression(node)));
        }
    }
    AST::Variable(id, expression)
}

fn visit_binary_expr(pair: Pair<'_, Rule>) -> AST {
    let mut children = pair.into_inner();
    let mut lhs = visit_expression(children.next().unwrap());

    while let Some(mut expr) = children.next() {
        let op = match expr.as_rule() {
            Rule::op_ge => Oprand::Ge,
            Rule::op_le => Oprand::Le,
            Rule::op_gt => Oprand::Gt,
            Rule::op_lt => Oprand::Lt,
            Rule::op_eq => Oprand::Eq,
            Rule::op_ne => Oprand::Ne,
            Rule::op_add => Oprand::Add,
            Rule::op_sub => Oprand::Sub,
            Rule::op_mul => Oprand::Mul,
            Rule::op_div => Oprand::Div,
            _ => panic!(),
        };
        expr = children.next().unwrap();
        let rhs = visit_expression(expr);
        lhs = AST::BinaryExpr(op, boxed!(lhs), boxed!(rhs));
    }
    lhs
}

fn dfs(tabs: &mut Vec<bool>, pair: Pair<'_, Rule>) {
    let mut pair = pair.into_inner();
    let mut current = pair.next();
    let mut next = pair.next();
    while let Some(i) = current {
        for tab in tabs.iter() {
            if *tab {
                print!("│   ");
            } else {
                print!("    ");
            }
        }
        if next.is_some() {
            print!("├── ");
            tabs.push(true);
        } else {
            print!("└── ");
            tabs.push(false);
        }
        println!("{:?}", i.as_rule());

        dfs(tabs, i);
        tabs.pop();

        current = next;
        next = pair.next();
    }
}

/// print the parse tree, like command tree's style
pub fn parse_tree_visable(parse_tree: Pair<'_, Rule>) {
    dfs(&mut vec![], parse_tree);
}

#[cfg(test)]
mod test_parse {
    // use pest::iterators::Pair;
    use pest::Parser;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn parse_tree_test() {
        let mut f = File::open("test/test.c").unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        let root = super::CParser::parse(super::Rule::program, &buf)
            .unwrap()
            .next()
            .unwrap();
        assert_eq!(root.as_rule(), super::Rule::program);
        super::parse_tree_visable(root);
    }
    #[test]
    fn ast_test() {
        let mut f = File::open("test/test.c").unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        let ast = super::AST::parse(buf);
        for i in &ast {
            println!("{:?}", i);
        }
        println!("{}", serde_json::to_string_pretty(&ast).unwrap());
    }
}
