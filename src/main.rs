#[warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic
)]

mod bytecode;
mod jit_prog;
mod expr;
mod hash;
mod jit;
mod search;

use bytecode::code::{Instruction, Program, Value};
use bytecode::gen::emit;
use expr::expr::{Tag, Expr};
use expr::parse::parse;
use expr::closure::*;
use jit::{asm::*, linux::*};
use rand::prelude::*;
use std::mem::transmute;
use jit_prog::Jit;
use std::time::{Instant, Duration};
use hash::Hash;
use search::bfs::Search;

fn format_micros(time: f64) -> String {
    let micros = time % 1000.;
    let millis = (time / 1000.) % 1000.;
    let seconds = (time / 1000_000.) % 1000.;

    format!("{:.0}s, {:.0}ms, {:.0}us", seconds, millis, micros)
}

fn main() {
    // calling search.next() n times, search.to_visit will contain 3n + 1 elements
    let mut search = Search::default();

    for _ in 0..100 {
        println!("element {}, search len is now {}\n", search.next().unwrap(), search.len());
    }
}

/*
fn main() {
    print!("\n\n\n\n\n\n");
    let mut rng = thread_rng();

    let mut expr_time = 0;
    let mut closure_time = 0;
    let mut bytecode_time = 0;
    let mut jit_time = 0;
    let mut expr_len = 0;

    for i in 0..100000 {
        if i % 100 == 0 {
            println!("\x1b[5A\x1b[1000Di = {}, avg expression length is {}", i, expr_len as f64 / i as f64);
            print!(
                "(per 10 expressions)\nnaive expression evaluation took \t{}\nclosure compiled expressions took \t{}\nthe bytecode interpreter took \t\t{}\nthe jit compiler took \t\t\t{}",
                format_micros(10. * expr_time as f64 / (i + 1) as f64),
                format_micros(10. * closure_time as f64 / (i + 1) as f64),
                format_micros(10. * bytecode_time as f64 / (i + 1) as f64),
                format_micros(10. * jit_time as f64 / (i + 1) as f64),
            );
        }
        let expr = Expr::rand(&mut rng);
        expr_len += expr.len();
        let closure = Hasher::from(&expr);
        let prog = emit(&expr, 6);
        let func = Jit::<Linux_x86_64>::jit_prog(&prog);
        std::fs::write("log.txt", format!("{}\nwith bytecode\n{}\nand machine code\n{}\n", expr, prog, Jit::<Linux_x86_64>::objdump_prog(&prog).unwrap())).unwrap();

        let bytes: Vec<_> = (0..1000).map(|_| rng.gen()).collect();
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

    println!();
}
*/

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
            let prog = emit(&expr, 6);
            let func = Jit::<Linux_x86_64>::jit_prog(&prog);

            for _ in 0..TESTS {
                let hash_state = rng.gen();
                let byte: u8 = rng.gen();
                let jit_res = func(hash_state, byte as u64);
                let eval_res = prog.eval(hash_state, byte);

                if jit_res != eval_res {
                    failed.push(expr);
                    continue 'outer;
                }
            }
        }

        if !failed.is_empty() {
            failed.sort_by_key(|e| usize::MAX - e.len());

            for expr in &failed {
                println!("\n{}", expr);
            }

            panic!("{} out of {} expressions failed", failed.len(), ITERS);
        }
    }
}
