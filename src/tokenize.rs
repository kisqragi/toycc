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
    fn getc(&self) -> Option<&char> {
        self.code.get(self.pos)
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
pub enum TokenKind {
    Reserved,
    Num,
    Eof,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,    // Kind of Token
    pub val : i64,          // Number literal
    pub s   : String,       // String of Token
    pub loc : usize,
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

fn ispunct(c: &char) -> bool {
    let mut punct = ['!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-',
                 '.', '/', ':', ';', '<', '=', '>', '?', '@', '[', '\\', ']', '^',
                 '_', '`', '{', '|', '}'].iter();
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
        let c = lexer.getc().unwrap();
        // Skip whitespace characters.
        if c.is_whitespace() {
            lexer.next_pos(1);
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

        // Multi-letter punctuators
        // ==, !=, <= and >=
        if (c == &'=' || c == &'!' || c == &'>' || c == &'<') && lexer.code[lexer.pos+1] == '=' {
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
