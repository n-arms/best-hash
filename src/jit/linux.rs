use super::asm::Assembler;
use super::code_vec::CodeVec;
use libc::{c_void, _SC_PAGESIZE, munmap};
use std::mem::transmute;

pub struct Linux_x86_64 {
    pub buffer: CodeVec,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Memory {
    Register(Register),
    Stack(usize),
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Register {
    /// argument 2
    RDI,
    /// argument 1
    RSI,
    /// has to be used for memory loads and stores
    RAX,
    /// has to be used for rotations
    RCX,
    RDX,
    R8,
    R9,
    R10,
    R11,
}

impl Register {
    pub fn emit(&self) -> u8 {
        match self {
            Register::RAX => 0,
            Register::RCX => 1,
            Register::RDX => 2,
            Register::RSI => 6,
            Register::RDI => 7,
            Register::R8 => 8,
            Register::R9 => 9,
            Register::R10 => 10,
            Register::R11 => 11,
        }
    }
}

impl From<usize> for Memory {
    fn from(num: usize) -> Self {
        let register = match num {
            0 => Register::RDI,
            1 => Register::RSI,
            2 => Register::RAX,
            3 => Register::RCX,
            4 => Register::RDX,
            5 => Register::R8,
            6 => Register::R9,
            7 => Register::R10,
            8 => Register::R11,
            _ => return Memory::Stack(num - 8),
        };
        Memory::Register(register)
    }
}

/// calculate the byte used to refer to the given index on the stack
fn to_stack_idx(idx: usize) -> u8 {
    let [byte, ..] = (-8 * idx as i8).to_le_bytes();
    byte
}

impl Default for Linux_x86_64 {
    fn default() -> Linux_x86_64 {
        let mut buffer = CodeVec::new(_SC_PAGESIZE as usize);
        buffer.push(0xf3);
        buffer.push(0x0f);
        buffer.push(0x1e);
        buffer.push(0xfa);
        Linux_x86_64 { buffer }
    }
}

impl Assembler for Linux_x86_64 {
    type Memory = Memory;

    fn mov_mem(&mut self, dst: Self::Memory, src: Self::Memory) {
        match (dst, src) {
            (Memory::Register(dst_reg), Memory::Register(src_reg)) => {
                self.buffer
                    .push(0x48 + (dst_reg.emit() >> 3) + ((src_reg.emit() >> 3) << 2));
                self.buffer.push(0x89);
                self.buffer
                    .push(0b11_000_000 + ((src_reg.emit() % 8) << 3) + (dst_reg.emit() % 8));
            }
            // 48 8b 7c 24 08       	mov    0x8(%rsp),%rdi
            // 48 8b 74 24 08       	mov    0x8(%rsp),%rsi
            // 48 8b 4c 24 08       	mov    0x8(%rsp),%rcx
            // 48 8b 54 24 08       	mov    0x8(%rsp),%rdx
            // 7c = 01 111 100
            // 74 = 01 110 100
            // 4c =
            (Memory::Register(dst_reg), Memory::Stack(src_idx)) => {
                self.buffer.push(0x48 + ((dst_reg.emit() >> 3) << 2));
                self.buffer.push(0x8b);
                self.buffer.push(0b01_000_100 + ((dst_reg.emit() % 8) << 3));
                self.buffer.push(0x24);
                self.buffer.push(to_stack_idx(src_idx));
            }
            (Memory::Stack(dst_idx), Memory::Register(src_reg)) => {
                self.buffer.push(0x48 + ((src_reg.emit() >> 3) << 2));
                self.buffer.push(0x89);
                self.buffer.push(0b01_000_100 + ((src_reg.emit() % 8) << 3));
                self.buffer.push(0x24);
                //self.buffer.push((dst_idx * 8) as u8);
                self.buffer.push(to_stack_idx(dst_idx));
            }
            (Memory::Stack(dst_idx), Memory::Stack(src_idx)) => {
                self.mov_mem(Memory::Register(Register::RAX), Memory::Stack(src_idx));
                self.mov_mem(Memory::Stack(dst_idx), Memory::Register(Register::RAX));
            }
        }
    }

    fn mov_imm(&mut self, dst: Self::Memory, src: u64) {
        match dst {
            Memory::Register(dst_reg) => {
                self.buffer.push(0x48 + (dst_reg.emit() >> 3));
                self.buffer.push(0b10_111_000 + (dst_reg.emit() % 8));
                for byte in src.to_le_bytes() {
                    self.buffer.push(byte);
                }
            }
            Memory::Stack(dst_idx) => {
                self.mov_imm(Memory::Register(Register::RAX), src);
                self.mov_mem(Memory::Stack(dst_idx), Memory::Register(Register::RAX));
            }
        }
    }
    /*
    48 01 f7    	add    %rsi,%rdi     = 11 110 111
    48 01 ce    	add    %rcx,%rsi     = 11 001 110
    48 01 d1    	add    %rdx,%rcx     = 11 010 001
    48 01 fa    	add    %rdi,%rdx     = 11 111 010
    */

    fn add_mem(&mut self, dst: Self::Memory, src: Self::Memory) {
        match (dst, src) {
            (Memory::Register(dst_reg), Memory::Register(src_reg)) => {
                self.buffer
                    .push(0x48 + (dst_reg.emit() >> 3) + ((src_reg.emit() >> 3) << 2));
                self.buffer.push(0x01);
                self.buffer
                    .push(0b11_000_000 + ((src_reg.emit() % 8) << 3) + (dst_reg.emit() % 8));
            }
            (Memory::Stack(dst_idx), Memory::Register(src_reg)) => {
                self.buffer.push(0x48 + ((src_reg.emit() >> 3) << 2));
                self.buffer.push(0x01);
                self.buffer.push(0b01_000_100 + ((src_reg.emit() % 8) << 3));
                self.buffer.push(0x24);
                self.buffer.push(to_stack_idx(dst_idx));
            }
            (Memory::Register(dst_reg), Memory::Stack(src_idx)) => {
                self.buffer.push(0x48 + ((dst_reg.emit() >> 3) << 2));
                self.buffer.push(0x03);
                self.buffer.push(0b01_000_100 + ((dst_reg.emit() % 8) << 3));
                self.buffer.push(0x24);
                self.buffer.push(to_stack_idx(src_idx));
            }
            (Memory::Stack(dst_idx), Memory::Stack(src_idx)) => {
                self.mov_mem(Memory::Register(Register::RAX), Memory::Stack(dst_idx));
                self.add_mem(Memory::Register(Register::RAX), Memory::Stack(src_idx));
                self.mov_mem(Memory::Stack(dst_idx), Memory::Register(Register::RAX));
            }
        }
    }
    /*
    48 81 c7 23 c1 ab 00 	add    $0xabc123,%rdi c7 = 11 000 111
    48 81 c6 23 c1 ab 00 	add    $0xabc123,%rsi c6 = 11 000 110
    48 81 c1 23 c1 ab 00 	add    $0xabc123,%rcx c1 = 11 000 001
    48 81 c2 23 c1 1b 00 	add    $0x1bc123,%rdx c2 = 11 000 010
    */
    fn add_imm(&mut self, dst: Self::Memory, src: u32) {
        match dst {
            Memory::Register(dst_reg) => {
                self.buffer.push(0x48 + (dst_reg.emit() >> 3));
                self.buffer.push(0x81);
                self.buffer.push(0b11_000_000 + (dst_reg.emit() % 8));

                for byte in src.to_le_bytes() {
                    self.buffer.push(byte);
                }
            }
            Memory::Stack(dst_idx) => {
                self.buffer.push(0x48);
                self.buffer.push(0x81);
                self.buffer.push(0x44);
                self.buffer.push(0x24);
                self.buffer.push(to_stack_idx(dst_idx));

                for byte in src.to_le_bytes() {
                    self.buffer.push(byte);
                }
            }
        }
    }

    fn xor_mem(&mut self, dst: Self::Memory, src: Self::Memory) {
        match (dst, src) {
            (Memory::Register(dst_reg), Memory::Register(src_reg)) => {
                self.buffer
                    .push(0x48 + (dst_reg.emit() >> 3) + ((src_reg.emit() >> 3) << 2));
                self.buffer.push(0x31);
                self.buffer
                    .push(0b11_000_000 + ((src_reg.emit() % 8) << 3) + (dst_reg.emit() % 8));
            }
            (Memory::Stack(dst_idx), Memory::Register(src_reg)) => {
                self.buffer.push(0x48 + ((src_reg.emit() >> 3) << 2));
                self.buffer.push(0x31);
                self.buffer.push(0b01_000_100 + ((src_reg.emit() % 8) << 3));
                self.buffer.push(0x24);
                self.buffer.push(to_stack_idx(dst_idx));
            }
            (Memory::Register(dst_reg), Memory::Stack(src_idx)) => {
                self.buffer.push(0x48 + ((dst_reg.emit() >> 3) << 2));
                self.buffer.push(0x33);
                self.buffer.push(0b01_000_100 + ((dst_reg.emit() % 8) << 3));
                self.buffer.push(0x24);
                self.buffer.push(to_stack_idx(src_idx));
            }
            (Memory::Stack(dst_idx), Memory::Stack(src_idx)) => {
                self.mov_mem(Memory::Register(Register::RAX), Memory::Stack(dst_idx));
                self.xor_mem(Memory::Register(Register::RAX), Memory::Stack(src_idx));
                self.mov_mem(Memory::Stack(dst_idx), Memory::Register(Register::RAX));
            }
        }
    }
    /*
    48 81 f7 23 c1 ab 00 	xor    $0xabc123,%rdi    f7 = 11 110 111
    48 81 f6 23 c1 ab 00 	xor    $0xabc123,%rsi    f6 = 11 110 110
    48 81 f1 23 c1 ab 00 	xor    $0xabc123,%rcx    f1 = 11 110 001
    48 81 f2 23 c1 ab 00 	xor    $0xabc123,%rdx    f2 = 11 110 010
    */

    fn xor_imm(&mut self, dst: Self::Memory, src: u32) {
        match dst {
            Memory::Register(dst_reg) => {
                self.buffer.push(0x48 + (dst_reg.emit() >> 3));
                self.buffer.push(0x81);
                self.buffer.push(0b11_110_000 + (dst_reg.emit() % 8));

                for byte in src.to_le_bytes() {
                    self.buffer.push(byte);
                }
            }
            Memory::Stack(dst_idx) => {
                self.buffer.push(0x48);
                self.buffer.push(0x81);
                self.buffer.push(0x74);
                self.buffer.push(0x24);
                self.buffer.push(to_stack_idx(dst_idx));

                for byte in src.to_le_bytes() {
                    self.buffer.push(byte);
                }
            }
        }
    }

    // note: all the rotation instructions will trash the RCX register
    fn rotl_mem(&mut self, dst: Self::Memory, src: Self::Memory) {
        match (dst, src) {
            (Memory::Register(dst_reg), _) => {
                if src != Memory::Register(Register::RCX) {
                    self.mov_mem(Memory::Register(Register::RCX), src);
                }
                self.buffer.push(0x48 + (dst_reg.emit() >> 3));
                self.buffer.push(0xd3);
                self.buffer.push(0b11_000_000 + (dst_reg.emit() % 8));
            }
            _ => {
                self.mov_mem(Memory::Register(Register::RAX), dst);
                self.rotl_mem(Memory::Register(Register::RAX), src);
                self.mov_mem(dst, Memory::Register(Register::RAX));
            }
        }
    }

    fn rotl_imm(&mut self, dst: Self::Memory, src: u32) {
        match dst {
            Memory::Register(dst_reg) => {
                self.buffer.push(0x48 + (dst_reg.emit() >> 3));
                self.buffer.push(0xc1);
                self.buffer.push(0b11_000_000 + (dst_reg.emit() % 8));
                self.buffer.push((src % u8::MAX as u32) as u8);
            }
            Memory::Stack(dst_idx) => {
                self.mov_mem(Memory::Register(Register::RAX), Memory::Stack(dst_idx));
                self.rotl_imm(Memory::Register(Register::RAX), src);
                self.mov_mem(Memory::Stack(dst_idx), Memory::Register(Register::RAX));
            }
        }
    }

    fn rotr_mem(&mut self, dst: Self::Memory, src: Self::Memory) {
        match (dst, src) {
            (Memory::Register(dst_reg), _) => {
                if src != Memory::Register(Register::RCX) {
                    self.mov_mem(Memory::Register(Register::RCX), src);
                }
                self.buffer.push(0x48 + (dst_reg.emit() >> 3));
                self.buffer.push(0xd3);
                self.buffer.push(0b11_001_000 + (dst_reg.emit() % 8));
            }
            _ => {
                self.mov_mem(Memory::Register(Register::RAX), dst);
                self.rotr_mem(Memory::Register(Register::RAX), src);
                self.mov_mem(dst, Memory::Register(Register::RAX));
            }
        }
    }

    fn rotr_imm(&mut self, dst: Self::Memory, src: u32) {
        match dst {
            Memory::Register(dst_reg) => {
                self.buffer.push(0x48 + (dst_reg.emit() >> 3));
                self.buffer.push(0xc1);
                self.buffer.push(0b11_001_000 + (dst_reg.emit() % 8));
                self.buffer.push((src % u8::MAX as u32) as u8);
            }
            Memory::Stack(dst_idx) => {
                self.mov_mem(Memory::Register(Register::RAX), Memory::Stack(dst_idx));
                self.rotr_imm(Memory::Register(Register::RAX), src);
                self.mov_mem(Memory::Stack(dst_idx), Memory::Register(Register::RAX));
            }
        }
    }

    fn finalize(mut self) -> (*const u8, usize, Box<dyn Fn()>) {
        self.buffer.push(0xc3);
        let (buffer, len, cap) = self.buffer.into_raw_parts();
        (buffer as *const u8, len, Box::new(move || unsafe {
            munmap(buffer as *mut c_void, cap);
        }))
    }
}
