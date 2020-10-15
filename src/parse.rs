use super::ast::{ Ast, AstKind, UnaryOp, BinaryOp };
use super::tokenize::{ Token, TokenKind, Keyword, Symbol };
//use super::types::{ Type, add_type, is_integer, ty_int, pointer_to, func_type, copy_type };
use super::types::{ Type, ty_int, pointer_to, func_type };
use std::process::exit;

fn new_binary(op: BinaryOp, lhs: Ast, rhs: Ast) -> Ast {
    Ast::new(AstKind::BinaryOp(op, Box::new(lhs), Box::new(rhs)))
}

fn new_unary(op: UnaryOp, expr: Ast) -> Ast {
    Ast::new(AstKind::UnaryOp(op, Box::new(expr)))
}

fn new_num(pc: &mut ParseContext) -> Ast {
    if pc.tokens[pc.pos].kind.is_num() {
        let val = pc.tokens[pc.pos].get_num();
        return Ast { kind: AstKind::Num(val) };
    }
    panic!("number expected, but got {}", pc.tokens[pc.pos].get_string());
}


/*
fn find_var(pc: &mut ParseContext) -> Option<usize> {
    for (i, var) in pc.locals.iter().enumerate() {
        if pc.tokens[pc.pos].get_string() == var.name {
            return Some(i);
        }
    }
    None 
}


fn new_var_ast(ty: Type, name: String) -> Ast {
    Ast::new(AstKind::Var{ ty, name, offset:0 })
}

fn new_lvar_params(pc: &mut ParseContext, t: Type) {
    let ty = t.clone();
    let name = t.name.unwrap().get_string();

    let v = Var {
        name,
        ty,
        offset: 0,
    };
    pc.locals.push(v);
}

fn new_lvar(pc: &mut ParseContext, ty: Type) -> usize {
    let v = Var {
        name: pc.tokens[pc.pos].get_string(),
        ty,
        offset: 0,
    };
    pc.locals.push(v);
    return pc.locals.len()-1;
}
*/


// stmt = "return" expr ";"
//      | "{" compound-stmt
//      | "if" "(" expr ")" stmt ("else" stmt)?
//      | "for" "(" expr? ";" expr? ";" expr? ")" stmt
//      | "while" "(" expr ")" stmt
//      | expr-stmt
fn stmt(pc: &mut ParseContext) -> Ast {
    match pc.tokens[pc.pos].kind {
        // "return" statement
        TokenKind::Keyword(Keyword::Return) => {
            pc.pos += 1;
            let expr = expr(pc);
            let ast = Ast::new(AstKind::Return(pc.curr_funcname.clone(), Box::new(expr)));
            skip(pc, TokenKind::Symbol(Symbol::Semicolon));
            return ast;
        }
        // "if" statement
        TokenKind::Keyword(Keyword::If) => {

            pc.pos += 1;
            skip(pc, TokenKind::Symbol(Symbol::OpeningParen));

            // set cond
            let cond = expr(pc);

            skip(pc, TokenKind::Symbol(Symbol::ClosingParen));

            // set then
            let then = stmt(pc);

            // "else"
            let mut els = None;
            if pc.tokens[pc.pos].kind == TokenKind::Keyword(Keyword::Else) {
                pc.pos += 1;
                els = Some(Box::new(stmt(pc)));
            }

            return Ast::new(
                AstKind::If {
                    cond: Box::new(cond),
                    then: Box::new(then),
                    els
                }
            );
        }
        // "for" statement
        TokenKind::Keyword(Keyword::For) => {
            pc.pos += 1;
            skip(pc, TokenKind::Symbol(Symbol::OpeningParen));

            // initとincは値を返さない
            // init
            let mut init = None;
            if pc.tokens[pc.pos].kind != TokenKind::Symbol(Symbol::Semicolon) {
                init = Some(Box::new(new_unary(UnaryOp::ExprStmt, expr(pc))));
            }
            skip(pc, TokenKind::Symbol(Symbol::Semicolon));

            // cond
            let mut cond = None;
            if pc.tokens[pc.pos].kind != TokenKind::Symbol(Symbol::Semicolon) {
                cond = Some(Box::new(new_unary(UnaryOp::ExprStmt, expr(pc))));
            }
            skip(pc, TokenKind::Symbol(Symbol::Semicolon));

            let mut inc = None;
            if pc.tokens[pc.pos].kind != TokenKind::Symbol(Symbol::ClosingParen) {
                inc = Some(Box::new(new_unary(UnaryOp::ExprStmt, expr(pc))));
            }
            skip(pc, TokenKind::Symbol(Symbol::ClosingParen));

            let then = Some(Box::new(stmt(pc)));

            return Ast::new(AstKind::For { init, cond, inc, then });
        }
        // "while" statement
        TokenKind::Keyword(Keyword::While) => {
            pc.pos += 1;
            skip(pc, TokenKind::Symbol(Symbol::OpeningParen));

            let cond = Some(Box::new(expr(pc)));
            skip(pc, TokenKind::Symbol(Symbol::ClosingParen));

            let then = Some(Box::new(stmt(pc)));

            return Ast::new(AstKind::For{
                init: None,
                inc: None,
                cond,
                then
            });
        }
        // "{...}" compound statement
        TokenKind::Symbol(Symbol::OpeningBrace) => {
            pc.pos += 1;
            let body = compound_stmt(pc);
            return body;
        }
        _ => expr_stmt(pc)
    }
}

