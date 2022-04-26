use std::fmt;
use crate::hash::Hash;

#[derive(Clone)]
pub enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    RotLeft(Box<Expr>, Box<Expr>),
    RotRight(Box<Expr>, Box<Expr>),
    Const(u64),
    HashState,
    Byte
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Add(a, b) => write!(f, "({} + {})", a, b),
            Expr::Xor(a, b) => write!(f, "({} xor {})", a, b),
            Expr::RotLeft(a, b) => write!(f, "({} << {})", a, b),
            Expr::RotRight(a, b) => write!(f, "({} >> {})", a, b),
            Expr::Const(num) => write!(f, "{}", num),
            Expr::HashState => write!(f, "state"),
            Expr::Byte => write!(f, "byte"),
        }
    }
}

fn hash_byte(expr: &Expr, hash_state: u64, byte: u8) -> u64 {
    match expr {
        Expr::Add(a, b) => hash_byte(&a, hash_state, byte) + hash_byte(&b, hash_state, byte),
        Expr::Xor(a, b) => hash_byte(&a, hash_state, byte) ^ hash_byte(&b, hash_state, byte),
        Expr::RotLeft(a, b) => hash_byte(&a, hash_state, byte).rotate_left(hash_byte(&b, hash_state, byte) as u32),
        Expr::RotRight(a, b) => hash_byte(&a, hash_state, byte).rotate_right(hash_byte(&b, hash_state, byte) as u32),
        Expr::Const(num) => *num,
        Expr::HashState => hash_state,
        Expr::Byte => byte as u64,
    }
}

impl Hash for Expr {
    fn hash_bytes(&self, init: u64, bytes: &[u8]) -> u64 {
        let mut hash = init;

        for byte in bytes {
            hash = hash_byte(&self, hash, *byte);
        }

        hash
    }
}
