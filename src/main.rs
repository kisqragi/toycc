extern crate toy;

use toy::codegen::codegen;
use toy::tokenize::{ tokenize, Lexer };
use toy::parse::parse;

use std::env;
use std::process;
use std::iter::Iterator;

fn align_to(n: usize, align: usize) -> usize {
    (n + align - 1) / align * align
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("{}: invalid number of arguments", args[0]);
        process::exit(1);
    }

    let mut lexer = Lexer::new(&args[1]);
    let tokens = tokenize(&mut lexer);
    let mut prog = parse(&tokens);

    for i in 0..prog.functions.len() {
        let mut offset = 32;
        for j in 0..prog.functions[i].locals.len() {
            offset += 8;
            prog.functions[i].locals[j].offset = offset;
        }
        prog.functions[i].stack_size = align_to(offset, 16);
    }

    codegen(prog);
}
