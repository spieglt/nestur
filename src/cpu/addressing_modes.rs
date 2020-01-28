impl super::Cpu {
    pub fn absolute(&mut self) -> usize {
        self.clock += 4;
        <usize>::from(
            ((self.read(self.PC + 2) as usize) << 8) + // high byte, little endian
            (self.read(self.PC + 1)) as usize, // low byte
        )
    }

    pub fn absolute_x(&mut self) -> usize {
        let current_opcode = self.read(self.PC);
        let old_address = self.absolute();
        let new_address = old_address + self.X as usize;
        match current_opcode {
            0x1C | 0x1D | 0x3C | 0x3D | 0x5C | 0x5D | 0x7C | 0x7D | 0xBC | 0xBD | 0xDC | 0xDD
            | 0xFC | 0xFD => self.address_page_cross(old_address, new_address),
            0x1E | 0x1F | 0x3E | 0x3F | 0x5E | 0x5F | 0x7E | 0x7F | 0x9D | 0xC3 | 0xC7 | 0xCF
            | 0xD3 | 0xD7 | 0xDB | 0xDE | 0xDF | 0xFE | 0xFF => self.clock += 1,
            _ => panic!("illegal opcode using abs x: {:02x}", current_opcode),
        }
        new_address
    }

    pub fn absolute_y(&mut self) -> usize {
        let current_opcode = self.PC;
        let old_address = self.absolute() as u16; // coerce to u16 for wrapping addition
        let new_address = old_address.wrapping_add(self.Y as u16);
        let old_address = old_address as usize; // coerce back
        let new_address = new_address as usize;
        if current_opcode == 0x99 {
            self.clock += 1;
        } else {
            self.address_page_cross(old_address, new_address);
        }
        new_address
    }

    pub fn accumulator(&mut self) -> usize {
        self.clock += 2;
        0
    }

    pub fn immediate(&mut self) -> usize {
        self.clock += 2;
        self.PC + 1
    }

    pub fn implied(&mut self) -> usize {
        self.clock += 2;
        0
    }

    pub fn indexed_indirect(&mut self) -> usize {
        self.clock += 6;
        let operand = self.read(self.PC + 1);
        let zp_low_addr = operand.wrapping_add(self.X);
        let zp_high_addr = zp_low_addr.wrapping_add(1); // take account of zero page wraparound
        let zp_low_byte = self.read(zp_low_addr as usize);
        let zp_high_byte = self.read(zp_high_addr as usize);
        ((zp_high_byte as usize) << 8) + zp_low_byte as usize
    }

    pub fn indirect(&mut self) -> usize {
        let operand_address =
            ((self.read(self.PC + 2) as usize) << 8) + (self.read(self.PC + 1) as usize);
        let low_byte = self.read(operand_address) as usize;
        // BUG TIME! from https://wiki.nesdev.com/w/index.php/Errata
        // "JMP ($xxyy), or JMP indirect, does not advance pages if the lower eight bits
        // of the specified address is $FF; the upper eight bits are fetched from $xx00,
        // 255 bytes earlier, instead of the expected following byte."
        let high_byte = if operand_address & 0xFF == 0xFF {
            (self.read(operand_address as usize - 0xFF) as usize) << 8
        } else {
            (self.read(operand_address as usize + 1) as usize) << 8
        };
        let real_address = high_byte + low_byte;
        self.clock += 5;
        real_address
    }

    pub fn indirect_indexed(&mut self) -> usize {
        let operand = self.read(self.PC + 1);
        let zp_low_addr = operand;
        let zp_high_addr = operand.wrapping_add(1);
        let zp_low_byte = self.read(zp_low_addr as usize);
        let zp_high_byte = self.read(zp_high_addr as usize);
        let old_address = ((zp_high_byte as u16) << 8) + zp_low_byte as u16;
        let new_address = old_address.wrapping_add(self.Y as u16);
        if self.PC == 0xF1 {
            self.clock += 1;
        } else {
            self.address_page_cross(old_address as usize, new_address as usize);
        }
        self.clock += 5;
        new_address as usize
    }

    pub fn relative(&mut self) -> usize {
        self.clock += 2;
        self.PC + 1
    }

    pub fn zero_page(&mut self) -> usize {
        let operand = self.read(self.PC + 1);
        self.clock += 3;
        operand as usize
    }

    pub fn zero_page_x(&mut self) -> usize {
        let operand = self.read(self.PC + 1);
        self.clock += 4;
        operand.wrapping_add(self.X) as usize
    }

    pub fn zero_page_y(&mut self) -> usize {
        let operand = self.read(self.PC + 1);
        self.clock += 4;
        operand.wrapping_add(self.Y) as usize
    }
}
