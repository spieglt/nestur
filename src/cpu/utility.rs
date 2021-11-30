use super::{CARRY_FLAG, NEGATIVE_FLAG, STACK_OFFSET, ZERO_FLAG, Mode};

impl super::Cpu {

    pub fn advance_pc(&mut self, mode: Mode) {
        self.pc += match mode {
            Mode::ABS => 3,
            Mode::ABX => 3,
            Mode::ABY => 3,
            Mode::ACC => 1,
            Mode::IMM => 2,
            Mode::IMP => 1,
            Mode::IDX => 2,
            Mode::IND => 3,
            Mode::INX => 2,
            Mode::REL => 2,
            Mode::ZPG => 2,
            Mode::ZPX => 2,
            Mode::ZPY => 2,
        }
    }

    pub fn add_offset_to_pc(&mut self, offset: i8) {
        match offset >= 0 {
            true => {
                let decoded_offset = offset as usize;
                self.pc += decoded_offset;
            },
            false => {
                // instr_test-v5/rom_singles/11-stack.nes:
                // letting decoded_offset be (-offset) as usize was allowing for overflow if offset was -128/0b10000000
                let decoded_offset = (-offset) as u8;
                self.pc -= decoded_offset as usize;
            },
        }
    }

    pub fn address_page_cross(&mut self, old_address: usize, new_address: usize) {
        if old_address >> 8 != new_address >> 8 {
            self.clock += 1;
        }
    }

    pub fn branch_page_cross(&mut self, old_address: usize, new_address: usize) {
        if old_address >> 8 != new_address >> 8 {
            self.clock += 1;
        }
    }

    pub fn branch(&mut self, unsigned_offset: u8) {
        let offset = unsigned_offset as i8;
        self.clock += 1;
        let old_addr = self.pc;
        self.add_offset_to_pc(offset);
        let new_addr = self.pc;
        self.branch_page_cross(old_addr, new_addr);
    }

    pub fn compare(&mut self, reg: u8, byte: u8) {
        if reg >= byte {
            self.p |= CARRY_FLAG;
        } else {
            self.p &= 0xFF - CARRY_FLAG;
        }
        self.set_zero_flag(if reg == byte {0} else {1});
        let diff = reg.wrapping_sub(byte);
        self.set_negative_flag(diff);
    }

    pub fn pop(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        let byte = self.read(STACK_OFFSET + self.s as usize);
        byte
    }

    pub fn push(&mut self, byte: u8) {
        self.write(STACK_OFFSET + self.s as usize, byte);
        self.s = self.s.wrapping_sub(1);
    }

    pub fn set_negative_flag(&mut self, num: u8) {
        if num & 0x80 == 0x80 {
            self.p |= NEGATIVE_FLAG;
        } else {
            self.p &= 0xFF - NEGATIVE_FLAG;
        }
    }

    pub fn set_zero_flag(&mut self, num: u8) {
        if num == 0 {
            self.p |= ZERO_FLAG;
        } else {
            self.p &= 0xFF - ZERO_FLAG;
        }
    }

}
