use crate::hash::Hash;
use super::expr::Expr;

pub struct Hasher<'a> {
    closure: Box<dyn Fn(u64, u8) -> u64 + 'a>
}
impl<'a> From<&'a Expr> for Hasher<'a> {
    fn from(expr: &'a Expr) -> Self {
        match expr {
            Expr::Add(a, b) => {
                let ac = Hasher::from(a.as_ref());
                let bc = Hasher::from(b.as_ref());

                Hasher {
                    closure: Box::new(move |hash_state, byte| (ac.closure)(hash_state, byte) + (bc.closure)(hash_state, byte))
                }
            },
            Expr::Xor(a, b) => {
                let ac = Hasher::from(a.as_ref());
                let bc = Hasher::from(b.as_ref());

                Hasher {
                    closure: Box::new(move |hash_state, byte| (ac.closure)(hash_state, byte) ^ (bc.closure)(hash_state, byte))
                }
            },
            Expr::RotLeft(a, b) => {
                let ac = Hasher::from(a.as_ref());
                let bc = Hasher::from(b.as_ref());

                Hasher {
                    closure: Box::new(move |hash_state, byte| (ac.closure)(hash_state, byte).rotate_left((bc.closure)(hash_state, byte) as u32))
                }
            },
            Expr::RotRight(a, b) => {
                let ac = Hasher::from(a.as_ref());
                let bc = Hasher::from(b.as_ref());

                Hasher {
                    closure: Box::new(move |hash_state, byte| (ac.closure)(hash_state, byte).rotate_right((bc.closure)(hash_state, byte) as u32))
                }
            },
            Expr::Const(num) => {
                Hasher {
                    closure: Box::new(|_, _| *num)
                }
            }
            Expr::HashState => {
                Hasher {
                    closure: Box::new(|state, _| state)
                }
            },
            Expr::Byte => {
                Hasher {
                    closure: Box::new(|_, byte| byte as u64)
                }
            },
        }
    }
}

impl Hash for Hasher<'_> {
    fn hash_bytes(&self, init: u64, bytes: &[u8]) -> u64 {
        let mut hash = init;

        for byte in bytes {
            hash = (self.closure)(init, *byte);
        }

        hash
    }
}
