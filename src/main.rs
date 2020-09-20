extern crate toy;

use toy::codegen::gen_expr;
use toy::tokenize::{ tokenize, Lexer };
use toy::parse::{ parse, reg };
use toy::CUR;

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
    let node = parse(&tokens);

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // Save callee-saved registers.
    println!("  push r12");
    println!("  push r13");
    println!("  push r14");
    println!("  push r15");

    gen_expr(node);

    unsafe {
        println!("  mov rax, {}", reg(CUR-1));
    }

    println!("  pop r15");
    println!("  pop r14");
    println!("  pop r13");
    println!("  pop r12");
    println!("  ret");
    println!("  ");
}
