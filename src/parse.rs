use super::tokenize::{ Token, TokenKind };
use super::types::{ Type, add_type, is_integer, ty_int, pointer_to };
use std::process::exit;

pub static mut LOCALS: Vec<Var> = vec![];

#[derive(Debug, PartialEq, Clone)]
pub enum NodeKind {
    Num,        // Integer
    Add,        // +
    Sub,        // -
    Mul,        // *
    Div,        // /
    Equal,      // ==
    Ne,         // !=
    Lt,         // <
    Le,         // <=
    If,         // "if"
    For,        // "for"
    Block,      // { ... }
    ExprStmt,   // Expression statement
    Return,     // Return statement
    Assign,     // =
    Addr,       // &
    Deref,      // *
    Var,        // Variable
    Funcall,    // Function call
    Null,       // Default value of NodeKind
}

impl Default for NodeKind {
    fn default() -> Self { NodeKind::Null }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Node {
    pub kind: NodeKind,             // Node kind
    pub ty: Type,                   // Type, e.g. int or pointer to int

    pub lhs: Option<Box<Node>>,     // Left-hand side
    pub rhs: Option<Box<Node>>,     // Right-hand side

    // "if" or "for" statement
    pub cond: Option<Box<Node>>,
    pub then: Option<Box<Node>>,
    pub els: Option<Box<Node>>,
    pub init: Option<Box<Node>>,
    pub inc: Option<Box<Node>>,

    // Block
    pub body: Option<Vec<Box<Node>>>,

    // Function call
    pub funcname: String,

    pub var: Option<usize>,         // Used if kind == NodeKind::Var
    pub val: i64,                   // Used if kind == NodeKind::Num
}

#[derive(Debug, Default, Clone)]
pub struct Var {
    pub name: String,
    pub ty: Type,
    pub offset: usize,
}

fn find_var(tokens: &Vec<Token>, pos: usize) -> Option<usize> {
    unsafe {
        for (i, var) in LOCALS.iter().enumerate() {
            if tokens[pos].s == var.name {
                return Some(i);
            }
        }
    }
    None 
}

fn new_binary(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Node {
    Node {
        kind,
        lhs: Some(lhs),
        rhs: Some(rhs),
        ..Default::default()
    }
}

fn new_unary(kind: NodeKind, expr: Box<Node>) -> Node {
    Node {
        kind,
        lhs: Some(expr),
        ..Default::default()
    }
}

fn get_number(val: i64) -> Node {
    Node {
        kind: NodeKind::Num,
        val,
        ..Default::default()
    }
}

fn new_num(tokens: &Vec<Token>, pos: usize) -> Node {
    if tokens[pos].kind == TokenKind::Num {
        let val = tokens[pos].val;
        return get_number(val);
    }
    panic!("number expected, but got {}", tokens[pos].s);
}

fn new_var_node(var: usize) -> Node {
    Node {
        kind: NodeKind::Var,
        var: Some(var),
        ..Default::default()
    }
}

fn new_lvar(tokens: &Vec<Token>, pos: usize, ty: Type) -> usize {
    let v = Var {
        name: tokens[pos].s.clone(),
        ty,
        ..Default::default()
    };
    unsafe { LOCALS.push(v); }
    return unsafe { LOCALS.len()-1 };
}


// stmt = "return" expr ";"
//      | "{" compound-stmt
//      | "if" "(" expr ")" stmt ("else" stmt)?
//      | "for" "(" expr? ";" expr? ";" expr? ")" stmt
//      | "while" "(" expr ")" stmt
//      | expr-stmt
fn stmt(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    if tokens[pos].s == "return" {
        let (lhs, mut p) = expr(&tokens, pos+1);
        let node = new_unary(NodeKind::Return, Box::new(lhs));
        p = skip(&tokens, ";", p);
        return (node, p);
    }

    // "if" statement
    if tokens[pos].s == "if" {
        let mut node = Node { kind: NodeKind::If, ..Default::default() };

        let p = skip(&tokens, "(", pos+1);

        // set cond
        let (cond, mut p) = expr(&tokens, p);
        node.cond = Some(Box::new(cond));

        p = skip(&tokens, ")", p);

        // set then 
        let (then, mut p) = stmt(&tokens, p);
        node.then = Some(Box::new(then));

        // "else"
        if tokens[p].s == "else" {
            let (t, pt) = stmt(&tokens, p+1);
            node.els = Some(Box::new(t));
            p = pt;
        }

        return (node, p);
    }

    // "for" statement
    if tokens[pos].s == "for" {
        let mut node = Node { kind: NodeKind::For, ..Default::default() };

        let mut p = skip(&tokens, "(", pos+1);

        // initとincは値を返さない
        // init
        if tokens[p].s != ";" {
            let (init, p2) = expr(&tokens, p);
            node.init = Some(Box::new(new_unary(NodeKind::ExprStmt, Box::new(init))));
            p = p2;
        }
        p = skip(&tokens, ";", p);

        // cond 
        if tokens[p].s != ";" {
            let (cond, p2) = expr(&tokens, p);
            node.cond = Some(Box::new(cond));
            p = p2;
        }
        p = skip(&tokens, ";", p);

        if tokens[p].s != ")" {
            let (inc, p2) = expr(&tokens, p);
            node.inc = Some(Box::new(new_unary(NodeKind::ExprStmt, Box::new(inc))));
            p = p2;
        }
        p = skip(&tokens, ")", p);

        let (then, p) = stmt(&tokens, p);
        node.then = Some(Box::new(then));

        return (node, p);
    }

    if tokens[pos].s == "while" {
        let mut node = Node { kind: NodeKind::For, ..Default::default() };
        let p = skip(&tokens, "(", pos+1);

        let (cond, mut p) = expr(&tokens, p);
        node.cond = Some(Box::new(cond));
        p = skip(&tokens, ")", p);

        let (then, p) = stmt(&tokens, p);
        node.then = Some(Box::new(then));

        return (node, p);
    }

    if tokens[pos].s == "{" {
        let (body, p) = compound_stmt(&tokens, pos+1);
        return (body, p);
    }

    expr_stmt(&tokens, pos)
}

// compound-stmt = (declaration | stmt)* "}"
fn compound_stmt(tokens: &Vec<Token>, mut pos: usize) -> (Node, usize) {
    let mut node = Node { kind: NodeKind::Block, ..Default::default() };

    let mut body: Vec<Box<Node>> = vec![];
    while tokens[pos].s != "}" {
        if tokens[pos].s == "int" {
            let (mut node, p) = declaration(&tokens, pos);
            body.push(Box::new(add_type(&mut node)));
            pos = p;
        } else {
            let (mut node, p) = stmt(&tokens, pos);
            body.push(Box::new(add_type(&mut node)));
            pos = p;
        }
    }

    node.body = Some(body);
    return (node, pos+1);
}

// declaration = typespec (declarator ("=" expr)? ("," declarator ("=" expr)?)*)? ";"
fn declaration(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    let (basety, mut pos) = typespec(&tokens, pos);

    let mut body: Vec<Box<Node>> = vec![];
    let mut i = 0;
    loop {
        if tokens[pos].s == ";" { pos += 1; break; }

        if i > 0 {
            let p = skip(&tokens, ",", pos);
            pos = p;
        }
        i += 1;

        let (ty, p) = declarator(&tokens, pos, basety.clone());
        pos = p;
        let var = new_lvar(&tokens, pos-1, ty);

        if tokens[pos].s != "=" { continue; }

        let lhs = new_var_node(var);
        let (rhs, p) = assign(&tokens, pos-1);
        pos = p;
        let node = new_binary(NodeKind::Assign, Box::new(lhs), Box::new(rhs));
        body.push(Box::new(new_unary(NodeKind::ExprStmt, Box::new(node))));
    }


    let mut node = Node { kind: NodeKind::Block, ..Default::default() };
    node.body = Some(body);
    (node, pos)
}

// typespec = "int"
fn typespec(tokens: &Vec<Token>, mut pos: usize) -> (Type, usize) {
    pos = skip(&tokens, "int", pos);
    (ty_int(), pos)
}

// declarator = "*"* ident
fn declarator(tokens: &Vec<Token>, mut pos: usize, mut ty: Type) -> (Type, usize) {
    loop {
        let (f, p) = consume(&tokens, "*", pos);
        pos = p;
        if !f { break; }
        ty = pointer_to(ty);
    }

    if tokens[pos].kind != TokenKind::Ident {
        eprintln!("expected a variable name: {}", tokens[pos].s);
    }

    ty.name = Some(tokens[pos].clone());
    (ty, pos+1)
}

// expr-stmt = expr ";"
fn expr_stmt(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    let (lhs, mut p) = expr(&tokens, pos);
    let node = new_unary(NodeKind::ExprStmt, Box::new(lhs));
    p = skip(&tokens, ";", p);
    (node, p)
}

// expr =  assign
fn expr(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    assign(&tokens, pos)
}

// assign = equality ("=" assign)?
fn assign(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    let (mut node, mut pos) = equality(&tokens, pos);
    let op = &tokens[pos].s;
    if op == "=" {
        let (rhs, p) = assign(&tokens, pos+1);
        node = new_binary(NodeKind::Assign, Box::new(node), Box::new(rhs));  
        pos = p;
    }

    (node, pos)
}

// equality = relational ("==" relational | "!=" relational)*
fn equality(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    let (mut node, mut pos) = relational(&tokens, pos);

    loop {

        if tokens.len() == pos {
            return (node, pos);
        }
        let op = &tokens[pos].s;

        if op == "==" {
            let (rhs, p) = relational(&tokens, pos+1);
            node = new_binary(NodeKind::Equal, Box::new(node), Box::new(rhs));
            pos = p;
            continue;
        }

        if op == "!=" {
            let (rhs, p) = relational(&tokens, pos+1);
            node = new_binary(NodeKind::Ne, Box::new(node), Box::new(rhs));
            pos = p;
            continue;
        }

        return (node, pos);
    }

}

// relational = add ("<" add | "<=" add | ">" add | ">=" add)*
fn relational(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    let (mut node, mut pos) = add(&tokens, pos);

    loop {

        if tokens.len() == pos {
            return (node, pos);
        }
        let op = &tokens[pos].s;

        if op == "<" {
            let (rhs, p) = add(&tokens, pos+1);
            node = new_binary(NodeKind::Lt, Box::new(node), Box::new(rhs));
            pos = p;
            continue;
        }

        if op == "<=" {
            let (rhs, p) = add(&tokens, pos+1);
            node = new_binary(NodeKind::Le, Box::new(node), Box::new(rhs));
            pos = p;
            continue;
        }

        if op == ">" {
            let (rhs, p) = add(&tokens, pos+1);
            node = new_binary(NodeKind::Lt, Box::new(rhs), Box::new(node));
            pos = p;
            continue;
        }

        if op == ">=" {
            let (rhs, p) = add(&tokens, pos+1);
            node = new_binary(NodeKind::Le, Box::new(rhs), Box::new(node));
            pos = p;
            continue;
        }


        return (node, pos);
    }

}

fn new_add(mut lhs: Node, mut rhs: Node) -> Node {
    lhs = add_type(&mut lhs);
    rhs = add_type(&mut rhs);

    // num + num
    if is_integer(&lhs.ty) && is_integer(&rhs.ty) {
        return new_binary(NodeKind::Add, Box::new(lhs.clone()), Box::new(rhs.clone()));
    }

    if lhs.ty.base != None && rhs.ty.base != None {
        eprintln!("invalid operands");
        eprintln!("lhs = {:#?}", lhs);
        eprintln!("rhs = {:#?}", rhs);
    }

    // Canonicalize `num + ptr` to `ptr + num`.
    if lhs.ty.base == None && rhs.ty.base != None {
        let tmp = lhs;
        lhs = rhs;
        rhs = tmp;
    }

    // ptr + num
    rhs = new_binary(NodeKind::Mul, Box::new(rhs), Box::new(get_number(8)));
    new_binary(NodeKind::Add, Box::new(lhs), Box::new(rhs))
}

fn new_sub(mut lhs: Node, mut rhs: Node) -> Node {
    lhs = add_type(&mut lhs);
    rhs = add_type(&mut rhs);

    // num - num
    if is_integer(&lhs.ty) && is_integer(&rhs.ty) {
        return new_binary(NodeKind::Sub, Box::new(lhs.clone()), Box::new(rhs.clone()));
    }

    // ptr - num
    if lhs.ty.base != None && is_integer(&rhs.ty) {
        rhs = new_binary(NodeKind::Mul, Box::new(rhs), Box::new(get_number(8)));
        return new_binary(NodeKind::Sub, Box::new(lhs), Box::new(rhs));
    }

    // num - ptr (error)
    if lhs.ty.base == None && rhs.ty.base != None {
        eprintln!("invalid operands");
        eprintln!("lhs = {:#?}", lhs);
        eprintln!("rhs = {:#?}", rhs);
    }

    // `ptr-ptr` returns the result of `ptr-ptr` divided by its size.
    // The result is a number of elements, but the value can also be negative.
    lhs = new_binary(NodeKind::Sub, Box::new(lhs), Box::new(rhs));
    new_binary(NodeKind::Div, Box::new(lhs), Box::new(get_number(8)))
}

// add = mul ("+" mul | "-" mul)*
fn add(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    let (mut node, mut pos) = mul(&tokens, pos);

    loop {

        if tokens.len() == pos {
            return (node, pos);
        }
        let op = &tokens[pos].s;

        if op == "+" {
            let (rhs, p) = mul(&tokens, pos+1);
            node = new_add(node, rhs);
            pos = p;
            continue;
        }

        if op == "-" {
            let (rhs, p) = mul(&tokens, pos+1);
            node = new_sub(node, rhs);
            pos = p;
            continue;
        }

        return (node, pos);
    }
}

// mul = unary ("*" unary | "/" unary)*
fn mul(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    let (mut node, mut pos) = unary(&tokens, pos);

    
    loop {
        if tokens.len() == pos {
            return (node, pos);
        }

        let op = &tokens[pos].s;
        if op == "*" {
            let (rhs, p) = unary(&tokens, pos+1);
            node = new_binary(NodeKind::Mul, Box::new(node), Box::new(rhs));
            pos = p;
            continue;
        }

        if op == "/" {
            let (rhs, p) = unary(&tokens, pos+1);
            node = new_binary(NodeKind::Div, Box::new(node), Box::new(rhs));
            pos = p;
            continue;
        }

        return (node, pos);
    }
}

// unary = ("+" | "-" | "&" | "*")? unary
//       | primary
fn unary(tokens: &Vec<Token>, mut pos: usize) -> (Node, usize) {
    match *(&tokens[pos].s.as_str()) {
        "+" => return unary(&tokens, pos+1),
        "-" => {
            let (node, p) = unary(&tokens, pos+1);
            pos = p;
            return (new_binary(NodeKind::Sub, Box::new(get_number(0)), Box::new(node)), pos);
        }
        "&" => {
            let (node, p) = unary(&tokens, pos+1);
            pos = p;
            return (new_unary(NodeKind::Addr, Box::new(node)), pos);
        }
        "*" => {
            let (node, p) = unary(&tokens, pos+1);
            pos = p;
            return (new_unary(NodeKind::Deref, Box::new(node)), pos);
        }
        _ => return primary(tokens, pos)
    };
}

// primary = "(" expr ")" | ident args? | num 
// args = "(" ")"
fn primary(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    let c = &tokens[pos].s;
    if c == "(" {
        let (node, mut pos) = expr(&tokens, pos+1);
        pos = skip(&tokens, ")", pos);
        return (node, pos);
    }

    if tokens[pos].kind == TokenKind::Ident {
        // Function call
        if tokens[pos+1].s == "(" {
            let node = Node {
                kind: NodeKind::Funcall,
                funcname: tokens[pos].s.clone(),
                ..Default::default()
            };
            let pos = skip(&tokens, ")", pos+2);
            return (node, pos);
        }

        // Variable
        let var = find_var(&tokens, pos);
        if var == None {
            eprintln!("undefined variable: {}", tokens[pos].s);
            exit(1);
        }
        return (new_var_node(var.unwrap()), pos+1);
    }

    let node = new_num(&tokens, pos);
    (node, pos+1)
}

fn skip(tokens: &Vec<Token>, s: &str, pos: usize) -> usize {
    if &tokens[pos].s != s {
        panic!("expected '{}'", s);
    }
    pos + 1
}

// トークンが期待するトークンの場合、トークンを一つ消費して
// 真を返す。違う場合偽を返す。
fn consume(tokens: &Vec<Token>, s: &str, mut pos: usize) -> (bool, usize) {
    if &tokens[pos].s == s {
        pos += 1;
        return (true, pos);
    }
    (false, pos)
}


#[derive(Debug)]
pub struct Function {
    pub body: Node,
    pub stack_size: usize,
}

// program = stmt*
pub fn parse(tokens: &Vec<Token>) -> Function {
    let mut nodes = vec![];

    let mut pos = 0;
    while tokens[pos].kind != TokenKind::Eof {
        let (node, p) = stmt(&tokens, pos);
        nodes.push(node);
        pos = p;
    }

    let mut pos = 0;
    pos = skip(&tokens, "{", pos);
    let (body, _pos) = compound_stmt(&tokens, pos);

    Function {
        body,
        stack_size: 0,
    }
}
