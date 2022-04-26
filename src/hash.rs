

pub trait Hash {
    fn hash_bytes(&self, init: u64, bytes: &[u8]) -> u64;
}
