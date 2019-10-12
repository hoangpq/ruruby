use crate::lexer::{Annot, Loc};

/*
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub loc: Loc,
}
*/
pub type Node = Annot<NodeKind>;

impl Node {
    pub fn new_number(num: i64, loc: Loc) -> Self {
        Node::new(NodeKind::Number(num), loc)
    }

    pub fn new_comp_stmt() -> Self {
        Node::new(NodeKind::CompStmt(vec![]), Loc(0, 0))
    }

    pub fn new_binop(op: BinOp, lhs: Node, rhs: Node, loc: Loc) -> Self {
        let kind = NodeKind::BinOp(op, Box::new(lhs), Box::new(rhs));
        Node::new(kind, loc)
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            NodeKind::BinOp(op, lhs, rhs) => write!(f, "[{:?} ( {}, {} )]", op, lhs, rhs),
            NodeKind::CompStmt(nodes) => write!(f, "[{:?}]", nodes),
            _ => write!(f, "[{:?}]", self.kind),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Number(i64),
    BinOp(BinOp, Box<Node>, Box<Node>),
    Assign(Box<Node>, Box<Node>),
    CompStmt(Vec<Node>),
    If(Box<Node>, Box<Node>, Box<Node>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Eq,
}
