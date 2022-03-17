use pest::{iterators::Pair, Parser};
#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CParser;
use sugars::boxed;

#[derive(Debug)]
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
    IntLiteral(i64),
}
#[derive(Debug)]
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
#[derive(Debug)]
pub enum Type {
    Int,
    Void,
    IntArray(usize),
    IntPtr,
}

/// Turn the source code to AST
pub fn parse(source_code: String) -> Vec<AST> {
    let root = CParser::parse(Rule::program, &source_code)
        .unwrap()
        .next()
        .unwrap();

    let mut ast = vec![];
    visit_program(root, &mut ast);
    return ast;
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

fn visit_int_literal(pair: Pair<'_, Rule>) -> i64 {
    match pair.as_rule() {
        Rule::bin_literal => i64::from_str_radix(pair.as_str(), 2).unwrap(),
        Rule::oct_literal => i64::from_str_radix(pair.as_str(), 8).unwrap(),
        Rule::dec_literal => i64::from_str_radix(pair.as_str(), 10).unwrap(),
        Rule::hex_literal => i64::from_str_radix(pair.as_str(), 16).unwrap(),
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

fn visit_expression(pair: Pair<'_, Rule>) -> AST {
    let child = pair.into_inner().next().unwrap();
    match child.as_rule() {
        Rule::assignment_expr => visit_assignment_expr(child),
        Rule::unary_expr => visit_unary_expr(child),
        _ => visit_binary_expr(child),
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

#[cfg(test)]
mod test_parse {
    use pest::iterators::Pair;
    use pest::Parser;
    use std::fs::File;
    use std::io::Read;

    fn dfs(tab: usize, pair: Pair<'_, super::Rule>) {
        for _ in 0..tab {
            print!("  ");
        }
        println!("{:?}", pair.as_rule());
        for i in pair.into_inner() {
            dfs(tab + 1, i);
        }
    }
    #[test]
    fn parse_tree_test() {
        let mut f = File::open("test/test.c").unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        let pairs = super::CParser::parse(super::Rule::program, &buf)
            .unwrap()
            .next()
            .unwrap();
        // println!("{:?}", pairs.as_span());

        dfs(0, pairs);
    }
    #[test]
    fn ast_test() {
        let mut f = File::open("test/test.c").unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        let ast = super::parse(buf);
        for i in ast {
            println!("{:?}", i);
        }
    }
}
