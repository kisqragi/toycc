use super::tokenize::{ Token, TokenKind };

#[derive(Debug, PartialEq)]
pub enum NodeKind {
    Num,    // Integer
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Equal,  // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,         // Node kind
    pub lhs: Option<Box<Node>>, // Left-hand side
    pub rhs: Option<Box<Node>>, // Right-hand side
    pub val: i64,               // Used if kind == NodeKind::Num
}

impl Node {
    fn new_binary(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Self {
            kind,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: 0,   
        }
    }

    fn new_num(val: i64) -> Self {
        Self {
            kind: NodeKind::Num,
            lhs: None,
            rhs: None,
            val: val,
        }
    }

    fn number(tokens: &Vec<Token>, pos: usize) -> Self {
        if tokens[pos].kind == TokenKind::Num {
            let val = tokens[pos].val;
            return Self::new_num(val);
        }
        panic!("number expected, but got {}", tokens[pos].s);
    }

    // expr = mul ("+" mul | "-" mul)*
    fn expr(tokens: &Vec<Token>, pos: usize) -> (Self, usize) {
        Self::equality(&tokens, pos)
    }

    // equality = relational ("==" relational | "!=" relational)*
    fn equality(tokens: &Vec<Token>, pos: usize) -> (Self, usize) {
        let (mut node, mut pos) = Self::relational(&tokens, pos);

        loop {

            if tokens.len() == pos {
                return (node, pos);
            }
            let op = &tokens[pos].s;

            if op == "==" {
                let (rhs, p) = Self::relational(&tokens, pos+1);
                node = Self::new_binary(NodeKind::Equal, Box::new(node), Box::new(rhs));
                pos = p;
                continue;
            }

            if op == "!=" {
                let (rhs, p) = Self::relational(&tokens, pos+1);
                node = Self::new_binary(NodeKind::Ne, Box::new(node), Box::new(rhs));
                pos = p;
                continue;
            }

            return (node, pos);
        }
 
    }

    // relational = add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(tokens: &Vec<Token>, pos: usize) -> (Self, usize) {
        let (mut node, mut pos) = Self::add(&tokens, pos);

        loop {

            if tokens.len() == pos {
                return (node, pos);
            }
            let op = &tokens[pos].s;

            if op == "<" {
                let (rhs, p) = Self::add(&tokens, pos+1);
                node = Self::new_binary(NodeKind::Lt, Box::new(node), Box::new(rhs));
                pos = p;
                continue;
            }

            if op == "<=" {
                let (rhs, p) = Self::add(&tokens, pos+1);
                node = Self::new_binary(NodeKind::Le, Box::new(node), Box::new(rhs));
                pos = p;
                continue;
            }

            if op == ">" {
                let (rhs, p) = Self::add(&tokens, pos+1);
                node = Self::new_binary(NodeKind::Lt, Box::new(rhs), Box::new(node));
                pos = p;
                continue;
            }

            if op == ">=" {
                let (rhs, p) = Self::add(&tokens, pos+1);
                node = Self::new_binary(NodeKind::Le, Box::new(rhs), Box::new(node));
                pos = p;
                continue;
            }


            return (node, pos);
        }
 
    }
    
    // add = mul ("+" mul | "-" mul)*
    fn add(tokens: &Vec<Token>, pos: usize) -> (Self, usize) {
        let (mut node, mut pos) = Self::mul(&tokens, pos);

        loop {

            if tokens.len() == pos {
                return (node, pos);
            }
            let op = &tokens[pos].s;

            if op == "+" {
                let (rhs, p) = Self::mul(&tokens, pos+1);
                node = Self::new_binary(NodeKind::Add, Box::new(node), Box::new(rhs));
                pos = p;
                continue;
            }

            if op == "-" {
                let (rhs, p) = Self::mul(&tokens, pos+1);
                node = Self::new_binary(NodeKind::Sub, Box::new(node), Box::new(rhs));
                pos = p;
                continue;
            }

            return (node, pos);
        }
    }

    // mul = unary ("*" unary | "/" unary)*
    fn mul(tokens: &Vec<Token>, pos: usize) -> (Self, usize) {
        let (mut node, mut pos) = Self::unary(&tokens, pos);

        
        loop {
            if tokens.len() == pos {
                return (node, pos);
            }

            let op = &tokens[pos].s;
            if op == "*" {
                let (rhs, p) = Self::unary(&tokens, pos+1);
                node = Self::new_binary(NodeKind::Mul, Box::new(node), Box::new(rhs));
                pos = p;
                continue;
            }

            if op == "/" {
                let (rhs, p) = Self::unary(&tokens, pos+1);
                node = Self::new_binary(NodeKind::Div, Box::new(node), Box::new(rhs));
                pos = p;
                continue;
            }

            return (node, pos);
        }
    }

    // unary = ("+" | "-") unary
    //       | primary
    fn unary(tokens: &Vec<Token>, mut pos: usize) -> (Self, usize) {
        let op = &tokens[pos].s;
        if op == "+" {
            return Self::unary(&tokens, pos+1);
        }
        if op == "-" {
            let (node, p) = Self::unary(&tokens, pos+1);
            pos = p;
            return (Self::new_binary(NodeKind::Sub, Box::new(Self::new_num(0)), Box::new(node)), pos);
        }

        Self::primary(tokens, pos)
    }

    // primary = "(" expr ")" | num
    fn primary(tokens: &Vec<Token>, mut pos: usize) -> (Self, usize) {
        let c = &tokens[pos].s;
        if c == "(" {
            let (node, mut pos) = Self::expr(&tokens, pos+1);
            pos = Self::skip(&tokens[pos].s, ")", pos);
            return (node, pos);
        }

        let node = Self::number(&tokens, pos);
        pos += 1;
        (node, pos)
    }

    fn skip(tok: &String, s: &str, pos: usize) -> usize {
        if tok != &s {
            panic!("expected '{}'", s);
        }
        pos + 1
    }
}

pub fn reg(idx: usize) -> String {
    let r = ["rdi", "rsi", "r10", "r11", "r12", "r13", "r14", "r15"];
    if r.len() <= idx {
        panic!("register out of range: {}", idx);
    }

    r[idx].to_string()
    
}

pub fn parse(tokens: &Vec<Token>) -> Node {
    let (node, _pos) = Node::expr(&tokens, 0);
    node
}