// compound-stmt = (declaration | stmt)* "}"
fn compound_stmt(pc: &mut ParseContext) -> Ast {
    let mut body: Vec<Box<Ast>> = vec![];
    while pc.tokens[pc.pos].kind != TokenKind::Symbol(Symbol::ClosingBrace) {
        if pc.tokens[pc.pos].kind == TokenKind::Keyword(Keyword::Int) {
            let ast = declaration(pc);
            //body.push(Box::new(add_type(&mut Ast)));
            body.push(Box::new(ast));
        } else {
            let ast = stmt(pc);
            body.push(Box::new(ast));
            //body.push(Box::new(add_type(&mut Ast)));
        }
    }

    pc.pos += 1;
    return Ast::new(AstKind::Block(body));
}

// funcdef = typespec declarator "{" compound-stmt
fn funcdef(pc: &mut ParseContext) -> Ast {
    //pc.locals = Vec::new();
    let ty = typespec(pc);
    let ty = declarator(pc, ty);

    skip(pc, TokenKind::Symbol(Symbol::OpeningBrace));

    pc.curr_funcname  = ty.name.clone();
    /*
    for t in ty.get_params() {
        new_lvar_params(pc, t);
    }
    */
//    let params = pc.locals.clone();

    let ast = compound_stmt(pc);
//    let locals = pc.locals.clone();
    Ast::new(AstKind::Funcdef {
        name: ty.name.clone(),
        body: Box::new(ast),
        params: ty.get_params(),
        stack_size: 32,
    })
}

// declaration = typespec (declarator ("=" expr)? ("," declarator ("=" expr)?)*)? ";"
fn declaration(pc: &mut ParseContext) -> Ast {
    let basety = typespec(pc);

    let mut body: Vec<Box<Ast>> = vec![];
    let mut i = 0;
    /*
    loop {
        if pc.tokens[pc.pos].kind == TokenKind::Symbol(Symbol::Semicolon) {
            skip(pc, TokenKind::Symbol(Symbol::Semicolon));
            break;
        }

        if i > 0 {
            skip(pc, TokenKind::Symbol(Symbol::Comma));
        }
        i += 1;

        let ty = declarator(pc, basety.clone());
        pc.pos -= 1;
        let var = new_lvar(pc, ty);

        pc.pos += 1;
        if pc.tokens[pc.pos].kind != TokenKind::Symbol(Symbol::Assign) { continue; }

        let lhs = new_var_Ast(ty, var);
        pc.pos += 1;
        let rhs = assign(pc);
        let Ast = new_binary(BinaryOp::Assign, lhs, rhs);
        body.push(Box::new(new_unary(UnaryOp::ExprStmt, Ast)));
    }
    */

    return Ast::new(AstKind::Block(body));
}

