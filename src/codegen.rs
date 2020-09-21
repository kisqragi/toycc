use super::parse::{ Node, NodeKind };
static mut CUR: usize = 0;

fn reg(idx: usize) -> String {
    let r = ["rdi", "rsi", "r10", "r11", "r12", "r13", "r14", "r15"];
    if r.len() <= idx {
        panic!("register out of range: {}", idx);
    }

    r[idx].to_string()
}

fn gen_addr(node: Node) {
    if node.kind == NodeKind::Var {
        let mut offset = ((node.name as u32) - ('a' as u32) + 1) * 8;
        offset += 32;   // for callee-saved registers
        unsafe {
            println!("  lea {}, [rbp-{}]", reg(CUR), offset);
            CUR += 1;
        }
        return;
    }

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
        _ => panic!("invalid statement")
    }
}

pub fn codegen(nodes: Vec<Node>) {
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // Prologue. r12-r15 are callee-saved registers.
    println!("  push rbp");
    println!("  mov rbp, rsp");
    // 240 = ('a'~'z')*8 + 32
    println!("  sub rsp, 240");
    println!("  mov [rsp-8], r12");
    println!("  mov [rsp-16], r13");
    println!("  mov [rsp-24], r14");
    println!("  mov [rsp-32], r15");

    for n in nodes {
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
