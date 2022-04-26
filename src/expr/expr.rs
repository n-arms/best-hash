use std::fmt;
use crate::hash::Hash;
use rand::prelude::*;

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
        Expr::Add(a, b) => hash_byte(&a, hash_state, byte).wrapping_add(hash_byte(&b, hash_state, byte)),
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

impl Expr {
    pub fn rand(rng: &mut ThreadRng) -> Expr {
        Expr::rand_with_depth(rng, 0)
    }

    pub fn rand_with_depth(rng: &mut ThreadRng, depth: usize) -> Expr {
        let seed = rng.gen::<u8>() % if depth >= 10 {
            4
        } else {
            8
        };

        match seed {
            0 | 1 => Expr::Const(rng.gen()),
            2 => Expr::Byte,
            3 => Expr::HashState,
            4 => {
                let a = Expr::rand_with_depth(rng, depth + 1);
                let b = Expr::rand_with_depth(rng, depth + 1);
                
                Expr::Add(Box::new(a), Box::new(b))
            }
            5 => {
                let a = Expr::rand_with_depth(rng, depth + 1);
                let b = Expr::rand_with_depth(rng, depth + 1);
                
                Expr::Xor(Box::new(a), Box::new(b))
            }
            6 => {
                let a = Expr::rand_with_depth(rng, depth + 1);
                let b = Expr::rand_with_depth(rng, depth + 1);
                
                Expr::RotLeft(Box::new(a), Box::new(b))
            }
            7..=u8::MAX => {
                let a = Expr::rand_with_depth(rng, depth + 1);
                let b = Expr::rand_with_depth(rng, depth + 1);
                
                Expr::RotRight(Box::new(a), Box::new(b))
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Expr::Xor(a, b) |
            Expr::RotLeft(a, b) |
            Expr::RotRight(a, b) |
            Expr::Add(a, b) => a.len() + b.len(),
            Expr::Const(_) |
            Expr::HashState |
            Expr::Byte => 1,
        }
    }
}
