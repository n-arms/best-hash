mod bytecode;
mod jit_prog;
mod expr;
mod hash;
mod jit;

use bytecode::code::{Instruction, Program, Value};
use bytecode::gen::emit;
use expr::expr::Expr;
use expr::parse::parse;
use jit::{asm::*, linux::*};
use rand::prelude::*;
use std::mem::transmute;
use jit_prog::Jit;

fn main() {}

#[cfg(test)]
mod test {
    use super::*;

    const ITERS: usize = 1000;
    const TESTS: usize = 10;

    #[test]
    fn eval_jit_eq() {
        let mut rng = thread_rng();

        let mut failed = Vec::new();

        'outer: for _ in 0..ITERS {
            let expr = Expr::rand(&mut rng);
            let prog = emit(&expr, 7);
            let func = Jit::<Linux_x86_64>::jit_prog(&prog);

            for _ in 0..TESTS {
                let hash_state = rng.gen();
                let byte = rng.gen();
                let jit_res = func(hash_state, byte);
                let eval_res = prog.eval(hash_state, byte as u8);

                if jit_res != eval_res {
                    failed.push(expr);
                    continue 'outer;
                }
            }
        }

        if !failed.is_empty() {
            failed.sort_by_key(|e| e.len());

            for expr in failed {
                println!("\n{}", expr);
            }
        }
    }
}
