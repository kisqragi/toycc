use super::parse::{ Node, NodeKind, Function, LOCALS };
static mut CUR: usize = 0;
static mut LABELSEQ: usize = 1;

fn get_labelseq() -> usize {
    unsafe {
        let labelseq = LABELSEQ;
        LABELSEQ += 1;
        labelseq
    }
}

fn reg(idx: usize) -> String {
    let r = ["rdi", "rsi", "r10", "r11", "r12", "r13", "r14", "r15"];
    if r.len() <= idx {
        panic!("register out of range: {}", idx);
    }

    r[idx].to_string()
}

fn gen_addr(node: Node) {
    if node.kind == NodeKind::Var {
        unsafe {
            println!("  lea {}, [rbp-{}]", reg(CUR), LOCALS[node.var.unwrap()].offset);
            CUR += 1;
        }
        return;
    }

    println!("{:#?}", node);
    panic!("not an lvalue");
}

fn load() {
    unsafe {
        println!("  mov {}, [{}]", reg(CUR-1), reg(CUR-1));
    }
}

fn store() {
    unsafe {
        println!("  mov [{}], {}", reg(CUR-1), reg(CUR-2));
        CUR -= 1;
    }
}

fn gen_expr(node: Node) {
    match node.kind {
        NodeKind::Num => {
            unsafe {
                println!("  mov {}, {}", reg(CUR), node.val);
                CUR += 1;
                return;
            }
        }
        NodeKind::Var => {
            gen_addr(node);
            load();
            return;
        }
        NodeKind::Assign => {
            gen_expr(*node.rhs.unwrap());
            gen_addr(*node.lhs.unwrap());
            store();
            return;
        }
        _ => {}
    }

    gen_expr(*node.lhs.unwrap());
    gen_expr(*node.rhs.unwrap());

    let rd;
    let rs;
    unsafe {
        rd = reg(CUR-2);
        rs = reg(CUR-1);
        CUR -= 1;
    }

    match node.kind {
        NodeKind::Add => {
            println!("  add {}, {}", rd, rs);
        }
        NodeKind::Sub => {
            println!("  sub {}, {}", rd, rs);
        }
        NodeKind::Mul => {
            println!("  imul {}, {}", rd, rs);
        }
        NodeKind::Div => {
            println!("  mov rax, {}", rd);
            println!("  cqo");
            println!("  idiv {}", rs);
            println!("  mov {}, rax", rd);
        }
        NodeKind::Equal => {
            println!("  cmp {}, {}", rd, rs);
            println!("  sete al");
            println!("  movzb {}, al", rd);
        }
        NodeKind::Ne => {
            println!("  cmp {}, {}", rd, rs);
            println!("  setne al");
            println!("  movzb {}, al", rd);
        }
        NodeKind::Lt => {
            println!("  cmp {}, {}", rd, rs);
            println!("  setl al");
            println!("  movzb {}, al", rd);
        }
        NodeKind::Le => {
            println!("  cmp {}, {}", rd, rs);
            println!("  setle al");
            println!("  movzb {}, al", rd);
        }
        _ => panic!("invalid expression")
    }
}

fn gen_stmt(node: Node) {
    match node.kind {
        NodeKind::Return => {
            gen_expr(*node.lhs.unwrap());
            unsafe {
                CUR -= 1;
                println!("  mov rax, {}", reg(CUR));
            }
            println!("  jmp .L.return");
        }
        NodeKind::ExprStmt => {
            gen_expr(*node.lhs.unwrap());
            unsafe {
                CUR -= 1;
            }
        }
        NodeKind::If => {
            unsafe {
                let seq = get_labelseq();            
                if let Some(_) = node.els {
                    gen_expr(*node.cond.unwrap());
                    CUR -= 1;
                    println!("  cmp {}, 0", reg(CUR));
                    println!("  je .L.else.{}", seq);
                    gen_stmt(*node.then.unwrap());
                    println!("  jmp .L.end.{}", seq);
                    println!(".L.else.{}:", seq);
                    gen_stmt(*node.els.unwrap());
                    println!(".L.end.{}:", seq);
                } else {
                    gen_expr(*node.cond.unwrap());
                    CUR -= 1;
                    println!("  cmp {}, 0", reg(CUR));
                    println!("  je .L.end.{}", seq);
                    gen_stmt(*node.then.unwrap());
                    println!(".L.end.{}:", seq);
                }
            }
        }
        _ => panic!("invalid statement")
    }
}

pub fn codegen(prog: Function) {
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // Prologue. r12-r15 are callee-saved registers.
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, {}", prog.stack_size);
    println!("  mov [rsp-8], r12");
    println!("  mov [rsp-16], r13");
    println!("  mov [rsp-24], r14");
    println!("  mov [rsp-32], r15");

    for n in prog.nodes {
        gen_stmt(n);
    }

    // Epilogue
    println!(".L.return:");
    println!("  mov r12, [rsp-8]");
    println!("  mov r13, [rsp-16]");
    println!("  mov r14, [rsp-24]");
    println!("  mov r15, [rsp-32]");
    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
    println!("  ");
}
