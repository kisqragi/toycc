use super::types::{ Type, TypeKind };

#[derive(Debug)]
pub struct Ast {
    pub kind: AstKind,
}

#[derive(Debug)]
pub enum AstKind {
    Num(i64),
    UnaryOp(UnaryOp, Box<Ast>),
    BinaryOp(BinaryOp, Box<Ast>, Box<Ast>),
    If {
        cond: Box<Ast>,
        then: Box<Ast>,
        els: Option<Box<Ast>>,
    },
    For {
        init: Option<Box<Ast>>,
        cond: Option<Box<Ast>>,
        inc: Option<Box<Ast>>,
        then: Option<Box<Ast>>,
    },
    Block(Vec<Box<Ast>>),  // {...}
    Return(String, Box<Ast>),     // Return statement
    Var {   // Variable
        name: String,
        ty: Type,
        offset: usize,
    },
    Funcall {   // Function call
        name: String,
        args: Vec<Box<Ast>>,
    },
    Funcdef {
        name: String,
        body: Box<Ast>,
        //locals: Vec<String>,
        params: Vec<Type>,
        stack_size: usize,
    },
    Null,       // Default value of AstKind
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Plus,       // +
    Minus,      // -
    Addr,       // &
    Deref,      // *
    ExprStmt,   // Expression statement
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Assign, // =
}

impl Ast {
    pub fn new(kind: AstKind) -> Self {
        Self { kind }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.kind, AstKind::Num(_))
    }

    pub fn is_pointer(&self) -> bool {
        match &self.kind {
            AstKind::Var { name: _, ty, offset: _ } => {
                ty.kind.is_pointer()
            }
            _ => false
        }
    }
}
