use crate::jit::asm::Assembler;
use crate::bytecode::code::{Program, Instruction, Value};
use std::marker::PhantomData;
use std::mem::transmute;
use std::fs::write;
use std::process::*;
use std::io;
use std::slice;

pub struct Jit<A: Assembler> {
    _marker: PhantomData<A>
}

impl<A> Jit<A> 
where
    A: Assembler + Default,
    A::Memory: From<usize>
{
    pub fn jit_prog(prog: &Program) -> fn(u64, u64) -> u64 {
        let mut asm = A::default();
        Jit::asm_prog(&mut asm, prog);

        let (buffer, _) = asm.finalize();

        unsafe {
            transmute(buffer)
        }
    }

    pub fn objdump_prog(prog: &Program) -> io::Result<String> {
        let mut asm = A::default();
        Jit::asm_prog(&mut asm, prog);

        let (buffer, len) = asm.finalize();
        
        let slice = unsafe {slice::from_raw_parts(buffer, len) };

        write("temp.bin", slice)?;

        let child = Command::new("objdump")
            .arg("-D")
            .arg("-b")
            .arg("binary")
            .arg("-mi386:x86-64")
            //.arg("-M")
            //.arg("intel")
            .arg("temp.bin")
            .output()?;

        Ok(String::from_utf8_lossy(&child.stdout).to_string())
    }

    fn asm_prog(asm: &mut A, prog: &Program) {
        for instr in prog.instructions.iter().copied() {
            match instr {
                Instruction::Move(dst, Value::Immediate(num)) => {
                    asm.mov_imm(A::Memory::from(dst), num as u64)
                }
                Instruction::Move(dst, Value::Reference(src)) => {
                    asm.mov_mem(A::Memory::from(dst), A::Memory::from(src))
                }
                Instruction::MoveAbs(dst, num) => asm.mov_imm(A::Memory::from(dst), num),
                Instruction::Add(dst, Value::Immediate(num)) => asm.add_imm(A::Memory::from(dst), num),
                Instruction::Add(dst, Value::Reference(src)) => {
                    asm.add_mem(A::Memory::from(dst), A::Memory::from(src))
                }
                Instruction::Xor(dst, Value::Immediate(num)) => asm.xor_imm(A::Memory::from(dst), num),
                Instruction::Xor(dst, Value::Reference(src)) => {
                    asm.xor_mem(A::Memory::from(dst), A::Memory::from(src))
                }
                Instruction::RotLeft(dst, Value::Immediate(num)) => {
                    asm.rotl_imm(A::Memory::from(dst), num)
                }
                Instruction::RotLeft(dst, Value::Reference(src)) => {
                    asm.rotl_mem(A::Memory::from(dst), A::Memory::from(src))
                }
                Instruction::RotRight(dst, Value::Immediate(num)) => {
                    asm.rotr_imm(A::Memory::from(dst), num)
                }
                Instruction::RotRight(dst, Value::Reference(src)) => {
                    asm.rotr_mem(A::Memory::from(dst), A::Memory::from(src))
                }
            }
        }
    }
}
