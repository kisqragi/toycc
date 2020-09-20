use std::env;
use std::process;
use std::iter::Iterator;

static mut CUR: usize = 0;

#[derive(Debug)]
struct Lexer {
    code: Vec<char>,
    pos: usize,
}

impl Lexer {

    fn new(args: &String) -> Lexer {
        let code: Vec<char> = args.chars().collect();
        Lexer { 
            code,
            pos: 0 
        }
    }

    // codeのpos番目の文字を取得
    fn getc(&self) -> Option<&char> {
        self.code.get(self.pos)
    }

    // posを1つ進める
    fn next_pos(&mut self) {
        self.pos += 1;
    }

    // posがcodeの最後の示しているか
    fn is_last(&self) -> bool {
        self.code.len() == self.pos
    }

    // 12+34があったら12までをi64に変換し、
    // posを進める。posは+の位置になる
    fn strtol(&mut self) -> Option<i64> {
        let mut s = String::new();
        while let Some(c) = self.code.get(self.pos) {
            if c.is_ascii_digit() {
                s.push_str(&c.to_string());
                self.pos += 1;
            } else {
                break;
            }
        }

        Some(s.parse::<i64>().unwrap())
    }
}

#[derive(Debug, PartialEq)]
enum TokenKind {
    Reserved,
    Num,
    Eof,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,    // Kind of Token
    val : i64,          // Number literal
    s   : String,       // String of Token
    loc : usize,
}

#[derive(Debug, PartialEq)]
enum NodeKind {
    Num,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
struct Node {
    kind: NodeKind,         // Node kind
    lhs: Option<Box<Node>>, // Left-hand side
    rhs: Option<Box<Node>>, // Right-hand side
    val: i64,               // Used if kind == NodeKind::Num
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

    fn parse(tokens: &Vec<Token>) -> Self {
        let (node, _pos) = Self::expr(&tokens, 0);
        node
    }

    fn skip(tok: &String, s: &str, pos: usize) -> usize {
        if tok != &s {
            panic!("expected '{}'", s);
        }
        pos + 1
    }

    fn reg(idx: usize) -> String {
        let r = ["rdi", "rsi", "r10", "r11", "r12", "r13", "r14", "r15"];
        if r.len() <= idx {
            panic!("register out of range: {}", idx);
        }

        r[idx].to_string()
        
    }

    fn gen_expr(self) {
        if self.kind == NodeKind::Num {
            unsafe {
                println!("  mov {}, {}", Self::reg(CUR), self.val);
                CUR += 1;
                return;
            }
        }

        self.lhs.unwrap().gen_expr();
        self.rhs.unwrap().gen_expr();

        let rd;
        let rs;
        unsafe {
            rd = Self::reg(CUR-2);
            rs = Self::reg(CUR-1);
            CUR -= 1;
        }

        match self.kind {
            NodeKind::Add => {
                println!("  add {}, {}", rd, rs);
            }
            NodeKind::Sub => {
                println!("  sub {}, {}", rd, rs);
            }
            NodeKind::Mul => {
                println!("  imul {}, {}", rd, rs);
            }
            NodeKind::Div => {
                println!("  mov rax, {}", rd);
                println!("  cqo");
                println!("  idiv {}", rs);
                println!("  mov {}, rax", rd);
            }
            _ => panic!("invalid expression")
        }
    }
}

fn error_at(lexer: &Lexer, pos: usize, s: String) {
    for c in lexer.code.iter() {
        print!("{}", c);
    }
    println!();

    for _ in 0..pos {
        print!(" ");
    }
    print!("^ ");
    println!("{}", s);
    process::exit(1);
}

fn tokenize(lexer: &mut Lexer) -> Vec<Token> {
    let mut tokens = vec![];
    while !lexer.is_last() {
        let c = lexer.getc().unwrap();
        // Skip whitespace characters.
        if c.is_whitespace() {
            lexer.next_pos();
            continue;
        }

        // Numeric literal
        if c.is_ascii_digit() {
            let kind = TokenKind::Num;
            let loc = lexer.pos;
            let val = lexer.strtol().unwrap();
            let s = lexer.code[loc..lexer.pos].iter().collect();
            let token = Token { kind, val, s, loc };
            tokens.push(token);
            continue;
        }

        // Punctuator
        if c == &'+' || c == &'-' || c == &'*' || c == &'/' || c == &'(' || c == &')' {
            let token = Token {
                kind: TokenKind::Reserved,
                val : 0,
                s   : c.to_string(),
                loc : lexer.pos,
            };
            lexer.next_pos();
            tokens.push(token);
            continue;
        }

        error_at(lexer, lexer.pos, "invalid token".to_string());

    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        val : 0,
        s   : "".to_string(),
        loc : lexer.pos,
    });

    return tokens;

}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("{}: invalid number of arguments", args[0]);
        process::exit(1);
    }

    let mut lexer = Lexer::new(&args[1]);
    let tokens = tokenize(&mut lexer);
    let node = Node::parse(&tokens);

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // Save callee-saved registers.
    println!("  push r12");
    println!("  push r13");
    println!("  push r14");
    println!("  push r15");

    node.gen_expr();

    unsafe {
        println!("  mov rax, {}", Node::reg(CUR-1));
    }

    println!("  pop r15");
    println!("  pop r14");
    println!("  pop r13");
    println!("  pop r12");
    println!("  ret");
    println!("  ");
    
}