// typespec = "int"
fn typespec(pc: &mut ParseContext) -> Type {
    skip(pc, TokenKind::Keyword(Keyword::Int));
    ty_int()
}

// declarator = "*"* ident type-suffix
fn declarator(pc: &mut ParseContext, mut ty: Type) -> Type {
    loop {
        if !consume(pc, TokenKind::Symbol(Symbol::Asterisk)) { break; }
        ty = pointer_to(ty);
    }

    if !pc.tokens[pc.pos].kind.is_identifier() {
        eprintln!("expected a variable name: {}", pc.tokens[pc.pos].get_string());
    }

    let pos = pc.pos;
    let name = pc.tokens[pos].get_string();
    pc.pos += 1;
    let mut ty = type_suffix(pc, ty);
    ty.name = name;
    ty
}

// type-suffix = ( "(" func-params? ")" )
// func-params = param ("," param)*
// param       = typespec declarator
fn type_suffix(pc: &mut ParseContext, mut ty: Type) -> Type {
    if pc.tokens[pc.pos].kind == TokenKind::Symbol(Symbol::OpeningParen) {
        pc.pos += 1;

        let mut params: Vec<Type> = vec![];

        while pc.tokens[pc.pos].kind != TokenKind::Symbol(Symbol::ClosingParen) {
            if params.len() != 0 {
                skip(pc, TokenKind::Symbol(Symbol::Comma));
            }
            let basety = typespec(pc);
            params.push(declarator(pc, basety));
        }

        ty = func_type(ty, params);

        skip(pc, TokenKind::Symbol(Symbol::ClosingParen));
        return ty;
    }
    ty
}

// expr-stmt = expr ";"
fn expr_stmt(pc: &mut ParseContext) -> Ast {
    let Ast = new_unary(UnaryOp::ExprStmt, expr(pc));
    skip(pc, TokenKind::Symbol(Symbol::Semicolon));
    Ast
}

// expr =  assign
fn expr(pc: &mut ParseContext) -> Ast {
    assign(pc)
}

// assign = equality ("=" assign)?
fn assign(pc: &mut ParseContext) -> Ast {
    let mut Ast = equality(pc);
    if pc.tokens[pc.pos].kind == TokenKind::Symbol(Symbol::Assign) {
        pc.pos += 1;
        Ast = new_binary(BinaryOp::Assign, Ast, assign(pc));  
    }

    Ast
}

// equality = relational ("==" relational | "!=" relational)*
fn equality(pc: &mut ParseContext) -> Ast {
    let mut Ast = relational(pc);

    loop {
        match pc.tokens[pc.pos].kind  {
            TokenKind::Symbol(Symbol::Eq) => {
                pc.pos += 1;
                let rhs = relational(pc);
                Ast = new_binary(BinaryOp::Eq, Ast, rhs);
                continue;
            }
            TokenKind::Symbol(Symbol::Ne) => {
                pc.pos += 1;
                let rhs = relational(pc);
                Ast = new_binary(BinaryOp::Ne, Ast, rhs);
                continue;
            }
            _ => return Ast
        }
    }

}

