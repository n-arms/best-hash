mod expr;
mod hash;

use rand::prelude::*;
use hash::*;
use expr::{expr::*, closure};

const EXPRS: usize = 1000;
const CLUSTERS: usize = 1000;
const CLUSTER_SIZE: usize = 5;
const BYTES_PER_MUTATION: usize = 5;
const INIT: u64 = 0;

fn main() {
    let mut rng = thread_rng();

    let mut best_short = 0f64;
    let mut best_med = 0f64;
    let mut best_long = 0f64;

    for i in 0..EXPRS {
        if i % 10 == 0 {
            println!("{}\n", i);
        }

        let expr = Expr::rand(&mut rng);
        let len = expr.len();

        let closure = closure::Hasher::from(&expr); 
        let short = score_hasher(closure.clone(), len, INIT, CLUSTERS, CLUSTER_SIZE, 5, 5 / BYTES_PER_MUTATION, &mut rng);
        let med = score_hasher(closure.clone(), len, INIT, CLUSTERS, CLUSTER_SIZE, 25, 25 / BYTES_PER_MUTATION, &mut rng);
        let long = score_hasher(closure.clone(), len, INIT, CLUSTERS, CLUSTER_SIZE, 100, 100 / BYTES_PER_MUTATION, &mut rng);

        if short > best_short {
            println!("{} beats the old short score of {}\n{}\n", short, best_short, expr);
            best_short = short;
        }

        if med > best_med {
            println!("{} beats the old medium score of {}\n{}\n", med, best_med, expr);
            best_med = med;
        }

        if long > best_long {
            println!("{} beats the old long score of {}\n{}\n", long, best_long, expr);
            best_long = long;
        }
    }
}
