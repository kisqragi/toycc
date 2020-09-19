extern crate toy;
use toy::strtol;

use std::env;
use std::process;
use std::iter::Iterator;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("{}: invalid number of arguments", args[0]);
        process::exit(1);
    }

    let p = &args[1];


    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let (n, mut p) = strtol(&p);
    println!("  mov rax, {}", n.unwrap());

    while let Some(c) = p.chars().nth(0) {
        let s = p.split_off(1);

        if c == '+' {
            let (n, remaining) = strtol(&s);
            p = remaining;
            println!("  add rax, {}", n.unwrap());
            continue;
        }

        if c == '-' {
            let (n, remaining) = strtol(&s);
            p = remaining;
            println!("  sub rax, {}", n.unwrap());
            continue;
        }

        eprintln!("unexpected charater: {}", p);
        return;
    }


    println!("  ret");
    println!("  ");
    
}
