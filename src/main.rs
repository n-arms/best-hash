mod expr;
mod hash;

use rand::prelude::*;
use hash::*;
use expr::{expr::*, closure::*};

fn main() {
    let mut rng = thread_rng();

    let mut scores = Vec::new();

    for _ in 0..100 {
        let expr = Expr::rand(&mut rng);
        let score = score_hasher(expr.clone(), expr.len(), 0, 100, 5, 10, 3, &mut rng);

        scores.push((expr, score));
    }

    scores.sort_by_key(|(_, score)| (score * 1000f64) as usize);

    for (expr, score) in scores.iter() {//.rev() {
        println!("score of {}\n{}\n", score, expr);
    }
}
