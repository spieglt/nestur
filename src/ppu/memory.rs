use crate::cartridge::Mirror;

impl super::Ppu {

    pub fn read(&mut self, address: usize) -> u8 {
        match address {
            0x0000..=0x1FFF => self.mapper.borrow_mut().read(address),
            0x2000..=0x3EFF => self.read_nametable(address),
            0x3F00..=0x3FFF => {
                let a = address % 0x0020;
                let value = self.palette_ram[a];
                value
            },
            _ => 0,
        }
    }

    pub fn write(&mut self, address: usize, value: u8) {
        // let address = addr % 0x4000;
        match address {
            0x0000..=0x1FFF => self.mapper.borrow_mut().write(address, value),
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
        match self.mapper.borrow_mut().get_mirroring() {
            Mirror::LowBank => self.nametable_A[offset],
            Mirror::HighBank => self.nametable_B[offset],
            Mirror::Horizontal => {
                match base {
                    0x0000..=0x07FF => self.nametable_A[offset],
                    0x0800..=0x0FFF => self.nametable_B[offset],
                    _ => panic!("panicked reading nametable base: {}", base),
                }
            },
            Mirror::Vertical => {
                match base {
                    0x0000..=0x03FF | 0x0800..=0x0BFF => self.nametable_A[offset],
                    0x0400..=0x07FF | 0x0C00..=0x0FFF => self.nametable_B[offset],
                    _ => panic!("panicked reading nametable base: {}", base),
                }
            },
            Mirror::FourScreen => {
                match base {
                    0x0000..=0x03FF => self.nametable_A[offset],
                    0x0400..=0x07FF => self.nametable_B[offset],
                    0x0800..=0x0BFF => self.nametable_C[offset],
                    0x0C00..=0x0FFF => self.nametable_D[offset],
                    _ => panic!("panicked reading nametable base: {}", base),
                }
            },
        }
    }

    fn write_nametable(&mut self, address: usize, value: u8) {
        let base = address % 0x1000;
        let offset = base % 0x0400;
        match self.mapper.borrow_mut().get_mirroring() {
            Mirror::LowBank => self.nametable_A[offset] = value,
            Mirror::HighBank => self.nametable_B[offset] = value,
            Mirror::Horizontal => {
                match base {
                    0x0000..=0x07FF => self.nametable_A[offset] = value,
                    0x0800..=0x0FFF => self.nametable_B[offset] = value,
                    _ => panic!("panicked writing nametable base: {}", base),
                }
            },
            Mirror::Vertical => {
                match base {
                    0x0000..=0x03FF | 0x0800..=0x0BFF => self.nametable_A[offset] = value,
                    0x0400..=0x07FF | 0x0C00..=0x0FFF => self.nametable_B[offset] = value,
                    _ => panic!("panicked writing nametable base: {}", base),
                }
            },
            Mirror::FourScreen => {
                match base {
                    0x0000..=0x03FF => self.nametable_A[offset] = value,
                    0x0400..=0x07FF => self.nametable_B[offset] = value,
                    0x0800..=0x0BFF => self.nametable_C[offset] = value,
                    0x0C00..=0x0FFF => self.nametable_D[offset] = value,
                    _ => panic!("panicked writing nametable base: {}", base),
                }
            },
        }
    }
}
