#[warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic
)]
mod bytecode;
mod expr;
mod hash;
mod jit;
mod jit_prog;
mod search;

use bytecode::code::{Instruction, Program, Value};
use bytecode::gen::emit;
use expr::closure::*;
use expr::expr::{Expr, Tag};
use expr::parse::parse;
use hash::{score_hasher, Hash};
use jit::{asm::*, linux::*};
use jit_prog::Jit;
use rand::prelude::*;
use search::bfs::Search;
use search::tag::Tagger;
use std::mem::transmute;
use std::time::{Duration, Instant};

fn format_micros(time: f64) -> String {
    let micros = time % 1000.;
    let millis = (time / 1000.) % 1000.;
    let seconds = (time / 1000_000.) % 1000.;

    format!("{:.0}s, {:.0}ms, {:.0}us", seconds, millis, micros)
}

fn main() {
    // calling search.next() n times, search.to_visit will contain 3n + 1 elements
    let search = Search::default();
    let tagger = Tagger::default();
    let mut rng = thread_rng();
    let mut scored_exprs: Vec<_> = search
        .take(100000)
        .enumerate()
        .map(|(i, expr)| {
            if i % 100 == 0 {
                println!("{}", i);
            }
            let mut score = 0.;
            for _ in 0..100 {
                let tagged = tagger.annotate(&expr);
                let prog = emit(&tagged, 6);
                let jit = Jit::<Linux_x86_64>::jit_prog(&prog);
                score += score_hasher(jit, tagged.len(), 0, 10, 3, 50, 3, &mut rng);
            }

            (score, expr)
        })
        .collect();

    scored_exprs.sort_by_key(|(score, _)| (score * 100f64) as u128);

    for (score, expr) in scored_exprs.iter().take(5) {
        println!("{}\n\thas score {}\n", expr, score);
    }

    for (score, expr) in scored_exprs.iter().skip(scored_exprs.len() - 5) {
        println!("{}\n\thas score {}\n", expr, score);
    }

    let len_total = scored_exprs
        .iter()
        .map(|(_, expr)| expr.len() as f64)
        .fold(0f64, |acc, len| acc + len);
    let len_bad = scored_exprs
        .iter()
        .take(5)
        .map(|(_, expr)| expr.len() as f64)
        .fold(0f64, |acc, len| acc + len);
    let len_good = scored_exprs
        .iter()
        .skip(scored_exprs.len() - 5)
        .map(|(_, expr)| expr.len() as f64)
        .fold(0f64, |acc, len| acc + len);

    println!("the average length of an expression is {}, the average length of a bad expression is {}, the average length of a good expression is {}", len_total / (scored_exprs.len() as f64), len_bad / 5f64, len_good / 5f64);
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
