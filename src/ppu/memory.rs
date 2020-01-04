impl super::Ppu {
    
    pub fn read(&mut self, addr: usize) -> u8 {
        let address = addr % 0x4000;
        match addr {
            0x0000..=0x1FFF => {
                if self.pattern_tables.len() > 0 {
                    *(self.mapper_func)(self, address, false).unwrap() // unwrapping because mapper funcs won't return None for reads
                } else {
                    0
                }
            },
            0x2000..=0x3EFF => self.read_nametable(address),
            0x3F00..=0x3FFF => {
                let a = address % 0x0020;
                let value = self.palette_ram[a];
                value
            },
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: usize, value: u8) {
        let address = addr % 0x4000;
        match addr {
            0x0000..=0x1FFF => {
                match (self.mapper_func)(self, address, true) {
                    Some(loc) => *loc = value,
                    None => (),
                }
            },
            0x2000..=0x3EFF => self.write_nametable(address, value),
            0x3F00..=0x3FFF => {
                // I did not read this closely enough for a long time.
                // Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C.
                // Note that this goes for writing as well as reading.
                // A symptom of not having implemented this correctly in an emulator is the sky being black in Super Mario Bros.,
                // which writes the backdrop color through $3F10. 
                match address % 0x10 {
                    0x00 => {
                        self.palette_ram[0] = value;
                        self.palette_ram[0x10] = value;
                    },
                    0x04 => {
                        self.palette_ram[0x04] = value;
                        self.palette_ram[0x14] = value;
                    },
                    0x08 => {
                        self.palette_ram[0x08] = value;
                        self.palette_ram[0x18] = value;
                    },
                    0x0C => {
                        self.palette_ram[0x0C] = value;
                        self.palette_ram[0x1C] = value;
                    },
                    _ => self.palette_ram[address % 0x0020] = value,
                }
            },
            _ => (),
        }
    }

    fn read_nametable(&mut self, address: usize) -> u8 {
        let base = address % 0x1000;
        let offset = base % 0x0400;
        if self.mirroring == 0 { // horizontal
            match base {
                0x0000..=0x07FF => {
                    self.nametable_0[offset]
                },
                0x0800..=0x0FFF => {
                    self.nametable_2[offset]
                },
                _ => panic!("panicked writing nametable base: {}", base),
            }
        } else { // vertical
            match base {
                0x0000..=0x03FF | 0x0800..=0x0BFF => {
                    self.nametable_0[offset]
                },
                0x0400..=0x07FF | 0x0C00..=0x0FFF => {
                    self.nametable_1[offset]
                },
                _ => panic!("panicked writing nametable base: {}", base),
            }
        }
    }

    fn write_nametable(&mut self, address: usize, value: u8) {
        let base = address % 0x1000;
        let offset = base % 0x0400;
        if self.mirroring == 0 { // horizontal
            match base {
                0x0000..=0x07FF => {
                    self.nametable_0[offset] = value;
                    self.nametable_1[offset] = value;
                },
                0x0800..=0x0FFF => {
                    self.nametable_2[offset] = value;
                    self.nametable_3[offset] = value;
                },
                _ => panic!("panicked writing nametable base: {}", base),
            }
        } else { // vertical
            match base {
                0x0000..=0x03FF | 0x0800..=0x0BFF => {
                    self.nametable_0[offset] = value;
                    self.nametable_2[offset] = value;
                },
                0x0400..=0x07FF | 0x0C00..=0x0FFF => {
                    self.nametable_1[offset] = value;
                    self.nametable_3[offset] = value;
                },
                _ => panic!("panicked writing nametable base: {}", base),
            }
        }
    }
}
