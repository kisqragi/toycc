use std::env;
use std::process;
use std::iter::Iterator;

const REGS: [&str; 8] = ["rdi", "rsi", "r10", "r11", "r12", "r13", "r14", "r15"];
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
}

#[derive(Debug)]
struct Node {
    kind: NodeKind,         // Node kind
    lhs: Option<Box<Node>>, // Left-hand side
    rhs: Option<Box<Node>>, // Right-hand side
    val: i64,               // Used if kind == NodeKind::Num
}

impl Node {
    fn new(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Self {
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

    fn expr(tokens: Vec<Token>) -> Self {
        let mut pos = 0;
        let mut lhs = Self::number(&tokens, pos);
        pos += 1;

        loop {

            let op = &tokens[pos].s;

            if op == "+" {
                let rhs = Self::number(&tokens, pos+1);
                lhs = Self::new(NodeKind::Add, Box::new(lhs), Box::new(rhs));
                pos += 2;
                continue;
            }

            if op == "-" {
                let rhs = Self::number(&tokens, pos+1);
                lhs = Self::new(NodeKind::Sub, Box::new(lhs), Box::new(rhs));
                pos += 2;
                continue;
            }

            return lhs;
        }
    }

    fn gen(self) -> String {
        if self.kind == NodeKind::Num {
            unsafe {
                let reg;
                if CUR >= REGS.len() {
                    panic!("register exhausted");
                }
                reg = REGS[CUR];
                CUR += 1;
                println!("  mov {}, {}", reg, self.val);
                return reg.into();
            }
        }

        let dst = self.lhs.unwrap().gen();
        let src = self.rhs.unwrap().gen();

        match self.kind {
            NodeKind::Add => {
                println!("  add {}, {}", dst, src);
                return dst;
            }
            NodeKind::Sub => {
                println!("  sub {}, {}", dst, src);
                return dst;
            }
            _ => panic!("unknown operator")
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
        // Skip whitespace characters.
        if lexer.getc().unwrap().is_whitespace() {
            lexer.next_pos();
            continue;
        }

        // Numeric literal
        if lexer.getc().unwrap().is_ascii_digit() {
            let kind = TokenKind::Num;
            let loc = lexer.pos;
            let val = lexer.strtol().unwrap();
            let s = lexer.code[loc..lexer.pos].iter().collect();
            let token = Token { kind, val, s, loc };
            tokens.push(token);
            continue;
        }

        // Punctuator
        if lexer.getc().unwrap() == &'+' || lexer.getc().unwrap() == &'-' {
            let token = Token {
                kind: TokenKind::Reserved,
                val : 0,
                s   : lexer.getc().unwrap().to_string(),
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
    let node = Node::expr(tokens);

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    println!("  mov rax, {}", node.gen());

    println!("  ret");
    println!("  ");
    
}
