use crate::hash::Hash;
use rand::prelude::*;
use std::fmt;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Expr<TAG> {
    Add(Box<Expr<TAG>>, Box<Expr<TAG>>),
    Xor(Box<Expr<TAG>>, Box<Expr<TAG>>),
    RotLeft(Box<Expr<TAG>>, Box<Expr<TAG>>),
    RotRight(Box<Expr<TAG>>, Box<Expr<TAG>>),
    Tag(TAG)
}

#[derive(Clone)]
pub enum Tag {
    Const(u64),
    HashState,
    Byte
}

impl fmt::Display for Expr<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Add(a, b) => write!(f, "({} + {})", a, b),
            Expr::Xor(a, b) => write!(f, "({} xor {})", a, b),
            Expr::RotLeft(a, b) => write!(f, "({} << {})", a, b),
            Expr::RotRight(a, b) => write!(f, "({} >> {})", a, b),
            Expr::Tag(Tag::Const(num)) => write!(f, "{}", num),
            Expr::Tag(Tag::HashState) => write!(f, "state"),
            Expr::Tag(Tag::Byte) => write!(f, "byte"),
        }
    }
}

impl fmt::Display for Expr<()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Add(a, b) => write!(f, "({} + {})", a, b),
            Expr::Xor(a, b) => write!(f, "({} xor {})", a, b),
            Expr::RotLeft(a, b) => write!(f, "({} << {})", a, b),
            Expr::RotRight(a, b) => write!(f, "({} >> {})", a, b),
            Expr::Tag(()) => write!(f, "tag")
        }
    }
}

fn hash_byte(expr: &Expr<Tag>, hash_state: u64, byte: u8) -> u64 {
    match expr {
        Expr::Add(a, b) => {
            hash_byte(a, hash_state, byte).wrapping_add(hash_byte(b, hash_state, byte))
        }
        Expr::Xor(a, b) => hash_byte(a, hash_state, byte) ^ hash_byte(b, hash_state, byte),
        Expr::RotLeft(a, b) => {
            hash_byte(a, hash_state, byte).rotate_left(hash_byte(b, hash_state, byte) as u32)
        }
        Expr::RotRight(a, b) => {
            hash_byte(a, hash_state, byte).rotate_right(hash_byte(b, hash_state, byte) as u32)
        }
        Expr::Tag(Tag::Const(num)) => *num,
        Expr::Tag(Tag::HashState) => hash_state,
        Expr::Tag(Tag::Byte) => byte as u64,
    }
}

impl Hash for Expr<Tag> {
    fn hash_bytes(&self, init: u64, bytes: &[u8]) -> u64 {
        let mut hash = init;

        for byte in bytes {
            hash = hash_byte(self, hash, *byte);
        }

        hash
    }
}

impl Expr<Tag> {
    pub fn rand(rng: &mut ThreadRng) -> Expr<Tag> {
        Expr::rand_with_depth(rng, 0)
    }

    pub fn rand_with_depth(rng: &mut ThreadRng, depth: usize) -> Expr<Tag> {
        let seed = rng.gen::<u8>() % if depth >= 10 { 4 } else { 8 };

        match seed {
            0 | 1 => Expr::Tag(Tag::Const(rng.gen())),
            2 => Expr::Tag(Tag::Byte),
            3 => Expr::Tag(Tag::HashState),
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
            Expr::Xor(a, b) | Expr::RotLeft(a, b) | Expr::RotRight(a, b) | Expr::Add(a, b) => {
                a.len() + b.len()
            }
            Expr::Tag(Tag::Const(_)) | Expr::Tag(Tag::HashState) | Expr::Tag(Tag::Byte) => 1,
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            Expr::Xor(a, b) | Expr::RotLeft(a, b) | Expr::RotRight(a, b) | Expr::Add(a, b) => {
                a.len().max(b.len()) + 1
            }
            Expr::Tag(Tag::Const(num)) if *num > u32::MAX as u64 => 1,
            Expr::Tag(Tag::Const(_)) | Expr::Tag(Tag::HashState) | Expr::Tag(Tag::Byte) => 0,
        }
    }
}
