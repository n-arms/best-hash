mod bytecode;
mod expr;
mod hash;

use expr::parse::parse;
use expr::expr::Expr;
use bytecode::gen::emit;
use rand::prelude::*;

fn main() {
    let mut rng = thread_rng();
    let expr = Expr::rand(&mut rng);
    println!("{}\n\ncompiles to\n\n{}", expr, emit(&expr, 7));
}
