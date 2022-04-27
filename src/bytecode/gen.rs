use super::code::*;
use crate::expr::expr::Expr;
use std::collections::HashSet;

pub fn emit(expr: &Expr, registers: usize) -> Program {
    let mem_idx = register_allocate(expr, registers);

    emit_expr(expr, &mem_idx)
}

type BinInstruction = fn(Memory, Value) -> Instruction;

fn emit_expr(expr: &Expr, mem_idx: &[usize]) -> Program {
    let (bin_instr, a, b) = match expr {
        Expr::Add(a, b) => (Instruction::Add as BinInstruction, a, b),
        Expr::Xor(a, b) => (Instruction::Xor as BinInstruction, a, b),
        Expr::RotLeft(a, b) => (Instruction::RotLeft as BinInstruction, a, b),
        Expr::RotRight(a, b) => (Instruction::RotRight as BinInstruction, a, b),
        Expr::Const(num) => {
            return Program {
                instructions: Vec::new(),
                result: Value::Immediate(*num),
            }
        }
        Expr::HashState => {
            return Program {
                instructions: Vec::new(),
                result: Value::Reference(0),
            }
        }
        Expr::Byte => {
            return Program {
                instructions: Vec::new(),
                result: Value::Reference(1),
            }
        }
    };

    let Program {
        mut instructions,
        result: a_res,
    } = emit_expr(&a, &mem_idx[1..]);

    instructions.push(Instruction::Move(mem_idx[0], a_res));

    let Program {
        instructions: b_instrs,
        result: b_res,
    } = emit_expr(&b, &mem_idx[1..]);

    instructions.extend(b_instrs);

    instructions.push(bin_instr(mem_idx[0], b_res));

    Program {
        instructions,
        result: Value::Reference(mem_idx[0]),
    }
}

// allocate the registers for the intermediate results of an expression
//
// the result memory index of a subexpression of depth n can be found by taking the nth index of
// the vector you get from `register_allocate`
fn register_allocate(expr: &Expr, registers: usize) -> Vec<usize> {
    let mut levels = vec![0usize; expr.depth()];
    measure_levels(&expr, &mut levels);

    let mut tagged_levels: Vec<_> = levels
        .iter()
        .copied()
        .enumerate()
        .map(|(level, measure)| (level, measure))
        .collect();
    tagged_levels.sort_by_key(|(_, measure)| *measure);

    let top_levels: HashSet<_> = tagged_levels
        .iter()
        .rev()
        .take(registers)
        .map(|(level, _)| *level)
        .collect();

    let mut memory_idx = levels; // reuse the memory from levels
    let mut reg_offset = 2; // the first two registers / memory slots are reserved for the hash state and the byte being hashed
    let mut mem_offset = reg_offset + registers;

    for i in 0..memory_idx.len() {
        if top_levels.contains(&i) {
            memory_idx[i] = reg_offset;
            reg_offset += 1;
        } else {
            memory_idx[i] = mem_offset;
            mem_offset += 1;
        }
    }

    memory_idx
}

fn measure_levels(expr: &Expr, levels: &mut [usize]) {
    match expr {
        Expr::Add(a, b) | Expr::Xor(a, b) | Expr::RotLeft(a, b) | Expr::RotRight(a, b) => {
            levels[0] += 1;
            measure_levels(&a, &mut levels[1..]);
            measure_levels(&b, &mut levels[1..]);
        }
        Expr::Const(_) | Expr::HashState | Expr::Byte => (),
    }
}