// relational = add ("<" add | "<=" add | ">" add | ">=" add)*
fn relational(pc: &mut ParseContext) -> Ast {
    let mut Ast = add(pc);

    loop {
        match pc.tokens[pc.pos].kind {
            TokenKind::Symbol(Symbol::Lt) => {
                pc.pos += 1;
                Ast = new_binary(BinaryOp::Lt, Ast, add(pc));
                continue;
            }
            TokenKind::Symbol(Symbol::Le) => {
                pc.pos += 1;
                Ast = new_binary(BinaryOp::Le, Ast, add(pc));
                continue;
            }
            TokenKind::Symbol(Symbol::Gt) => {
                pc.pos += 1;
                Ast = new_binary(BinaryOp::Lt, add(pc), Ast);
                continue;
            }
            TokenKind::Symbol(Symbol::Ge) => {
                pc.pos += 1;
                Ast = new_binary(BinaryOp::Le, add(pc), Ast);
                continue;
            }
            _ => return Ast
        }
    }
}

fn new_add(mut lhs: Ast, mut rhs: Ast) -> Ast {
    //lhs = add_type(&mut lhs);
    //rhs = add_type(&mut rhs);

    // num + num
    if !lhs.is_pointer() && !rhs.is_pointer() {
        return new_binary(BinaryOp::Add, lhs, rhs);
    }

    if lhs.is_pointer() && rhs.is_pointer() {
        eprintln!("invalid operands");
        eprintln!("lhs = {:#?}", lhs);
        eprintln!("rhs = {:#?}", rhs);
    }

    // Canonicalize `num + ptr` to `ptr + num`.
    if !lhs.is_pointer() && rhs.is_pointer() {
        let tmp = lhs;
        lhs = rhs;
        rhs = tmp;
    }

    // ptr + num
    rhs = new_binary(BinaryOp::Mul, rhs, Ast::new(AstKind::Num(8)));
    new_binary(BinaryOp::Add, lhs, rhs)
}

fn new_sub(mut lhs: Ast, mut rhs: Ast) -> Ast {
    //lhs = add_type(&mut lhs);
    //rhs = add_type(&mut rhs);

    // num - num
    if !lhs.is_pointer() && !rhs.is_pointer() {
        return new_binary(BinaryOp::Sub, lhs, rhs);
    }

    // ptr - num
    if lhs.is_pointer() && !rhs.is_pointer() {
        rhs = new_binary(BinaryOp::Mul, rhs, Ast::new(AstKind::Num(8)));
        return new_binary(BinaryOp::Sub, lhs, rhs)
    }

    // num - ptr (error)
    if !lhs.is_pointer() && rhs.is_pointer() {
        eprintln!("invalid operands");
        eprintln!("lhs = {:#?}", lhs);
        eprintln!("rhs = {:#?}", rhs);
    }

    // `ptr-ptr` returns the result of `ptr-ptr` divided by its size.
    // The result is a number of elements, but the value can also be negative.
    lhs = new_binary(BinaryOp::Sub, lhs, rhs);
    new_binary(BinaryOp::Div, lhs, Ast::new(AstKind::Num(8)))
}

// add = mul ("+" mul | "-" mul)*
fn add(pc: &mut ParseContext) -> Ast {
    let mut ast = mul(pc);

    loop {
        match pc.tokens[pc.pos].kind {
            TokenKind::Symbol(Symbol::Add) => {
                pc.pos += 1;
                let rhs = mul(pc);
                ast = new_add(ast, rhs);
                continue;
            }
            TokenKind::Symbol(Symbol::Sub) => {
                pc.pos += 1;
                let rhs = mul(pc);
                ast = new_sub(ast, rhs);
                continue;
            }
            _ => return ast
        }
    }
}

// mul = unary ("*" unary | "/" unary)*
fn mul(pc: &mut ParseContext) -> Ast {
    let mut ast = unary(pc);

    loop {
        match pc.tokens[pc.pos].kind {
            TokenKind::Symbol(Symbol::Asterisk) => {
                pc.pos += 1;
                ast = new_binary(BinaryOp::Mul, ast, unary(pc));
                continue;
            }
            TokenKind::Symbol(Symbol::Div) => {
                pc.pos += 1;
                ast = new_binary(BinaryOp::Div, ast, unary(pc));
                continue;
            }
            _ => return ast
        }

    }
}

