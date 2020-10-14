use super::ast::{ Ast, AstKind, BinaryOp };
use super::parse::Program;

static mut CUR: i64 = 0;
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

fn reg(idx: usize) -> String {
    let r = ["r10", "r11", "r12", "r13", "r14", "r15"];
    if r.len() <= idx {
        panic!("register out of range: {}", idx);
    }

    r[idx].to_string()
}
/*
static mut LABELSEQ: usize = 1;

// get_cur(1) => CUR++ (C like)
// get_cur(-1) => CUR-- (C like)


fn get_labelseq() -> usize {
    unsafe {
        let labelseq = LABELSEQ;
        LABELSEQ += 1;
        labelseq
    }
}

fn argreg(idx: usize) -> String {
    let argreg = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
    argreg[idx].to_string()
}



fn gen_addr(node: Ast, f: &Function) {
    match node.kind {
        AstKind::Var{ name,ty, offset } => {
            println!("  lea {}, [rbp-{}]", reg(get_cur(1)), offset);
        }
        AstKind::Deref => {
            gen_expr(*node.lhs.unwrap(), f);
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

fn gen_expr(node: Ast, f: &Function) {
    match node.kind {
        AstKind::Var => {
            gen_addr(node, f);
            load();
            return;
        }
        AstKind::Assign => {
            gen_expr(*node.rhs.unwrap(), f);
            gen_addr(*node.lhs.unwrap(), f);
            store();
            return;
        }
        AstKind::Deref => {
            gen_expr(*node.lhs.unwrap(), f);
            load();
            return;
        }
        AstKind::Addr => {
            gen_addr(*node.lhs.unwrap(), f);
            return;
        }
        AstKind::Funcall => {
            let mut nargs = 0;
            for arg in node.args.unwrap() {
                gen_expr(*arg, f);
                nargs += 1;
            }

            for i in 1..nargs+1 {
                let cur = get_cur(-1);
                println!("  mov {}, {}", argreg(nargs-i), reg(cur-1));
            }


            println!("  push r10");
            println!("  push r11");
//            println!("  mov rax, 0");
            println!("  call {}", node.funcname);
            println!("  pop r11");
            println!("  pop r10");
            println!("  mov {}, rax", reg(get_cur(1)));
            return;
        }
        _ => {}
    }

    gen_expr(*node.lhs.unwrap(), f);
    gen_expr(*node.rhs.unwrap(), f);

    let rd;
    let rs;
    let cur = get_cur(-1);
    rd = reg(cur-2);
    rs = reg(cur-1);

    match node.kind {
       AstKind::Equal => {
            println!("  cmp {}, {}", rd, rs);
            println!("  sete al");
            println!("  movzb {}, al", rd);
        }
        AstKind::Ne => {
            println!("  cmp {}, {}", rd, rs);
            println!("  setne al");
            println!("  movzb {}, al", rd);
        }
        AstKind::Lt => {
            println!("  cmp {}, {}", rd, rs);
            println!("  setl al");
            println!("  movzb {}, al", rd);
        }
        AstKind::Le => {
            println!("  cmp {}, {}", rd, rs);
            println!("  setle al");
            println!("  movzb {}, al", rd);
        }
        _ => panic!("invalid expression")
    }
}

fn gen_stmt(node: Ast, f: &Function) {
    match node.kind {
        AstKind::ExprStmt => {
            gen_expr(*node.lhs.unwrap(), f);
            unsafe {
                CUR -= 1;
            }
        }
        AstKind::If => {
            let seq = get_labelseq();            
            if let Some(_) = node.els {
                gen_expr(*node.cond.unwrap(), f);
                let cur = get_cur(-1);
                println!("  cmp {}, 0", reg(cur-1));
                println!("  je .L.else.{}", seq);
                gen_stmt(*node.then.unwrap(), f);
                println!("  jmp .L.end.{}", seq);
                println!(".L.else.{}:", seq);
                gen_stmt(*node.els.unwrap(), f);
                println!(".L.end.{}:", seq);
            } else {
                gen_expr(*node.cond.unwrap(), f);
                let cur = get_cur(-1);
                println!("  cmp {}, 0", reg(cur-1));
                println!("  je .L.end.{}", seq);
                gen_stmt(*node.then.unwrap(), f);
                println!(".L.end.{}:", seq);
            }
        }
        AstKind::For => {
            let seq = get_labelseq();
            if let Some(_) = node.init {
                gen_stmt(*node.init.unwrap(), f);
            }
            println!(".L.begin.{}:", seq);
            if let Some(_) = node.cond {
                gen_expr(*node.cond.unwrap(), f);
                let cur = get_cur(-1);
                println!("  cmp {}, 0", reg(cur-1));
                println!("  je .L.end.{}", seq);
            }
            gen_stmt(*node.then.unwrap(), f);
            if let Some(_) = node.inc {
                gen_stmt(*node.inc.unwrap(), f);
            }
            println!("  jmp .L.begin.{}", seq);
            println!(".L.end.{}:", seq);
        }
        _ => panic!("invalid statement")
    }
}
*/

