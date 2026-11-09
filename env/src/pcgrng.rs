use std::convert::Infallible;

use rand::{
    TryRng,
    rand_core::utils::{fill_bytes_via_next_word, next_u64_via_u32},
};

pub struct PCGRng {
    state: u64,
    inc: u64,
}

impl PCGRng {
    pub fn srandom_r(initstate: u64, initseq: u64) -> Self {
        let mut rng = PCGRng { state: 0, inc: 0 };
        rng.state = 0u64;
        rng.inc = (initseq << 1u64) | 1u64;
        rng.next_u32();
        rng.state += initstate;
        rng.next_u32();
        rng
    }

    pub fn seed(seed: u64) -> Self {
        PCGRng::srandom_r(seed, 0u64)
    }

    pub fn next_u32(&mut self) -> u32 {
        let oldstate = self.state;
        // Advance internal state
        self.state = oldstate
            .wrapping_mul(6364136223846793005u64)
            .wrapping_add(self.inc | 1);
        // Calculate output function (XSH RR), uses old state for max ILP
        let xorshifted = ((oldstate >> 18u32) ^ oldstate) >> 27u32;
        let rot = oldstate >> 59u32;
        ((xorshifted >> rot) | (xorshifted << ((-(rot as i64)) & 31))) as u32
    }
}

impl TryRng for PCGRng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.next_u32())
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        next_u64_via_u32(self)
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        fill_bytes_via_next_word(dst, || Ok(self.next_u32()))
    }
}
