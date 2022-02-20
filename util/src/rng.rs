use std::mem;
use std::slice;

use rand::{self, RngCore, SeedableRng};

#[derive(Clone)]
pub struct JKiss32Rng {
    seed: JKiss32RngSeed,
    c: bool,
}

impl SeedableRng for JKiss32Rng {
    type Seed = JKiss32RngSeed;

    fn from_seed(seed: Self::Seed) -> Self {
        Self { seed, c: false }
    }
}

impl RngCore for JKiss32Rng {
    fn next_u32(&mut self) -> u32 {
        self.seed.y ^= self.seed.y << 5;
        self.seed.y ^= self.seed.y >> 7;
        self.seed.y ^= self.seed.y << 22;
        let t = self
            .seed
            .z
            .wrapping_add(self.seed.w)
            .wrapping_add(self.c as u32) as i32;
        self.seed.z = self.seed.w;
        self.c = t < 0;
        self.seed.w = (t & 0x7FFFFFFF) as u32;
        self.seed.x = self.seed.x.wrapping_add(1411392427);
        self.seed
            .x
            .wrapping_add(self.seed.y)
            .wrapping_add(self.seed.w)
    }

    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | self.next_u32() as u64
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let count = dest.len() / 4;
        let offset = count * 4;
        let remainder = dest.len() - offset;

        let mut ptr = dest.as_mut_ptr() as *mut u32;
        for _ in 0..count {
            unsafe {
                ptr.write_unaligned(self.next_u32());
                ptr = ptr.add(1);
            }
        }

        dest[offset..].copy_from_slice(&self.next_u32().to_ne_bytes()[..remainder]);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct JKiss32RngSeed {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub w: u32,
}

impl AsMut<[u8]> for JKiss32RngSeed {
    fn as_mut(&mut self) -> &mut [u8] {
        let len = mem::size_of::<Self>();
        unsafe { slice::from_raw_parts_mut(&mut self.x as *mut u32 as *mut u8, len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;

    #[test]
    fn test_rng() {
        let mut rng = JKiss32Rng::from_seed(JKiss32RngSeed::default());

        let values: [u32; 5] = rng.gen();

        assert_eq!(
            values,
            [1411392427, 2822784854, 4234177281, 1350602412, 2761994839]
        );
    }
}
