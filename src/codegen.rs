use super::parse::{ Node, NodeKind, Function, LOCALS };
static mut CUR: i64 = 0;
static mut LABELSEQ: usize = 1;

// get_cur(1) => CUR++ (C like)
// get_cur(-1) => CUR-- (C like)
fn get_cur(n: i64) -> usize {
    let t;
    unsafe {
        t = CUR;
        if CUR + n < 0 {
            panic!("CUR is less than zero: {}", CUR+n);
        }
        CUR += n
    }
    t as usize
}

fn get_labelseq() -> usize {
    unsafe {
        let labelseq = LABELSEQ;
        LABELSEQ += 1;
        labelseq
    }
}

fn reg(idx: usize) -> String {
    let r = ["r10", "r11", "r12", "r13", "r14", "r15"];
    if r.len() <= idx {
        panic!("register out of range: {}", idx);
    }

    r[idx].to_string()
}

fn gen_addr(node: Node) {
    match node.kind {
        NodeKind::Var => {
            println!("  lea {}, [rbp-{}]", reg(get_cur(1)), unsafe { LOCALS[node.var.unwrap()].offset });
        }
        NodeKind::Deref => {
            gen_expr(*node.lhs.unwrap());
        }
        _ => {
            println!("{:#?}", node);
            panic!("not an lvalue");
        }
    }
}

fn load() {
    let cur = get_cur(0)-1;
    println!("  mov {}, [{}]", reg(cur), reg(cur));
}

fn store() {
    let cur = get_cur(-1);
    println!("  mov [{}], {}", reg(cur-1), reg(cur-2));
}

fn gen_expr(node: Node) {
    match node.kind {
        NodeKind::Num => {
            println!("  mov {}, {}", reg(get_cur(1)), node.val);
            return;
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
        NodeKind::Deref => {
            gen_expr(*node.lhs.unwrap());
            load();
            return;
        }
        NodeKind::Addr => {
            gen_addr(*node.lhs.unwrap());
            return;
        }
        _ => {}
    }

    gen_expr(*node.lhs.unwrap());
    gen_expr(*node.rhs.unwrap());

    let rd;
    let rs;
    let cur = get_cur(-1);
    rd = reg(cur-2);
    rs = reg(cur-1);

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
            let cur = get_cur(-1);
            println!("  mov rax, {}", reg(cur-1));
            println!("  jmp .L.return");
        }
        NodeKind::ExprStmt => {
            gen_expr(*node.lhs.unwrap());
            unsafe {
                CUR -= 1;
            }
        }
        NodeKind::If => {
            let seq = get_labelseq();            
            if let Some(_) = node.els {
                gen_expr(*node.cond.unwrap());
                let cur = get_cur(-1);
                println!("  cmp {}, 0", reg(cur-1));
                println!("  je .L.else.{}", seq);
                gen_stmt(*node.then.unwrap());
                println!("  jmp .L.end.{}", seq);
                println!(".L.else.{}:", seq);
                gen_stmt(*node.els.unwrap());
                println!(".L.end.{}:", seq);
            } else {
                gen_expr(*node.cond.unwrap());
                let cur = get_cur(-1);
                println!("  cmp {}, 0", reg(cur-1));
                println!("  je .L.end.{}", seq);
                gen_stmt(*node.then.unwrap());
                println!(".L.end.{}:", seq);
            }
        }
        NodeKind::For => {
            let seq = get_labelseq();
            if let Some(_) = node.init {
                gen_stmt(*node.init.unwrap());
            }
            println!(".L.begin.{}:", seq);
            if let Some(_) = node.cond {
                gen_expr(*node.cond.unwrap());
                let cur = get_cur(-1);
                println!("  cmp {}, 0", reg(cur-1));
                println!("  je .L.end.{}", seq);
            }
            gen_stmt(*node.then.unwrap());
            if let Some(_) = node.inc {
                gen_stmt(*node.inc.unwrap());
            }
            println!("  jmp .L.begin.{}", seq);
            println!(".L.end.{}:", seq);
        }
        NodeKind::Block => {
            for n in node.body.unwrap() {
                gen_stmt(*n);
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

    gen_stmt(prog.body);

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
