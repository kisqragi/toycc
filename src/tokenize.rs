use std::process;

#[derive(Debug)]
pub struct Lexer {
    code: Vec<char>,
    pos: usize,
}

impl Lexer {

    pub fn new(args: &String) -> Lexer {
        let code: Vec<char> = args.chars().collect();
        Lexer { 
            code,
            pos: 0 
        }
    }

    // codeのpos番目の文字を取得
    fn getc(&self) -> &char {
        match self.code.get(self.pos) {
            Some(c) => c,
            None => panic!("doesn’t exists token")
        }
    }

    // posをn進める
    fn next_pos(&mut self, n: usize) {
        self.pos += n;
    }

    // posがcodeの最後の示しているか
    fn is_last(&self) -> bool {
        self.code.len() == self.pos
    }

    // 12+34があったら12までをi64に変換し、
    // posを進める。posは+の位置になる
    fn strtol(&mut self) -> i64 {
        let mut s = String::new();
        while let Some(c) = self.code.get(self.pos) {
            if c.is_ascii_digit() {
                s.push_str(&c.to_string());
                self.pos += 1;
            } else {
                break;
            }
        }

        match s.parse::<i64>() {
            Ok(n) => n,
            Err(e) => panic!("can't convert to numbers: {}", e)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Reserved,   // Keywords or punctuators
    Ident,      // Identifiers
    Num,        // Numeric literal
    Eof,        // End-of-file markers
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,    // Kind of Token
    pub val : i64,          // Number literal
    pub s   : String,       // String of Token
    pub loc : usize,
}

fn error_at(lexer: &Lexer, pos: usize, s: &str) {
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

fn ispunct(c: &char) -> bool {
    // punct of C
    let mut punct = [
        '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-',
        '.', '/', ':', ';', '<', '=', '>', '?', '@', '[', '\\', ']', '^',
        '_', '`', '{', '|', '}'
    ].iter();

    match punct.find(|x| x == &c) {
        Some(_) => true,
        None => false,
    }
}

fn is_alpha(c: &char) -> bool {
    return ('a' <= *c && *c <= 'z') || ('A' <= *c && *c <= 'Z') || *c == '_';
}

fn is_alnum(c: &char) -> bool {
    return is_alpha(&c) || ('0' <= *c && *c <= '9');
}

fn startswith(vc: &[char], s: &str) -> bool {
    for i in 0..s.len() {
        if vc.get(i).unwrap() != &s.chars().nth(i).unwrap() {
            return false;
        }
    }
    true
}

pub fn tokenize(lexer: &mut Lexer) -> Vec<Token> {
    let mut tokens = vec![];
    while !lexer.is_last() {
        let c = lexer.getc();
        // Skip whitespace characters.
        if c.is_whitespace() {
            lexer.next_pos(1);
            continue;
        }

        // Numeric literal
        if c.is_ascii_digit() {
            let kind = TokenKind::Num;
            let loc = lexer.pos;
            let val = lexer.strtol();
            let s = lexer.code[loc..lexer.pos].iter().collect();
            let token = Token { kind, val, s, loc };
            tokens.push(token);
            continue;
        }

        // Keywords
        if startswith(&lexer.code[lexer.pos..], "return") && !is_alnum(lexer.code.get(lexer.pos+6).unwrap()) {
            let s: String = lexer.code[lexer.pos..(lexer.pos+6)].iter().collect();
            let token = Token {
                kind: TokenKind::Reserved,
                val : 0,
                s,
                loc : lexer.pos,
            };
            lexer.next_pos(6);
            tokens.push(token);
            continue;
        }

        if &'a' <= c && c <= &'z' {
            let token = Token {
                kind: TokenKind::Ident,
                val : 0,
                s: c.to_string(),
                loc : lexer.pos,
            };
            lexer.next_pos(1);
            tokens.push(token);
            continue;
        }

        // Multi-letter punctuators
        // ==, !=, <= and >=
        let op = &lexer.code[lexer.pos..];
        if startswith(op, "==") || startswith(op, "!=") ||
           startswith(op, "<=") || startswith(op, ">=") {
            let s: String = lexer.code[lexer.pos..(lexer.pos+2)].iter().collect();
            let token = Token {
                kind: TokenKind::Reserved,
                val : 0,
                s,
                loc : lexer.pos,
            };
            lexer.next_pos(2);
            tokens.push(token);
            continue;
        }

        // Punctuator
        if ispunct(c) {
            let token = Token {
                kind: TokenKind::Reserved,
                val : 0,
                s   : c.to_string(),
                loc : lexer.pos,
            };
            lexer.next_pos(1);
            tokens.push(token);
            continue;
        }

        error_at(lexer, lexer.pos, "invalid token");

    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        val : 0,
        s   : "".to_string(),
        loc : lexer.pos,
    });

    return tokens;

}
