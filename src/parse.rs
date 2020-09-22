use super::tokenize::{ Token, TokenKind };

pub static mut LOCALS: Vec<Var> = vec![];

#[derive(Debug, PartialEq)]
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
    ExprStmt,   // Expression statement
    Return,     // Return statement
    Assign,     // =
    Var,        // Variable
    Null,       // Default value of NodeKind
}

impl Default for NodeKind {
    fn default() -> Self { NodeKind::Null }
}

#[derive(Debug, Default, PartialEq)]
pub struct Node {
    pub kind: NodeKind,             // Node kind
    pub lhs: Option<Box<Node>>,     // Left-hand side
    pub rhs: Option<Box<Node>>,     // Right-hand side

    // "if" or "for" statement
    pub cond: Option<Box<Node>>,
    pub then: Option<Box<Node>>,
    pub els: Option<Box<Node>>,
    pub init: Option<Box<Node>>,
    pub inc: Option<Box<Node>>,

    pub var: Option<usize>,         // Used if kind == NodeKind::Var
    pub val: i64,                   // Used if kind == NodeKind::Num
}

#[derive(Debug, Default, Clone)]
pub struct Var {
    pub name: String,
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
        val: val,
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

fn new_lvar(tokens: &Vec<Token>, pos: usize) -> usize {
    let v = Var {
        name: tokens[pos].s.clone(),
        ..Default::default()
    };
    unsafe { LOCALS.push(v); }
    return unsafe { LOCALS.len()-1 };
}


// stmt = "return" expr ";"
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

    expr_stmt(&tokens, pos)
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
            node = new_binary(NodeKind::Add, Box::new(node), Box::new(rhs));
            pos = p;
            continue;
        }

        if op == "-" {
            let (rhs, p) = mul(&tokens, pos+1);
            node = new_binary(NodeKind::Sub, Box::new(node), Box::new(rhs));
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

// unary = ("+" | "-") unary
//       | primary
fn unary(tokens: &Vec<Token>, mut pos: usize) -> (Node, usize) {
    let op = &tokens[pos].s;
    if op == "+" {
        return unary(&tokens, pos+1);
    }
    if op == "-" {
        let (node, p) = unary(&tokens, pos+1);
        pos = p;
        return (new_binary(NodeKind::Sub, Box::new(get_number(0)), Box::new(node)), pos);
    }

    primary(tokens, pos)
}

// primary = "(" expr ")" | ident | num 
fn primary(tokens: &Vec<Token>, mut pos: usize) -> (Node, usize) {
    let c = &tokens[pos].s;
    if c == "(" {
        let (node, mut pos) = expr(&tokens, pos+1);
        pos = skip(&tokens, ")", pos);
        return (node, pos);
    }

    let node = match tokens[pos].kind {
        TokenKind::Num => { new_num(&tokens, pos) }
        TokenKind::Ident => { 
            let var = match find_var(&tokens, pos) {
                Some(v) => { v }
                None => { new_lvar(&tokens, pos) }
            };
            new_var_node(var)
        }
        _ => panic!("invalid primary: {}", tokens[pos].s)
    };

    pos += 1;
    (node, pos)
}

fn skip(tokens: &Vec<Token>, s: &str, pos: usize) -> usize {
    if &tokens[pos].s != s {
        panic!("expected '{}'", s);
    }
    pos + 1
}

#[derive(Debug)]
pub struct Function {
    pub nodes: Vec<Node>,
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

    Function {
        nodes,
        stack_size: 0,
    }
}
