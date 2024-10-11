use crate::combine;
use crate::table::CRC32_TABLE;

#[derive(Clone)]
pub struct State {
    state: u32,
}

impl State {
    pub const fn new(state: u32) -> Self {
        State { state }
    }

    pub fn update(&mut self, buf: &[u8]) {
        self.state = update_fast_16(self.state, buf);
    }

    pub const fn finalize(self) -> u32 {
        self.state
    }

    pub fn reset(&mut self) {
        self.state = 0;
    }

    pub fn combine(&mut self, other: u32, amount: u64) {
        self.state = combine::combine(self.state, other, amount);
    }
}

pub fn update_fast_16(prev: u32, mut buf: &[u8]) -> u32 {
    const UNROLL: usize = 8;
    const BYTES_AT_ONCE: usize = 16 * UNROLL;

    let mut crc = !prev;

    while buf.len() >= BYTES_AT_ONCE {
        for i in 0..UNROLL {
            let offset = i * 16;
            crc = CRC32_TABLE[0x0][buf[offset + 0xf] as usize]
                ^ CRC32_TABLE[0x1][buf[offset + 0xe] as usize]
                ^ CRC32_TABLE[0x2][buf[offset + 0xd] as usize]
                ^ CRC32_TABLE[0x3][buf[offset + 0xc] as usize]
                ^ CRC32_TABLE[0x4][buf[offset + 0xb] as usize]
                ^ CRC32_TABLE[0x5][buf[offset + 0xa] as usize]
                ^ CRC32_TABLE[0x6][buf[offset + 0x9] as usize]
                ^ CRC32_TABLE[0x7][buf[offset + 0x8] as usize]
                ^ CRC32_TABLE[0x8][buf[offset + 0x7] as usize]
                ^ CRC32_TABLE[0x9][buf[offset + 0x6] as usize]
                ^ CRC32_TABLE[0xa][buf[offset + 0x5] as usize]
                ^ CRC32_TABLE[0xb][buf[offset + 0x4] as usize]
                ^ CRC32_TABLE[0xc][buf[offset + 0x3] as usize ^ ((crc >> 0x18) & 0xFF) as usize]
                ^ CRC32_TABLE[0xd][buf[offset + 0x2] as usize ^ ((crc >> 0x10) & 0xFF) as usize]
                ^ CRC32_TABLE[0xe][buf[offset + 0x1] as usize ^ ((crc >> 0x08) & 0xFF) as usize]
                ^ CRC32_TABLE[0xf][buf[offset + 0x0] as usize ^ (crc & 0xFF) as usize];
            buf = &buf[BYTES_AT_ONCE..];
        }
    }

    update_slow(!crc, buf)
}

pub fn update_slow(prev: u32, buf: &[u8]) -> u32 {
    let mut crc = !prev;

    for &byte in buf {
        crc = CRC32_TABLE[0][((crc as u8) ^ byte) as usize] ^ (crc >> 8);
    }

    !crc
}

#[cfg(test)]
mod test {
    #[test]
    fn slow() {
        assert_eq!(super::update_slow(0, b""), 0);

        // test vectors from the iPXE project (input and output are bitwise negated)
        assert_eq!(super::update_slow(!0x12345678, b""), !0x12345678);
        assert_eq!(super::update_slow(!0xffffffff, b"hello world"), !0xf2b5ee7a);
        assert_eq!(super::update_slow(!0xffffffff, b"hello"), !0xc9ef5979);
        assert_eq!(super::update_slow(!0xc9ef5979, b" world"), !0xf2b5ee7a);

        // Some vectors found on Rosetta code
        assert_eq!(super::update_slow(0, b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"), 0x190A55AD);
        assert_eq!(super::update_slow(0, b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"), 0xFF6CAB0B);
        assert_eq!(super::update_slow(0, b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F"), 0x91267E8A);
    }

    quickcheck! {
        fn fast_16_is_the_same_as_slow(crc: u32, bytes: Vec<u8>) -> bool {
            super::update_fast_16(crc, &bytes) == super::update_slow(crc, &bytes)
        }
    }
}
