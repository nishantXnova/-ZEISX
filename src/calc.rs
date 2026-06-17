use std::io::{self, Write};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op { Add, Sub, Mul, Div, Num, Neg }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Token { Num(i64), Op(Op), LParen, RParen, Eof }

const PREC_MAP: [u8; 7] = [0, 1, 1, 1, 2, 2, 3];

#[inline(always)]
fn idx(t: &Token) -> usize {
    match t {
        Token::Eof => 0,
        Token::Num(_) => 1,
        Token::Op(Op::Add) => 2,
        Token::Op(Op::Sub) => 3,
        Token::Op(Op::Mul) => 4,
        Token::Op(Op::Div) => 5,
        Token::Op(Op::Neg) => 6,
        _ => 0,
    }
}

#[inline(always)]
fn prec(t: &Token) -> u8 { PREC_MAP[idx(t)] }

impl Token {
    #[inline]
    pub fn apply(self, a: i64, b: i64) -> i64 {
        match self {
            Token::Op(Op::Add) => a + b,
            Token::Op(Op::Sub) => a - b,
            Token::Op(Op::Mul) => a * b,
            Token::Op(Op::Div) if b == 0 => panic!("division by zero"),
            Token::Op(Op::Div) => a / b,
            _ => unreachable!(),
        }
    }
}

pub fn lex(raw: &str) -> Vec<Token> {
    let mut db: Vec<Token> = Vec::with_capacity(512);
    let bytes = raw.as_bytes();
    let mut i: usize = 0;
    let len = bytes.len();
    while i < len {
        let c = bytes[i];
        match c {
            b' ' | b'\t' | b'\n' | b'\r' => { i += 1; continue; }
            b'+' => { db.push(Token::Op(Op::Add)); i += 1; }
            b'*' => { db.push(Token::Op(Op::Mul)); i += 1; }
            b'/' => { db.push(Token::Op(Op::Div)); i += 1; }
            b'(' => { db.push(Token::LParen); i += 1; }
            b')' => { db.push(Token::RParen); i += 1; }
            b'-' => {
                let is_unary = db.is_empty() || matches!(db.last(), Some(Token::Op(_)) | Some(Token::LParen));
                if is_unary { db.push(Token::Op(Op::Neg)); } else { db.push(Token::Op(Op::Sub)); }
                i += 1;
            }
            d if d.is_ascii_digit() => {
                let mut v: i64 = 0;
                while i < len && (bytes[i] as char).is_ascii_digit() {
                    v = v * 10 + (bytes[i] - b'0') as i64;
                    i += 1;
                }
                db.push(Token::Num(v));
            }
            _ => {
                let start = i;
                while i < len && !(bytes[i] as char).is_whitespace() && !b"+-*/()".contains(&bytes[i]) {
                    i += 1;
                }
                let _bad = &raw[start..i];
                return vec![Token::Num(0), Token::Eof];
            }
        }
    }
    db.push(Token::Eof);
    db
}

pub fn compile(toks: Vec<Token>) -> Result<Vec<Token>, &'static str> {
    let mut out: Vec<Token> = Vec::with_capacity(toks.len() * 2);
    let mut ops: Vec<Token> = Vec::with_capacity(32);
    for t in toks {
        match t {
            Token::Num(_) => out.push(t),
            Token::Op(_) => {
                let t_is_neg = matches!(t, Token::Op(Op::Neg));
                while !ops.is_empty() {
                    let top = unsafe { *ops.get_unchecked(ops.len() - 1) };
                    if top == Token::LParen || top == Token::Eof { break; }
                    let tp = prec(&top);
                    let cp = prec(&t);
                    if tp > cp || (tp == cp && !t_is_neg) {
                        out.push(unsafe { ops.pop().unwrap_unchecked() });
                    } else {
                        break;
                    }
                }
                ops.push(t);
            }
            Token::LParen => ops.push(t),
            Token::RParen => {
                while let Some(top) = ops.last() {
                    if *top == Token::LParen { break; }
                    out.push(ops.pop().unwrap());
                }
                if ops.last() == Some(&Token::LParen) { ops.pop(); } else { return Err("mismatched ')'"); }
            }
            Token::Eof => break,
        }
    }
    while let Some(top) = ops.pop() {
        if matches!(top, Token::LParen | Token::RParen | Token::Eof) {
            return Err("mismatched parentheses");
        }
        out.push(top);
    }
    Ok(out)
}

pub fn eval_rpn(rpn: &[Token]) -> Result<i64, &'static str> {
    let mut st: [i64; 256] = [0; 256];
    let mut sp: usize = 0;
    for op in rpn {
        match op {
            Token::Num(n) => { st[sp] = *n; sp += 1; }
            Token::Op(Op::Add | Op::Sub | Op::Mul | Op::Div) => {
                if sp < 2 { return Err("stack underflow"); }
                let b = st[sp - 1]; let a = st[sp - 2]; sp -= 2;
                st[sp] = Token::apply(*op, a, b); sp += 1;
            }
            Token::Op(Op::Neg) => {
                if sp < 1 { return Err("stack underflow"); }
                st[sp - 1] = -st[sp - 1];
            }
            _ => return Err("unexpected token in RPN"),
        }
    }
    if sp != 1 { return Err("malformed expression"); }
    Ok(st[0])
}

pub fn evaluate(input: &str) -> Result<i64, &'static str> {
    let toks = lex(input);
    let rpn = compile(toks)?;
    eval_rpn(&rpn)
}

pub fn fast_two(input: &str) -> Option<(i64, Op, i64)> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i: usize = 0;
    while i < len && bytes[i].is_ascii_whitespace() { i += 1; }
    let s1 = i;
    while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'-') { i += 1; }
    let a: i64 = input[s1..i].parse().ok()?;
    while i < len && bytes[i].is_ascii_whitespace() { i += 1; }
    if i >= len { return None; }
    let op = match bytes[i] { b'+' => Op::Add, b'-' => Op::Sub, b'*' => Op::Mul, b'/' => Op::Div, _ => return None };
    i += 1;
    while i < len && bytes[i].is_ascii_whitespace() { i += 1; }
    let s2 = i;
    while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'-') { i += 1; }
    let b: i64 = input[s2..i].parse().ok()?;
    Some((a, op, b))
}

pub fn fmt_op(o: Op) -> &'static str {
    match o { Op::Add => "+", Op::Sub => "-", Op::Mul => "*", Op::Div => "/", _ => "?" }
}
