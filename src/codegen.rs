use super::parse::{ Node, NodeKind, reg };
use super::CUR;

pub fn gen_expr(node: Node) {
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

