macro_rules! count_ones {
    ($bytes:expr) => {
        {
            let mut bytes = $bytes;
            let mut count = 0;

            while bytes > 0 {
                count += 1;
                bytes &= bytes - 1;
            }

            count
        }
    }
}

use rand::prelude::*;

pub trait Hash {
    fn hash_bytes(&self, init: u64, bytes: &[u8]) -> u64;
}

pub fn score_hasher<H: Hash>(
    hasher: H, 
    len: usize,
    init: u64,
    clusters: usize, 
    cluster_size: usize, 
    bytes_len: usize,
    mutations: usize,
    rng: &mut ThreadRng,
) -> f64 {
    let mut score = 0f64;

    for _ in 0..clusters {
        let bytes: Vec<u8> = (0..bytes_len).map(|_| rng.gen()).collect();
        let bytes_hash = hasher.hash_bytes(init, &bytes);

        for _ in 0..cluster_size {
            let mut new_bytes = bytes.clone();

            for _ in 0..mutations {
                let byte = rng.gen::<usize>() % bytes_len;
                let bit = rng.gen::<u8>() % 8;
                new_bytes[byte] ^= 1 << bit; // flip a random bit in a random byte in bytes
            }

            let new_bytes_hash = hasher.hash_bytes(init, &new_bytes);
            let hash_diff = count_ones!(bytes_hash ^ new_bytes_hash); // the number of bits that are different in the hash
            score += hash_diff as f64 / mutations as f64;
        }
    }

    score / clusters as f64 / cluster_size as f64 / (len as f64 + 2f64).log2() as f64
}
