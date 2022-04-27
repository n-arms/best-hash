use crate::hash::Hash;
use std::fmt;

#[derive(Debug)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub result: Value,
}

pub type Memory = usize;

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Reference(Memory),
    Immediate(u64),
}

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    Move(Memory, Value),
    Add(Memory, Value),
    Xor(Memory, Value),
    RotLeft(Memory, Value),
    RotRight(Memory, Value),
}

impl Program {
    pub fn eval(&self, hash_state: u64, byte: u8) -> u64 {
        let mut mem = vec![0u64; 2.max(self.biggest_ptr() + 1)];
        mem[0] = hash_state;
        mem[1] = byte as u64;

        for instr in &self.instructions {
            match instr {
                Instruction::Move(dst, Value::Immediate(val)) => mem[*dst] = *val,
                Instruction::Move(dst, Value::Reference(src)) => mem[*dst] = mem[*src],
                Instruction::Add(dst, Value::Immediate(val)) => {
                    mem[*dst] = mem[*dst].wrapping_add(*val)
                }
                Instruction::Add(dst, Value::Reference(src)) => {
                    mem[*dst] = mem[*dst].wrapping_add(mem[*src])
                }
                Instruction::Xor(dst, Value::Immediate(val)) => mem[*dst] ^= *val,
                Instruction::Xor(dst, Value::Reference(src)) => mem[*dst] ^= mem[*src],
                Instruction::RotLeft(dst, Value::Immediate(val)) => {
                    mem[*dst] = mem[*dst].rotate_left(*val as u32)
                }
                Instruction::RotLeft(dst, Value::Reference(src)) => {
                    mem[*dst] = mem[*dst].rotate_left(mem[*src] as u32)
                }
                Instruction::RotRight(dst, Value::Immediate(val)) => {
                    mem[*dst] = mem[*dst].rotate_right(*val as u32)
                }
                Instruction::RotRight(dst, Value::Reference(src)) => {
                    mem[*dst] = mem[*dst].rotate_right(mem[*src] as u32)
                }
            }
        }

        match self.result {
            Value::Immediate(val) => val,
            Value::Reference(idx) => mem[idx],
        }
    }

    // return the largest memory address written to in a program
    pub fn biggest_ptr(&self) -> usize {
        let mut biggest = 0;

        for instr in &self.instructions {
            match instr {
                Instruction::Move(dst, _)
                | Instruction::Add(dst, _)
                | Instruction::Xor(dst, _)
                | Instruction::RotLeft(dst, _)
                | Instruction::RotRight(dst, _) => biggest = biggest.max(*dst),
            }
        }

        biggest
    }
}

impl Hash for Program {
    fn hash_bytes(&self, init: u64, bytes: &[u8]) -> u64 {
        let mut hash = init;
        for byte in bytes {
            hash = self.eval(hash, *byte);
        }
        hash
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Reference(mem) => write!(f, "%{}", mem),
            Value::Immediate(val) => write!(f, "${}", val),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Add(dst, src) => write!(f, "add %{} {}", dst, src),
            Instruction::Xor(dst, src) => write!(f, "xor %{} {}", dst, src),
            Instruction::RotLeft(dst, src) => write!(f, "rotl %{} {}", dst, src),
            Instruction::RotRight(dst, src) => write!(f, "rotr %{} {}", dst, src),
            Instruction::Move(dst, src) => write!(f, "mov %{} {}", dst, src),
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} where", self.result)?;
        for instr in &self.instructions {
            writeln!(f, "{}", instr)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bytecode::gen::emit;
    use crate::expr::expr::Expr;
    use rand::prelude::*;

    const INIT: u64 = 0;

    #[test]
    fn emit_eval_expr_eq() {
        let mut rng = thread_rng();
        let mut failed = Vec::new();

        'exprs: for _ in 0..1000 {
            let expr = Expr::rand(&mut rng);
            let prog = emit(&expr, 10);

            for _ in 0..100 {
                let bytes: Vec<u8> = (0..10).map(|_| rng.gen()).collect();

                if expr.hash_bytes(INIT, &bytes) != prog.hash_bytes(INIT, &bytes) {
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

            let closure = emit(&smallest, 10);

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
