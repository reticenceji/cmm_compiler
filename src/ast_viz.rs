//! Abstract Syntax Tree Visualizer
use std::sync::Mutex;

use crate::parser::AST;
use lazy_static::lazy_static;

#[derive(Debug)]
struct DiGraph {
    name: Option<String>,
    id: usize,
    conts: Vec<Content>,
}

#[derive(Debug)]
enum Node {
    Symbol(usize, String),
    Subgraph(DiGraph),
}

impl Node {
    pub fn new_symbol(name: &str) -> Self {
        Self::Symbol(ID_ALLOCATOR.lock().unwrap().alloc(), name.into())
    }

    pub fn new_subg(subg: DiGraph) -> Self {
        Self::Subgraph(subg)
    }

    pub fn get_id(&self) -> usize {
        match self {
            Self::Symbol(id, _) => id.to_owned(),
            Self::Subgraph(g) => g.id,
        }
    }

    pub fn to_dot(&self) -> String {
        match self {
            Self::Symbol(id, name) => {
                format!("node{} [ label = \" {} \" ];", id, name)
            }
            Self::Subgraph(subg) => subg.to_dot(),
        }
    }
}

#[derive(Debug)]
struct Edge {
    from: usize,
    to: usize,
}

impl Edge {
    // pub fn new(node1: &Node, node2: &Node) -> Self {
    //     let id1 = node1.get_id();
    //     let id2 = node2.get_id();

    //     Self {
    //         from: id1.to_owned(),
    //         to: id2.to_owned(),
    //     }
    // }

    pub fn new(g: &DiGraph, node: &Node) -> Self {
        let id1 = g.get_id();
        let id2 = node.get_id();

        Self {
            from: id1.to_owned(),
            to: id2.to_owned(),
        }
    }

    pub fn to_dot(&self) -> String {
        format!("node{} -> node{}", self.from, self.to)
    }
}

#[derive(Debug)]
enum Content {
    Node(Node),
    Edge(Edge),
}

impl Content {
    pub fn to_dot(&self) -> String {
        match self {
            Self::Node(node) => node.to_dot(),
            Self::Edge(edge) => edge.to_dot(),
        }
    }
}

impl DiGraph {
    pub fn new(name: &str, asts: &Vec<AST>) -> Self {
        let mut g = Self {
            name: Some(name.to_string()),
            id: ID_ALLOCATOR.lock().unwrap().alloc(),
            conts: Vec::new(),
        };

        g.parse_asts(asts);
        g
    }

    fn empty() -> Self {
        Self {
            name: None,
            id: ID_ALLOCATOR.lock().unwrap().alloc(),
            conts: Vec::new(),
        }
    }

    pub fn write_dot(&self) -> String {
        let buf = self.to_dot();
        format!("digraph prog {{\n{}\n}}", buf)
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    fn parse_asts(&mut self, asts: &Vec<AST>) {
        for ast in asts {
            let subg = DiGraph::from_ast(ast);
            let node = Node::Subgraph(subg);

            self.add_cont(Content::Edge(Edge::new(&self, &node)));
            self.add_cont(Content::Node(node));
        }
    }

    fn from_ast(ast: &AST) -> Self {
        let mut g = Self::empty();
        g.parse_ast(ast);
        g
    }

    pub fn add_cont(&mut self, cont: Content) {
        self.conts.push(cont)
    }

    fn parse_ast(&mut self, ast: &AST) {
        self.name = Some("xxx".to_string());
        match ast {
            AST::FunctionDec(ftype, name, params, box ast) => {
                self.name = Some("FunctionDec".to_string());
                let node_type = Node::new_symbol(&ftype.to_string());
                let node_name = Node::new_symbol(name);

                self.add_cont(Content::Edge(Edge::new(&self, &node_type)));
                self.add_cont(Content::Edge(Edge::new(&self, &node_name)));
                self.add_cont(Content::Node(node_type));
                self.add_cont(Content::Node(node_name));

                if !params.is_empty() {
                    let mut subg = DiGraph::empty();
                    subg.name = Some("Params".to_string());

                    for (ptype, name) in params {
                        let node_type = Node::new_symbol(&ptype.to_string());
                        let node_name = Node::new_symbol(name);

                        subg.add_cont(Content::Edge(Edge::new(&subg, &node_type)));
                        subg.add_cont(Content::Edge(Edge::new(&subg, &node_name)));
                        subg.add_cont(Content::Node(node_type));
                        subg.add_cont(Content::Node(node_name));
                    }

                    let node = Node::new_subg(subg);
                    self.add_cont(Content::Edge(Edge::new(&self, &node)));
                    self.add_cont(Content::Node(node));
                }

                let subg = DiGraph::from_ast(ast);
                let node = Node::new_subg(subg); 
                self.add_cont(Content::Edge(Edge::new(&self, &node)));
                self.add_cont(Content::Node(node));
            }
            AST::VariableDec(vtype, name) => {
                self.name = Some("VariableDec".to_string());
                let node_type = Node::new_symbol(&vtype.to_string());
                let node_name = Node::new_symbol(name);

                self.add_cont(Content::Edge(Edge::new(&self, &node_type)));
                self.add_cont(Content::Edge(Edge::new(&self, &node_name)));
                self.add_cont(Content::Node(node_type));
                self.add_cont(Content::Node(node_name));
            }
            AST::BlockStmt(ast1, ast2) => {
                self.name = Some("BlockStmt".to_string());
                for ast in ast1 {
                    let subg = DiGraph::from_ast(ast);
                    let node = Node::Subgraph(subg);

                    self.add_cont(Content::Edge(Edge::new(&self, &node)));
                    self.add_cont(Content::Node(node));
                }

                for ast in ast2 {
                    let subg = DiGraph::from_ast(ast);
                    let node = Node::Subgraph(subg);

                    self.add_cont(Content::Edge(Edge::new(&self, &node)));
                    self.add_cont(Content::Node(node));
                }
            }
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

    fn to_dot(&self) -> String {
        let mut buf = format!(
            "node{} [ label = \" {} \" ];",
            self.id,
            self.name.as_ref().expect("Unformed DiGraph!")
        );
        for cont in &self.conts {
            buf.push_str(&cont.to_dot());
            buf.push_str("\n");
        }

        buf
    }
}

/// IDAllocator for Nodes
struct IDAllocator {
    id: usize,
}

impl IDAllocator {
    pub fn alloc(&mut self) -> usize {
        self.id += 1;
        // Return values
        self.id - 1
    }
}

lazy_static! {
    /// Instance of IDAllocator
    static ref ID_ALLOCATOR: Mutex<IDAllocator> = Mutex::new(IDAllocator { id: 1 });
}

#[test]
fn test_ast() {
    use crate::parser::AST;
    use std::{fs::File, io::Read};
    let file = "test/test_ast.c";
    let mut source_file = File::open(file).expect("Unable to open source file!");
    let mut source_code = String::new();

    // Read source file
    source_file
        .read_to_string(&mut source_code)
        .expect("Unable to read the file!");

    let ast = AST::parse(source_code);

    // println!("{:#?}", ast);

    let g = DiGraph::new("test", &ast);
    println!("{}", g.write_dot())
}
