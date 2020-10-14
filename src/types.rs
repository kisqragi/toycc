//use super::parse::{ Node, NodeKind::* };
//use super::tokenize::Token;

#[derive(Debug, PartialEq, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeKind {
    Int,
    Ptr(Box<Type>),
    Func {
        ty: Box<Type>,
        params: Vec<Type>,
    },
    _None,   // Default
}

impl TypeKind {
    pub fn is_pointer(&self) -> bool {
        matches!(self, TypeKind::Ptr(_))
    }
}

impl Type {
    pub fn new(kind: TypeKind) -> Self {
        Type { kind, name: "".to_string() }
    }

    pub fn get_params(self) -> Vec<Type> {
        match self.kind {
            TypeKind::Func{ ty: _, params } => { params }
            _ => panic!()
        }
    }
}

/*
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Type {
    pub kind: TypeKind,

    // Pointer
    pub base: Option<Box<Type>>,

    // Declaration
    pub name: Option<Token>,

    // Function type
    pub return_ty: Option<Box<Type>>,
    pub params: Vec<Type>,
}

pub fn is_integer(ty: &Type) -> bool {
    ty.kind == TypeKind::Int
}
*/

pub fn pointer_to(base: Type) -> Type {
    Type::new(TypeKind::Ptr(Box::new(base)))
}

pub fn ty_int() -> Type {
    Type::new(TypeKind::Int)
}

pub fn func_type(return_ty: Type, params: Vec<Type>) -> Type {
    Type::new(TypeKind::Func {
        ty: Box::new(return_ty),
        params
    })
}

pub fn copy_type(ty: Type) -> Type {
    let ret = ty.clone();
    ret
}

/*
pub fn add_type(node: &mut Node) -> Node {

    if let Some(n) = &node.lhs { node.lhs = Some(Box::new(add_type(&mut n.as_ref().clone()))); }
    if let Some(n) = &node.rhs { node.rhs = Some(Box::new(add_type(&mut n.as_ref().clone()))); }
    if let Some(n) = &node.cond { node.cond = Some(Box::new(add_type(&mut n.as_ref().clone()))); }
    if let Some(n) = &node.then { node.then = Some(Box::new(add_type(&mut n.as_ref().clone()))); }
    if let Some(n) = &node.els { node.els = Some(Box::new(add_type(&mut n.as_ref().clone()))); }
    if let Some(n) = &node.init { node.init = Some(Box::new(add_type(&mut n.as_ref().clone()))); }
    if let Some(n) = &node.inc { node.inc = Some(Box::new(add_type(&mut n.as_ref().clone()))); }

    match node.kind {
        Add | Sub | Mul | Div | Assign => {
            node.ty = node.lhs.as_ref().unwrap().ty.clone()
        }
        Equal | Ne | Lt | Le | Var | Num | Funcall => {
            node.ty = ty_int()
        }
        Addr => node.ty = pointer_to(node.lhs.as_ref().unwrap().ty.clone()),
        Deref => {
            if node.lhs.as_ref().unwrap().ty.kind == TypeKind::Ptr {
                node.ty = *node.lhs.as_ref().unwrap().ty.base.as_ref().unwrap().clone()
            } else {
                node.ty = ty_int()
            }
        }
        _ =>  {} 
    }

    return node.clone();
}
*/
