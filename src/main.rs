use std::env;
use std::process;
use std::iter::Iterator;

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
    TkReserved,
    TkNum,
    TkEof,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,    // Kind of Token
    val : Option<i64>,          // Number literal
    s   : Option<String>,       // String of Token
    loc : usize,
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
            let kind = TokenKind::TkNum;
            let loc = lexer.pos;
            let val = Some(lexer.strtol().unwrap());
            let s = Some(lexer.code[loc..lexer.pos].iter().collect());
            let token = Token { kind, val, s, loc };
            tokens.push(token);
            continue;
        }

        // Punctuator
        if lexer.getc().unwrap() == &'+' || lexer.getc().unwrap() == &'-' {
            let token = Token {
                kind: TokenKind::TkReserved,
                val : None,
                s   : Some(lexer.getc().unwrap().to_string()),
                loc : lexer.pos,
            };
            lexer.next_pos();
            tokens.push(token);
            continue;
        }

        error_at(lexer, lexer.pos, "invalid token".to_string());

    }

    tokens.push(Token {
        kind: TokenKind::TkEof,
        val : None,
        s   : None,
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
    let mut tok_pos = 0;

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    println!("  mov rax, {}", tokens[tok_pos].val.unwrap());
    tok_pos += 1;

    while tokens[tok_pos].kind != TokenKind::TkEof {

        if tokens[tok_pos].s.as_ref().unwrap() == "+" {
            println!("  add rax, {}", tokens[tok_pos+1].val.unwrap());
            tok_pos += 2;
            continue;
        }

        if tokens[tok_pos].s.as_ref().unwrap() == "-" {
            println!("  sub rax, {}", tokens[tok_pos+1].val.unwrap());
            tok_pos += 2;
            continue;
        }

        eprintln!("unexpected charater: {:?}", tokens[tok_pos]);
        process::exit(1);
    }

    println!("  ret");
    println!("  ");
    
}