// unary = ("+" | "-" | "&" | "*")? unary
//       | primary
fn unary(pc: &mut ParseContext) -> Ast {
    match pc.tokens[pc.pos].kind {
        TokenKind::Symbol(Symbol::Add) => {
            pc.pos += 1;
            return unary(pc);
        }
        TokenKind::Symbol(Symbol::Sub) => {
            pc.pos += 1;
            return new_binary(BinaryOp::Sub, Ast::new(AstKind::Num(0)), unary(pc));
        }
        TokenKind::Symbol(Symbol::Ampersand) => {
            pc.pos += 1;
            return new_unary(UnaryOp::Addr, unary(pc));
        }
        TokenKind::Symbol(Symbol::Asterisk) => {
            pc.pos += 1;
            return new_unary(UnaryOp::Deref, unary(pc));
        }
        _ => return primary(pc)
    };
}

// primary   = "(" expr ")" | ident func-args? | num
fn primary(pc: &mut ParseContext) -> Ast {
    if pc.tokens[pc.pos].kind == TokenKind::Symbol(Symbol::OpeningParen) {
        pc.pos += 1;
        let Ast = expr(pc);
        skip(pc, TokenKind::Symbol(Symbol::ClosingParen));
        return Ast;
    }

    if pc.tokens[pc.pos].kind.is_identifier() {
        // Function call
        if pc.tokens[pc.pos+1].kind == TokenKind::Symbol(Symbol::OpeningParen) {
            return funcall(pc);
        }

        /*
        // Variable
        let var = find_var(pc);
        if var == None {
            eprintln!("undefined variable: {}", pc.tokens[pc.pos].get_string());
            exit(1);
        }
        pc.pos += 1;
        return new_var_Ast(var.unwrap());
        */
    }

    let Ast = new_num(pc);
    pc.pos += 1;
    Ast
}

// func-args = "(" (assign ("," assign)*)? ")"
fn funcall(pc: &mut ParseContext) -> Ast {
    let start = pc.pos;
    pc.pos += 2;   // eat ident & "("

    let mut args: Vec<Box<Ast>> = vec![];

    while pc.tokens[pc.pos].kind != TokenKind::Symbol(Symbol::ClosingParen) {
        if (pc.pos-2) != start {
            skip(pc, TokenKind::Symbol(Symbol::Comma));
        }
        let mut Ast = assign(pc);
        //args.push(Box::new(add_type(&mut Ast)));
        args.push(Box::new(Ast));
    }

    skip(pc, TokenKind::Symbol(Symbol::ClosingParen));

    Ast::new(
        AstKind::Funcall {
            name: pc.tokens[start].get_string(),
            args,
        }
    )
}

fn skip(pc: &mut ParseContext, t: TokenKind){
    if pc.tokens[pc.pos].kind != t {
        panic!("expected '{}'", pc.tokens[pc.pos].get_string());
    }
    pc.pos += 1
}

// トークンが期待するトークンの場合、トークンを一つ消費して
// 真を返す。違う場合偽を返す。
fn consume(pc: &mut ParseContext, t: TokenKind) -> bool {
    if pc.tokens[pc.pos].kind == t {
        pc.pos += 1;
        return true;
    }
    false
}

#[derive(Debug, Default)]
pub struct Program {
    pub functions: Vec<Ast>,
}

#[derive(Debug, Default)]
struct ParseContext {
    tokens: Vec<Token>,
    pos: usize,
    curr_funcname: String,
    //locals: Vec<Var>,
}

// program = funcdef*
pub fn parse(tokens: Vec<Token>) -> Program {
    let mut prog = Program { ..Default::default() };
    let mut pc = ParseContext { tokens, ..Default::default() };
    while pc.tokens[pc.pos].kind != TokenKind::Eof {
        let func = funcdef(&mut pc);
        prog.functions.push(func);
    }
    prog
}
