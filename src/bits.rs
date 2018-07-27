pub mod big_endian {
    pub fn u64_to_u8(x: u64) -> [u8; 8] {
        [
            ((x >> 56) & 0xff) as u8,
            ((x >> 48) & 0xff) as u8,
            ((x >> 40) & 0xff) as u8,
            ((x >> 32) & 0xff) as u8,
            ((x >> 24) & 0xff) as u8,
            ((x >> 16) & 0xff) as u8,
            ((x >> 8) & 0xff) as u8,
            (x & 0xff) as u8,
        ]
    }

    pub fn u32_to_u8(x: u32) -> [u8; 4] {
        [
            ((x >> 24) & 0xff) as u8,
            ((x >> 16) & 0xff) as u8,
            ((x >> 8) & 0xff) as u8,
            (x & 0xff) as u8,
        ]
    }

    pub fn u16_to_u8(x: u16) -> [u8; 2] {
        [((x >> 8) & 0xff) as u8, (x & 0xff) as u8]
    }

    pub fn u8_to_u32(x: [u8; 4]) -> u32 {
        (x[0] as u32) << 24 | (x[1] as u32) << 16 | (x[2] as u32) << 8 | (x[3] as u32)
    }

    pub fn u8_slice_to_u32(x: &[u8]) -> u32 {
        u8_to_u32([x[0], x[1], x[2], x[3]])
    }

    pub fn u8_to_u16(x: [u8; 2]) -> u16 {
        (x[0] as u16) << 8 | (x[1] as u16)
    }

    pub fn u8_slice_to_u16(x: &[u8]) -> u16 {
        u8_to_u16([x[0], x[1]])
    }

    pub fn u8_slice_to_usize(x: &[u8]) -> usize {
        let mut res = 0 as usize;
        for &e in x {
            res <<= 8;
            res += e as usize;
        }
        return res;
    }
}

pub mod little_endian {
    pub fn u16_to_u8(x: u16) -> [u8; 2] {
        [(x & 0xff) as u8, ((x >> 8) & 0xff) as u8]
    }
}
