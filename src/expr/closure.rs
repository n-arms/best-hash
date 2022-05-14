use super::expr::{Expr, Tag};
use crate::hash::Hash;
use std::rc::Rc;

#[derive(Clone)]
pub struct Hasher<'a> {
    closure: Rc<dyn Fn(u64, u8) -> u64 + 'a>,
}
impl<'a> From<&'a Expr<Tag>> for Hasher<'a> {
    fn from(expr: &'a Expr<Tag>) -> Self {
        match expr {
            Expr::Add(a, b) => {
                let ac = Hasher::from(a.as_ref());
                let bc = Hasher::from(b.as_ref());

                Hasher {
                    closure: Rc::new(move |hash_state, byte| {
                        (ac.closure)(hash_state, byte).wrapping_add((bc.closure)(hash_state, byte))
                    }),
                }
            }
            Expr::Xor(a, b) => {
                let ac = Hasher::from(a.as_ref());
                let bc = Hasher::from(b.as_ref());

                Hasher {
                    closure: Rc::new(move |hash_state, byte| {
                        (ac.closure)(hash_state, byte) ^ (bc.closure)(hash_state, byte)
                    }),
                }
            }
            Expr::RotLeft(a, b) => {
                let ac = Hasher::from(a.as_ref());
                let bc = Hasher::from(b.as_ref());

                Hasher {
                    closure: Rc::new(move |hash_state, byte| {
                        (ac.closure)(hash_state, byte)
                            .rotate_left((bc.closure)(hash_state, byte) as u32)
                    }),
                }
            }
            Expr::RotRight(a, b) => {
                let ac = Hasher::from(a.as_ref());
                let bc = Hasher::from(b.as_ref());

                Hasher {
                    closure: Rc::new(move |hash_state, byte| {
                        (ac.closure)(hash_state, byte)
                            .rotate_right((bc.closure)(hash_state, byte) as u32)
                    }),
                }
            }
            Expr::Tag(Tag::Const(num)) => Hasher {
                closure: Rc::new(|_, _| *num),
            },
            Expr::Tag(Tag::HashState) => Hasher {
                closure: Rc::new(|state, _| state),
            },
            Expr::Tag(Tag::Byte) => Hasher {
                closure: Rc::new(|_, byte| byte as u64),
            },
        }
    }
}

impl Hash for Hasher<'_> {
    fn hash_bytes(&self, init: u64, bytes: &[u8]) -> u64 {
        let mut hash = init;

        for byte in bytes {
            hash = (self.closure)(hash, *byte);
        }

        hash
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::prelude::*;

    const INIT: u64 = 0;

    #[test]
    fn closure_expr_eq() {
        let mut rng = thread_rng();
        let mut failed = Vec::new();

        'exprs: for _ in 0..1000 {
            let expr = Expr::rand(&mut rng);
            let closure = Hasher::from(&expr);

            for _ in 0..100 {
                let bytes: Vec<u8> = (0..10).map(|_| rng.gen()).collect();

                if expr.hash_bytes(INIT, &bytes) != closure.hash_bytes(INIT, &bytes) {
                    failed.push(expr.clone());
                    continue 'exprs;
                }
            }
        }

        if !failed.is_empty() {
            failed.sort_by_key(Expr::len);

            let smallest = failed[0].clone();

            failed.reverse();
            for expr in &failed {
                println!("{}", expr);
            }

            let closure = Hasher::from(&smallest);

            loop {
                let bytes: Vec<u8> = (0..10).map(|_| rng.gen()).collect();

                let a = closure.hash_bytes(INIT, &bytes);
                let b = smallest.hash_bytes(INIT, &bytes);
                if a != b {
                    println!("expr = {}, closure = {}", a, b);
                    panic!("{}\nexpressions failed on bytes {:?}", failed.len(), bytes);
                }
            }
        }
    }
}
