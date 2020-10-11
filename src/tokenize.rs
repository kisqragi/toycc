use std::process;

#[derive(Debug)]
pub struct Lexer {
    code: Vec<char>,
    pos: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,    // Kind of Token
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Keyword(Keyword), // Keyword
    Symbol(Symbol),     // Symbol
    Ident(String),      // Identifiers
    Num(i64),           // Numeric literal
    Eof,                // End-of-file markers
}

#[derive(Debug, PartialEq, Clone)]
pub enum Symbol {
    Add,            // +
    Sub,            // -
    Div,            // /
    Eq,             // ==
    Ne,             // !=
    Lt,             // <
    Le,             // <=
    Gt,             // >
    Ge,             // >=
    Assign,         // =
    Ampersand,      // &
    Asterisk,       // *
    Comma,          // ,
    Semicolon,      // ;
    OpeningParen,   // (
    ClosingParen,   // )
    OpeningBrace,   // {
    ClosingBrace,   // }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Int,        // "int"
    If,         // "if"
    Else,       // "else"
    For,        // "for"
    While,      // "while"
    Return,     // "return"
}

macro_rules! retrieve_ident {
    ($e:expr) => {
        match &$e.kind {
            &TokenKind::Ident(ref ident) => ident.to_string(),
            _ => "".to_string(),
        }
    };
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

impl Token {
    fn new(kind: TokenKind) -> Self {
        Self { kind }
    }

    fn convert_symbol(self) -> Self {
        let ident = retrieve_ident!(self);
        let kind = match ident.as_str() {
            "+"  => TokenKind::Symbol(Symbol::Add),
            "-"  => TokenKind::Symbol(Symbol::Sub),
            "*"  => TokenKind::Symbol(Symbol::Asterisk),
            "/"  => TokenKind::Symbol(Symbol::Div),
            "==" => TokenKind::Symbol(Symbol::Eq),
            "!=" => TokenKind::Symbol(Symbol::Ne),
            "<"  => TokenKind::Symbol(Symbol::Lt),
            "<=" => TokenKind::Symbol(Symbol::Le),
            ">"  => TokenKind::Symbol(Symbol::Gt),
            ">=" => TokenKind::Symbol(Symbol::Ge),
            "="  => TokenKind::Symbol(Symbol::Assign),
            "&"  => TokenKind::Symbol(Symbol::Ampersand),
            ","  => TokenKind::Symbol(Symbol::Comma),
            ";"  => TokenKind::Symbol(Symbol::Semicolon),
            "("  => TokenKind::Symbol(Symbol::OpeningParen),
            ")"  => TokenKind::Symbol(Symbol::ClosingParen),
            "{"  => TokenKind::Symbol(Symbol::OpeningBrace),
            "}"  => TokenKind::Symbol(Symbol::ClosingBrace),
            _    => return self 
        };
        Token::new(kind)
    }

    fn convert_keyword(self) -> Self {
        let ident = retrieve_ident!(self);
        let kind = match ident.as_str() {
            "int"    => TokenKind::Keyword(Keyword::Int),
            "if"     => TokenKind::Keyword(Keyword::If),
            "else"   => TokenKind::Keyword(Keyword::Else),
            "for"    => TokenKind::Keyword(Keyword::For),
            "while"  => TokenKind::Keyword(Keyword::While),
            "return" => TokenKind::Keyword(Keyword::Return),
                   _ => return self 
        };
        Token::new(kind)
    }

    fn convert_reserved(self) -> Self {
        let mut token = self.convert_symbol();
        token = token.convert_keyword();
        token
    }

    pub fn get_string(&self) -> String {
        match &self.kind {
            TokenKind::Ident(s) => s.clone(),
            TokenKind::Num(n) => n.to_string(),
            _ => format!("error:{:#?}", self),
        }
    }

    pub fn get_num(&self) -> i64 {
        match self.kind {
            TokenKind::Num(n) => n,
            _ => panic!()
        }
    }

}

impl TokenKind {
    pub fn is_identifier(&self) -> bool {
        matches!(self, TokenKind::Ident(_))
    }

    pub fn is_keyword(&self) -> bool {
        matches!(self, TokenKind::Keyword(_))
    }

    pub fn is_num(&self) -> bool {
        matches!(self, TokenKind::Num(_))
    }

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

fn starts_with_reserved(vc: &[char]) -> Option<String> {
    // Keyword
    let kw = ["return", "if", "else", "for", "while", "int"];

    for i in 0..kw.len() {
        let len = kw[i].len();
        if startswith(vc, kw[i]) && !is_alnum(&vc[len]) {
            return Some(kw[i].to_string());
        }
    }

    // Multi-letter punctuators
    let ops = ["==", "!=", "<=", ">="];
    for i in 0..ops.len() {
        if startswith(vc, ops[i]) {
            return Some(ops[i].to_string());
        }
    }

    None
}

pub fn error(s: String) {
    eprintln!("{}", s);
    process::exit(1);
}

impl Lexer {
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !self.is_last() {
            let mut c = self.getc();
            // Skip whitespace characters.
            if c.is_whitespace() {
                self.next_pos(1);
                continue;
            }

            // Numeric literal
            if c.is_ascii_digit() {
                let val = self.strtol();
                let token = Token::new(TokenKind::Num(val));
                tokens.push(token);
                continue;
            }

            // Keywords or Multi-letter punctuators
            if let Some(s) = starts_with_reserved(&self.code[self.pos..]) {
                let len = s.len();
                let token = Token::new(TokenKind::Ident(s));
                self.next_pos(len);
                tokens.push(token);
                continue;
            }

            // Identifier
            if is_alpha(c) {
                let mut s = String::new();
                while is_alnum(c) {
                    s.push_str(&c.to_string());
                    self.next_pos(1);
                    c = self.getc();
                }

                let token = Token::new(TokenKind::Ident(s));

                tokens.push(token);
                continue;
            }

            // Punctuator
            if ispunct(c) {
                let token = Token::new(TokenKind::Ident(c.to_string()));
                self.next_pos(1);
                tokens.push(token);
                continue;
            }

            error(format!("invalid token: {}", c));
        }

        tokens.push(Token::new(TokenKind::Eof));

        let mut tokens2 = Vec::new();
        for t in tokens {
            tokens2.push(t.convert_reserved());
        }

        return tokens2;
    }
}
