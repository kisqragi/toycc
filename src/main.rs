extern crate toycc;

use toycc::codegen::codegen;
use toycc::tokenize::Lexer;
use toycc::parse::parse;

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

    let lexer = Lexer::new(&args[1]);
    let tokens = lexer.tokenize();
    let mut prog = parse(tokens);

    for i in 0..prog.functions.len() {
        let mut offset = 32;
        for l in &mut prog.functions[i].locals {
            offset += 8;
            l.offset = offset;
        }
        prog.functions[i].stack_size = align_to(offset, 16);
    }

    codegen(prog);
}
