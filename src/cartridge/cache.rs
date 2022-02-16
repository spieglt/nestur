
pub struct Cache {
    pub data_lines: [[u8; 64]; 2048], // 128KB cache, 2048 lines of 64 bytes
    pub dirty: [u64; 2048 / 64], // one bit per data line
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            data_lines: [[0u8; 64]; 0x800],
            dirty: [!0; 2048 / 64], // mark all lines as dirty
        }
    }

    // pub fn read(&self, addr: usize) -> u8 {
    // }

    pub fn is_dirty(&self, bucket: usize) -> bool {
        let byte = self.dirty[bucket / 64];
        let offset = bucket % 64;
        (byte & (1 << offset)) != 0
    }

    pub fn mark_clean(&mut self, bucket: usize) {
        let offset = bucket % 64;
        self.dirty[bucket / 64] &= !(1 << offset);
    }

    pub fn mark_dirty(&mut self, bucket: usize) {
        self.dirty[bucket / 64] |= 1 << (bucket % 64);
    }
}
