

pub trait Assembler {
    type Memory;

    fn mov_mem(&mut self, dst: Self::Memory, src: Self::Memory);
    fn mov_imm(&mut self, dst: Self::Memory, src: u64);

    fn add_mem(&mut self, dst: Self::Memory, src: Self::Memory);
    fn add_imm(&mut self, dst: Self::Memory, src: u64);

    fn xor_mem(&mut self, dst: Self::Memory, src: Self::Memory);
    fn xor_imm(&mut self, dst: Self::Memory, src: u64);

    fn rotl_mem(&mut self, dst: Self::Memory, src: Self::Memory);
    fn rotl_imm(&mut self, dst: Self::Memory, src: u64);

    fn rotr_mem(&mut self, dst: Self::Memory, src: Self::Memory);
    fn rotr_imm(&mut self, dst: Self::Memory, src: u64);
}
