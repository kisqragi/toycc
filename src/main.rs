extern crate toy;

use toy::codegen::codegen;
use toy::tokenize::{ tokenize, Lexer };
use toy::parse::parse;

use std::env;
use std::process;
use std::iter::Iterator;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("{}: invalid number of arguments", args[0]);
        process::exit(1);
    }

    let mut lexer = Lexer::new(&args[1]);
    let tokens = tokenize(&mut lexer);
    let nodes = parse(&tokens);
    codegen(nodes);
}
