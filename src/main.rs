mod bytecode;
mod disasm;
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

fn main() {
    let mut rng = thread_rng();
    //let expr = Expr::rand(&mut rng);
    let expr = parse("(state >> 64)").unwrap();
    let prog = emit(&expr, 7);
    println!("{}\n\ncompiles to\n\n{}", expr, prog);

    let mut asm = Linux_x86_64::new();

    for instr in prog.instructions {
        match instr {
            Instruction::Move(dst, Value::Immediate(num)) => {
                asm.mov_imm(Memory::from(dst), num as u64)
            }
            Instruction::Move(dst, Value::Reference(src)) => {
                asm.mov_mem(Memory::from(dst), Memory::from(src))
            }
            Instruction::MoveAbs(dst, num) => asm.mov_imm(Memory::from(dst), num),
            Instruction::Add(dst, Value::Immediate(num)) => asm.add_imm(Memory::from(dst), num),
            Instruction::Add(dst, Value::Reference(src)) => {
                asm.add_mem(Memory::from(dst), Memory::from(src))
            }
            Instruction::Xor(dst, Value::Immediate(num)) => asm.xor_imm(Memory::from(dst), num),
            Instruction::Xor(dst, Value::Reference(src)) => {
                asm.xor_mem(Memory::from(dst), Memory::from(src))
            }
            Instruction::RotLeft(dst, Value::Immediate(num)) => {
                asm.rotl_imm(Memory::from(dst), num)
            }
            Instruction::RotLeft(dst, Value::Reference(src)) => {
                asm.rotl_mem(Memory::from(dst), Memory::from(src))
            }
            Instruction::RotRight(dst, Value::Immediate(num)) => {
                asm.rotr_imm(Memory::from(dst), num)
            }
            Instruction::RotRight(dst, Value::Reference(src)) => {
                asm.rotr_mem(Memory::from(dst), Memory::from(src))
            }
        }
    }

    let (buffer, len) = asm.finalize();

    disasm::print_objdump(buffer, len);

    let func: fn(u64, u64) -> u64 = unsafe { transmute(buffer) };

    println!("{}", func(4, 42));
}
