use super::parse::{ Node, NodeKind, reg };
use super::CUR;

fn gen_expr(node: Node) {
    if node.kind == NodeKind::Num {
        unsafe {
            println!("  mov {}, {}", reg(CUR), node.val);
            CUR += 1;
            return;
        }
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

    // Save callee-saved registers.
    println!("  push r12");
    println!("  push r13");
    println!("  push r14");
    println!("  push r15");

    for n in nodes {
        gen_stmt(n);
    }

    println!(".L.return:");
    println!("  pop r15");
    println!("  pop r14");
    println!("  pop r13");
    println!("  pop r12");
    println!("  ret");
    println!("  ");
}
