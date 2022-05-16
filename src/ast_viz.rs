//! Abstract Syntax Tree Visualizer
use crate::parser::{ASTInfo, Ast};
use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Debug)]
pub struct DiGraph {
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
    pub fn new(name: &str, asts: &Vec<Ast>) -> Self {
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

    fn parse_asts(&mut self, asts: &Vec<Ast>) {
        for ast in asts {
            let subg = DiGraph::from_ast(ast);
            let node = Node::Subgraph(subg);

            self.add_cont(Content::Edge(Edge::new(self, &node)));
            self.add_cont(Content::Node(node));
        }
    }

    fn from_ast(ast: &Ast) -> Self {
        let mut g = Self::empty();
        g.parse_ast(ast);
        g
    }

    fn add_cont(&mut self, cont: Content) {
        self.conts.push(cont)
    }

    fn parse_ast(&mut self, ast: &Ast) {
        match &ast.info {
            ASTInfo::FunctionDec(ftype, name, params, box ast) => {
                self.name = Some("FunctionDec".to_string());
                let node_type = Node::new_symbol(&ftype.to_string());
                let node_name = Node::new_symbol(name);

                self.add_cont(Content::Edge(Edge::new(self, &node_type)));
                self.add_cont(Content::Edge(Edge::new(self, &node_name)));
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
                    self.add_cont(Content::Edge(Edge::new(self, &node)));
                    self.add_cont(Content::Node(node));
                }

                let subg = DiGraph::from_ast(ast);
                let node = Node::new_subg(subg);
                self.add_cont(Content::Edge(Edge::new(self, &node)));
                self.add_cont(Content::Node(node));
            }
            ASTInfo::VariableDec(vtype, name) => {
                self.name = Some("VariableDec".to_string());
                let node_type = Node::new_symbol(&vtype.to_string());
                let node_name = Node::new_symbol(name);

                self.add_cont(Content::Edge(Edge::new(self, &node_type)));
                self.add_cont(Content::Edge(Edge::new(self, &node_name)));
                self.add_cont(Content::Node(node_type));
                self.add_cont(Content::Node(node_name));
            }
            ASTInfo::BlockStmt(ast1, ast2) => {
                self.name = Some("BlockStmt".to_string());
                for ast in ast1 {
                    let subg = DiGraph::from_ast(ast);
                    let node = Node::Subgraph(subg);

                    self.add_cont(Content::Edge(Edge::new(self, &node)));
                    self.add_cont(Content::Node(node));
                }

                for ast in ast2 {
                    let subg = DiGraph::from_ast(ast);
                    let node = Node::Subgraph(subg);

                    self.add_cont(Content::Edge(Edge::new(self, &node)));
                    self.add_cont(Content::Node(node));
                }
            }
            ASTInfo::SelectionStmt(box ast1, box ast2, ast3) => {
                self.name = Some("SelectionStmt".to_string());

                // If {cond} {expr}
                let if_node = Node::new_symbol("if");
                let cond_node = Node::Subgraph(DiGraph::from_ast(ast1));
                let true_node = Node::Subgraph(DiGraph::from_ast(ast2));

                self.add_cont(Content::Edge(Edge::new(self, &if_node)));
                self.add_cont(Content::Edge(Edge::new(self, &cond_node)));
                self.add_cont(Content::Edge(Edge::new(self, &true_node)));
                self.add_cont(Content::Node(if_node));
                self.add_cont(Content::Node(cond_node));
                self.add_cont(Content::Node(true_node));

                // Else {expr}
                if let Some(box ast) = ast3 {
                    let else_node = Node::new_symbol("else");
                    let false_node = Node::Subgraph(DiGraph::from_ast(ast));

                    self.add_cont(Content::Edge(Edge::new(self, &else_node)));
                    self.add_cont(Content::Edge(Edge::new(self, &false_node)));
                    self.add_cont(Content::Node(else_node));
                    self.add_cont(Content::Node(false_node));
                }
            }
            ASTInfo::IterationStmt(box ast1, box ast2) => {
                self.name = Some("IterationStmt".to_string());

                let while_node = Node::new_symbol("while");
                let cond_node = Node::new_subg(DiGraph::from_ast(ast1));
                let expr_node = Node::new_subg(DiGraph::from_ast(ast2));

                self.add_cont(Content::Edge(Edge::new(self, &while_node)));
                self.add_cont(Content::Edge(Edge::new(self, &cond_node)));
                self.add_cont(Content::Edge(Edge::new(self, &expr_node)));
                self.add_cont(Content::Node(while_node));
                self.add_cont(Content::Node(cond_node));
                self.add_cont(Content::Node(expr_node));
            }
            ASTInfo::ReturnStmt(ast) => {
                self.name = Some("ReturnStmt".to_string());

                let return_node = Node::new_symbol("return");

                self.add_cont(Content::Edge(Edge::new(self, &return_node)));
                self.add_cont(Content::Node(return_node));

                if let Some(box ast) = ast {
                    let retval_node = Node::new_subg(DiGraph::from_ast(ast));

                    self.add_cont(Content::Edge(Edge::new(self, &retval_node)));
                    self.add_cont(Content::Node(retval_node));
                }
            }
            ASTInfo::AssignmentExpr(box ast1, box ast2) => {
                self.name = Some("AssignmentExpr".to_string());

                let var_node = Node::new_subg(DiGraph::from_ast(ast1));
                let equal_node = Node::new_symbol("=");
                let expr_node = Node::new_subg(DiGraph::from_ast(ast2));

                self.add_cont(Content::Edge(Edge::new(self, &var_node)));
                self.add_cont(Content::Edge(Edge::new(self, &equal_node)));
                self.add_cont(Content::Edge(Edge::new(self, &expr_node)));
                self.add_cont(Content::Node(var_node));
                self.add_cont(Content::Node(equal_node));
                self.add_cont(Content::Node(expr_node));
            }
            ASTInfo::BinaryExpr(oprand, box ast1, box ast2) => {
                self.name = Some("BinaryExpr".to_string());
                let op_node = Node::new_symbol(&oprand.to_string());
                let lval = Node::new_subg(DiGraph::from_ast(ast1));
                let rval = Node::new_subg(DiGraph::from_ast(ast2));

                self.add_cont(Content::Edge(Edge::new(self, &op_node)));
                self.add_cont(Content::Edge(Edge::new(self, &lval)));
                self.add_cont(Content::Edge(Edge::new(self, &rval)));
                self.add_cont(Content::Node(op_node));
                self.add_cont(Content::Node(lval));
                self.add_cont(Content::Node(rval));
            }
            ASTInfo::CallExpr(name, params) => {
                self.name = Some("CallExpr".to_string());

                let name_node = Node::new_symbol(name);

                self.add_cont(Content::Edge(Edge::new(self, &name_node)));
                self.add_cont(Content::Node(name_node));

                if !params.is_empty() {
                    let mut subg = DiGraph::empty();
                    subg.name = Some("Params".to_string());

                    for ast in params {
                        let node = Node::new_subg(DiGraph::from_ast(ast));

                        subg.add_cont(Content::Edge(Edge::new(&subg, &node)));
                        subg.add_cont(Content::Node(node));
                    }

                    let node = Node::new_subg(subg);
                    self.add_cont(Content::Edge(Edge::new(self, &node)));
                    self.add_cont(Content::Node(node));
                }
            }
            ASTInfo::Variable(name, ast) => {
                self.name = Some("Variable".to_string());

                let name_node = Node::new_symbol(name);

                self.add_cont(Content::Edge(Edge::new(self, &name_node)));
                self.add_cont(Content::Node(name_node));

                // Array index
                if let Some(ast) = ast {
                    let mut subg = DiGraph::empty();
                    subg.name = Some("Index".to_string());

                    let index_node = Node::new_subg(DiGraph::from_ast(ast));

                    subg.add_cont(Content::Edge(Edge::new(&subg, &index_node)));
                    subg.add_cont(Content::Node(index_node));

                    let node = Node::new_subg(subg);
                    self.add_cont(Content::Edge(Edge::new(self, &node)));
                    self.add_cont(Content::Node(node));
                }
            }
            ASTInfo::IntLiteral(val) => {
                self.name = Some("IntLiteral".to_string());

                let int_node = Node::new_symbol(&val.to_string());

                self.add_cont(Content::Edge(Edge::new(self, &int_node)));
                self.add_cont(Content::Node(int_node));
            }
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
            buf.push('\n');
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
