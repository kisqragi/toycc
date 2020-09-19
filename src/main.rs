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

    fn getc(&self) -> Option<&char> {
        self.code.get(self.pos)
    }

    fn next_pos(&mut self) {
        self.pos += 1;
    }

    fn is_last(&self) -> bool {
        self.code.len() == self.pos
    }

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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("{}: invalid number of arguments", args[0]);
        process::exit(1);
    }

    let mut lexer = Lexer::new(&args[1]);


    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    println!("  mov rax, {}", lexer.strtol().unwrap());

    while !lexer.is_last() {
        let c = lexer.getc().unwrap();

        if c == &'+' {
            lexer.next_pos();
            println!("  add rax, {}", lexer.strtol().unwrap());
            continue;
        }

        if c == &'-' {
            lexer.next_pos();
            println!("  sub rax, {}", lexer.strtol().unwrap());
            continue;
        }

        eprintln!("unexpected charater: {}", c);
        return;
    }

    println!("  ret");
    println!("  ");
    
}