pub fn codegen(prog: Program) {
    println!(".intel_syntax noprefix");
    for f in prog.functions {
        f.codegen();
    }
}

impl Ast {
    pub fn codegen(self) {
        match self.kind {
            AstKind::Funcdef { name, body, params, stack_size } => {
                println!(".globl {}", name);
                println!("{}:", name);

                // Prologue. r12-r15 are callee-saved registers.
                println!("  push rbp");
                println!("  mov rbp, rsp");
                println!("  sub rsp, {}", stack_size);
                println!("  mov [rsp-8], r12");
                println!("  mov [rsp-16], r13");
                println!("  mov [rsp-24], r14");
                println!("  mov [rsp-32], r15");

                /*
                // Save arguments to the stack
                let mut i = f.params.len();

                for j in 0..f.params.len() {
                    i -= 1;
                    println!("  mov [rbp-{}], {}", f.locals[j].offset, argreg(i))
                }
                */

                // Emit code
                //gen_stmt(f.node.clone(), &f);
                body.codegen();

                // Epilogue
                println!(".L.return.{}:", name);
                println!("  mov r12, [rsp-8]");
                println!("  mov r13, [rsp-16]");
                println!("  mov r14, [rsp-24]");
                println!("  mov r15, [rsp-32]");
                println!("  mov rsp, rbp");
                println!("  pop rbp");
                println!("  ret");
            },
            AstKind::Num(val) => {
                println!("  mov {}, {}", reg(get_cur(1)), val);
                return;
            },
            AstKind::Return(funcname, ast) => {
                ast.codegen();
                let cur = get_cur(-1);
                println!("  mov rax, {}", reg(cur-1));
                println!("  jmp .L.return.{}", funcname);
            },
            AstKind::Block(asts) => {
                for ast in asts { ast.codegen();
                }
            },
            AstKind::BinaryOp(op, lhs, rhs) => {
                Self::codegen_binaryop(op, lhs, rhs);
            },
            _ => panic!()
        }
    }

    pub fn codegen_binaryop(op: BinaryOp, lhs: Box<Ast>, rhs: Box<Ast>) {
        lhs.codegen();
        rhs.codegen();

        let cur = get_cur(-1);
        let rd = reg(cur-2);
        let rs = reg(cur-1);
        match op {
            BinaryOp::Add => {
                println!("  add {}, {}", rd, rs);
            }
            BinaryOp::Sub => {
                println!("  sub {}, {}", rd, rs);
            }
            BinaryOp::Mul => {
                println!("  imul {}, {}", rd, rs);
            }
            BinaryOp::Div => {
                println!("  mov rax, {}", rd);
                println!("  cqo");
                println!("  idiv {}", rs);
                println!("  mov {}, rax", rd);
            }
            _ => panic!()
        }
    }
}

