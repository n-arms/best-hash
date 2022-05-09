mod bytecode;
mod jit_prog;
mod expr;
mod hash;
mod jit;

use bytecode::code::{Instruction, Program, Value};
use bytecode::gen::emit;
use expr::expr::Expr;
use expr::parse::parse;
use expr::closure::*;
use jit::{asm::*, linux::*};
use rand::prelude::*;
use std::mem::transmute;
use jit_prog::Jit;
use std::time::{Instant, Duration};
use hash::Hash;

fn format_micros(time: u128) -> String {
    let micros = time % 1000;
    let millis = (time / 1000) % 1000;
    let seconds = (time / 1000_000) % 1000;

    format!("{}s, {}ms, {}us", seconds, millis, micros)
}

fn main() {
    let mut rng = thread_rng();

    let mut expr_time = 0;
    let mut closure_time = 0;
    let mut bytecode_time = 0;
    let mut jit_time = 0;

    for _ in 0..10000 {
        let expr = Expr::rand(&mut rng);
        let closure = Hasher::from(&expr);
        let prog = emit(&expr, 6);
        let func = Jit::<Linux_x86_64>::jit_prog(&prog);

        let bytes: Vec<_> = (0..10000).map(|_| rng.gen()).collect();
        let init = rng.gen();

        let start = Instant::now();
        let expr_res = expr.hash_bytes(init, &bytes);
        expr_time += start.elapsed().as_micros();

        let start = Instant::now();
        let closure_res = closure.hash_bytes(init, &bytes);
        closure_time += start.elapsed().as_micros();

        let start = Instant::now();
        let bytecode_res = prog.hash_bytes(init, &bytes);
        bytecode_time += start.elapsed().as_micros();

        let start = Instant::now();
        let jit_res = func.hash_bytes(init, &bytes);
        jit_time += start.elapsed().as_micros();

        assert_eq!(expr_res, closure_res);
        assert_eq!(expr_res, bytecode_res);
        assert_eq!(expr_res, jit_res);
    }

    println!(
        "naive expression evaluation took \t{}\nclosure compiled expressions took \t{}\nthe bytecode interpreter took \t\t{}\nthe jit compiler took \t\t\t{}",
        format_micros(expr_time),
        format_micros(closure_time),
        format_micros(bytecode_time),
        format_micros(jit_time),
    );
}

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
