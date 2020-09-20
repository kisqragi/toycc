use super::tokenize::{ Token, TokenKind };

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
    ExprStmt,   // Expression statement
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,             // Node kind
    pub lhs: Option<Box<Node>>,     // Left-hand side
    pub rhs: Option<Box<Node>>,     // Right-hand side
    pub val: i64,                   // Used if kind == NodeKind::Num
}

fn new_binary(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Node {
    Node {
        kind,
        lhs: Some(lhs),
        rhs: Some(rhs),
        val: 0,   
    }
}

fn new_unary(kind: NodeKind, expr: Box<Node>) -> Node {
    Node {
        kind,
        lhs: Some(expr),
        rhs: None,
        val: 0,   
    }
}


fn new_num(val: i64) -> Node {
    Node {
        kind: NodeKind::Num,
        lhs: None,
        rhs: None,
        val: val,
    }
}

fn number(tokens: &Vec<Token>, pos: usize) -> Node {
    if tokens[pos].kind == TokenKind::Num {
        let val = tokens[pos].val;
        return new_num(val);
    }
    panic!("number expected, but got {}", tokens[pos].s);
}

// stmt = expr-stmt
fn stmt(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    expr_stmt(&tokens, pos)
}

// expr-stmt = expr ";"
fn expr_stmt(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    let (lhs, mut p) = expr(&tokens, pos);
    let node = new_unary(NodeKind::ExprStmt, Box::new(lhs));
    p = skip(&tokens[p].s, ";", p);
    (node, p)
}

// expr = mul ("+" mul | "-" mul)*
fn expr(tokens: &Vec<Token>, pos: usize) -> (Node, usize) {
    equality(&tokens, pos)
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
        return (new_binary(NodeKind::Sub, Box::new(new_num(0)), Box::new(node)), pos);
    }

    primary(tokens, pos)
}

// primary = "(" expr ")" | num
fn primary(tokens: &Vec<Token>, mut pos: usize) -> (Node, usize) {
    let c = &tokens[pos].s;
    if c == "(" {
        let (node, mut pos) = expr(&tokens, pos+1);
        pos = skip(&tokens[pos].s, ")", pos);
        return (node, pos);
    }

    let node = number(&tokens, pos);
    pos += 1;
    (node, pos)
}

fn skip(tok: &String, s: &str, pos: usize) -> usize {
    if tok != &s {
        panic!("expected '{}'", s);
    }
    pos + 1
}

pub fn reg(idx: usize) -> String {
    let r = ["rdi", "rsi", "r10", "r11", "r12", "r13", "r14", "r15"];
    if r.len() <= idx {
        panic!("register out of range: {}", idx);
    }

    r[idx].to_string()
    
}

// program = stmt*
pub fn parse(tokens: &Vec<Token>) -> Vec<Node> {
    let mut nodes = vec![];

    let mut pos = 0;
    while tokens[pos].kind != TokenKind::Eof {
        let (node, p) = stmt(&tokens, pos);
        nodes.push(node);
        pos = p;
    }

    nodes
}
