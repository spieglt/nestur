use super::{CARRY_FLAG, DECIMAL_FLAG, INTERRUPT_DISABLE_FLAG, IRQ_VECTOR, NEGATIVE_FLAG, NMI_VECTOR, OVERFLOW_FLAG, ZERO_FLAG, Mode};

// TODO: check unofficial opcodes for page crosses

impl super::Cpu {

    pub fn adc(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        let carry_bit = if self.p & CARRY_FLAG == 0 {0} else {1};
        let mut new_val = self.a.wrapping_add(byte); // add the byte at the _address to accum
        new_val = new_val.wrapping_add(carry_bit); // add carry flag to accumulator
        // set carry flag if we wrapped around and added something
        if new_val <= self.a && (byte != 0 || carry_bit != 0) {
            self.p |= CARRY_FLAG;
        } else {
            self.p &= 0xFF - CARRY_FLAG;
        }
        self.set_zero_flag(new_val);
        self.set_negative_flag(new_val);
        // signed 8-bit overflow can only happen if both signs were positive but result was negative, or if both signs were negative and the result was positive
        // sign is positive if num & 0x80 == 0, negative if num & 0x80 != 0
        // ((sum & 0x80 != 0) && (acc & 0x80 == 0) && (operand & 0x80 == 0)) || ((sum & 0x80 == 0) && (acc & 0x80 != 0) && (operand & 0x80 != 0))
        // simplifies to the below, thanks http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        if (byte ^ new_val) & (self.a ^ new_val) & 0x80 != 0 {
            self.p |= OVERFLOW_FLAG;
        } else {
            self.p &= 0xFF - OVERFLOW_FLAG;
        }
        self.a = new_val; // actually change the accumulator
    }

    pub fn and(&mut self, _address: usize, _mode: Mode) {
        self.a &= self.read(_address);
        self.set_zero_flag(self.a);
        self.set_negative_flag(self.a);
    }

    pub fn asl(&mut self, _address: usize, _mode: Mode) {
        let mut val = match _mode {
            Mode::ACC => self.a,
            _ => {
                self.clock += 2;
                self.read(_address)
            },
        };
        // put top bit in carry flag
        if val & (1<<7) != 0 {
            self.p |= CARRY_FLAG;
        } else {
            self.p &= 0xFF - CARRY_FLAG;
        }
        val <<= 1;
        match _mode {
            Mode::ACC => self.a = val,
            _ => self.write(_address, val),
        };
        self.set_zero_flag(val);
        self.set_negative_flag(val);
    }

    pub fn bcc(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        if self.p & CARRY_FLAG == 0 {
            self.branch(byte);
        }
    }

    pub fn bcs(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        if self.p & CARRY_FLAG != 0 {
            self.branch(byte);
        }
    }

    pub fn beq(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        if self.p & ZERO_FLAG != 0 {
            self.branch(byte);
        }
    }

    pub fn bit(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        let tested = byte & self.a;
        self.set_zero_flag(tested);
        let bit6 = byte & (1 << 6);
        if bit6 != 0 {
            self.p |= OVERFLOW_FLAG;
        } else {
            self.p &= 0xFF - OVERFLOW_FLAG;
        }
        let bit7 = byte & (1 << 7);
        if bit7 != 0 {
            self.p |= NEGATIVE_FLAG;
        } else {
            self.p &= 0xFF - NEGATIVE_FLAG;
        }
    }

    pub fn bmi(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        if self.p & NEGATIVE_FLAG != 0 {
            self.branch(byte);
        }
    }

    pub fn bne(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        if self.p & ZERO_FLAG == 0 {
            self.branch(byte);
        }
    }

    pub fn bpl(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        if self.p & NEGATIVE_FLAG == 0 {
            self.branch(byte);
        }
    }

    pub fn brk(&mut self, _address: usize, _mode: Mode) {
        // instr_test-v5/rom_singles/15-brk.nes and instr_test-v5/rom_singles/16-special.nes:
        // using self.pc + 1 in these next two lines allows these tests to pass.
        // I'm not sure why that's necessary as implied addressing mode is only supposed to consume 1 byte,
        // but the error message from 16-special.nes said "BRK should push address BRK + 2"

        // Aha! From http://nesdev.com/the%20%27B%27%20flag%20&%20BRK%20instruction.txt:
        // Regardless of what ANY 6502 documentation says, BRK is a 2 byte opcode. The
        // first is #$00, and the second is a padding byte. This explains why interrupt
        // routines called by BRK always return 2 bytes after the actual BRK opcode,
        // and not just 1.

        self.push(((self.pc + 1) >> 8) as u8); // push high byte
        self.push(((self.pc + 1) & 0xFF) as u8); // push low byte
        self.push(self.p | 0b00110000); // push status register with break bits set
        self.p |= INTERRUPT_DISABLE_FLAG; // set interrupt disable flag
        self.pc = ((self.read(IRQ_VECTOR + 1) as usize) << 8) // set program counter to IRQ/BRK vector, taking high byte
            + (self.read(IRQ_VECTOR) as usize); // and low byte
        self.clock += 5; // total of 7 cycles, 2 come from implied()
    }

    pub fn bvc(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        if self.p & OVERFLOW_FLAG == 0 {
            self.branch(byte);
        }
    }

    pub fn bvs(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        if self.p & OVERFLOW_FLAG != 0 {
            self.branch(byte);
        }
    }

    pub fn clc(&mut self, _address: usize, _mode: Mode) {
        self.p &= 0xFF - CARRY_FLAG;
    }

    pub fn cld(&mut self, _address: usize, _mode: Mode) {
        self.p &= 0xFF - DECIMAL_FLAG;
    }

    pub fn cli(&mut self, _address: usize, _mode: Mode) {
        self.p &= 0xFF - INTERRUPT_DISABLE_FLAG;
    }

    pub fn clv(&mut self, _address: usize, _mode: Mode) {
        self.p &= 0xFF - OVERFLOW_FLAG;
    }

    pub fn cmp(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        self.compare(self.a, byte);
    }

    pub fn cpx(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        self.compare(self.x, byte);
    }

    pub fn cpy(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        self.compare(self.y, byte);
    }

    pub fn dcp(&mut self, _address: usize, _mode: Mode) {
        // unofficial
        let val = self.read(_address).wrapping_sub(1);
        self.write(_address, val);
        self.compare(self.a, val);
    }

    pub fn dec(&mut self, _address: usize, _mode: Mode) {
        let val = self.read(_address).wrapping_sub(1);
        self.write(_address, val);
        self.set_zero_flag(val);
        self.set_negative_flag(val);
        self.clock += 2; // extra cycles for all addressing modes of this instruction
    }

    pub fn dex(&mut self, _address: usize, _mode: Mode) {
        self.x = self.x.wrapping_sub(1);
        self.set_zero_flag(self.x);
        self.set_negative_flag(self.x);
    }

    pub fn dey(&mut self, _address: usize, _mode: Mode) {
        self.y = self.y.wrapping_sub(1);
        self.set_zero_flag(self.y);
        self.set_negative_flag(self.y);
    }

    pub fn eor(&mut self, _address: usize, _mode: Mode) {
        self.a ^= self.read(_address);
        self.set_negative_flag(self.a);
        self.set_zero_flag(self.a);
    }

    pub fn inc(&mut self, _address: usize, _mode: Mode) {
        let val = self.read(_address).wrapping_add(1);
        self.write(_address, val);
        self.set_zero_flag(val);
        self.set_negative_flag(val);
        self.clock += 2; // extra cycles for all addressing modes of this instruction
    }

    pub fn isc(&mut self, _address: usize, _mode: Mode) {
        // unofficial
        self.inc(_address, _mode);
        self.sbc(_address, _mode);
    }

    pub fn inx(&mut self, _address: usize, _mode: Mode) {
        self.x = self.x.wrapping_add(1);
        self.set_zero_flag(self.x);
        self.set_negative_flag(self.x);
    }

    pub fn iny(&mut self, _address: usize, _mode: Mode) {
        self.y = self.y.wrapping_add(1);
        self.set_zero_flag(self.y);
        self.set_negative_flag(self.y);
    }

    pub fn jmp(&mut self, _address: usize, _mode: Mode) {
        if _mode == Mode::ABS {
            self.clock -= 1; // this only takes 3..
        }
        self.pc = _address;
    }

    pub fn jsr(&mut self, _address: usize, _mode: Mode) {
        // call to absolute already advances program counter by 3
        self.clock += 2;
        let minus1 = self.pc - 1; // so m1 is the last _byte of the jsr instruction. second _byte of the operand.
        self.push((minus1 >> 8) as u8);
        self.push((minus1 & 0xFF) as u8);
        self.pc = _address;
    }

    pub fn lax(&mut self, _address: usize, _mode: Mode) {
        // unofficial opcode that sets both X and accumulator
        // TODO: check cycle count? https://wiki.nesdev.com/w/index.php/Programming_with_unofficial_opcodes
        let byte = self.read(_address);
        self.a = byte;
        self.x = byte;
        self.set_zero_flag(byte);
        self.set_negative_flag(byte);
    }

    pub fn lda(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        self.a = byte;
        self.set_zero_flag(byte);
        self.set_negative_flag(byte);
    }

    pub fn ldx(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        self.x = byte;
        self.set_zero_flag(byte);
        self.set_negative_flag(byte);
    }

    pub fn ldy(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        self.y = byte;
        self.set_zero_flag(byte);
        self.set_negative_flag(byte);
    }

    pub fn lsr(&mut self, _address: usize, _mode: Mode) {
        let mut val = match _mode {
            Mode::ACC => self.a,
            _ => {
                self.clock += 2;
                self.read(_address)
            },
        };
        if val & 0x1 == 0x1 {
            self.p |= CARRY_FLAG;
        } else {
            self.p &= 0xFF - CARRY_FLAG;
        }
        val >>= 1;
        match _mode {
            Mode::ACC => self.a = val,
            _ => self.write(_address, val),
        };
        self.set_zero_flag(val);
        self.set_negative_flag(val);
    }

    pub fn nop(&mut self, _address: usize, _mode: Mode) {
    }

    pub fn ora(&mut self, _address: usize, _mode: Mode) {
        self.a |= self.read(_address);
        self.set_zero_flag(self.a);
        self.set_negative_flag(self.a);
    }

    pub fn pha(&mut self, _address: usize, _mode: Mode) {
        self.clock += 1;
        self.push(self.a);
    }

    pub fn php(&mut self, _address: usize, _mode: Mode) {
        self.clock += 1;
        self.push(self.p | 0b00110000);
    }

    pub fn pla(&mut self, _address: usize, _mode: Mode) {
        self.clock += 2;
        self.a = self.pop();
        self.set_zero_flag(self.a);
        self.set_negative_flag(self.a);
    }

    pub fn plp(&mut self, _address: usize, _mode: Mode) {
        self.clock += 2;
        self.p = self.pop();
        // TODO: figure out exactly what's supposed to happen here
        // let status = self.pop();
        // // for each bit in the popped status, if it's 1,
        // // set that bit of self.p to 1. if it's 0, set that
        // // bit of self.p to 0.
        // for i in 0..=7 {
        //     if i == 4 || i == 5 {
        //         continue; // ignore B flags
        //     }
        //     let bit = if status & (1 << i) == 0 {0} else {1};
        //     if bit != 0 {
        //         self.p |= 1 << i;
        //     } else {
        //         self.p &= 0xFF - (1 << i);
        //     }
        // }
        // self.p |= 1 << 5; // turn on bit 5
        // self.p &= 0xFF - (1 << 4); // and turn off bit 4 because god knows why
    }

    pub fn rla(&mut self, _address: usize, _mode: Mode) {
        // unofficial
        self.rol(_address, _mode);
        self.and(_address, _mode);
    }

    pub fn rol(&mut self, _address: usize, _mode: Mode) {
        let mut val = match _mode {
            Mode::ACC => self.a,
            _ => {
                self.clock += 2;
                self.read(_address)
            },
        };
        let carry_flag_bit = if self.p & CARRY_FLAG != 0 {1} else {0};
        let new_cfb = if val & 0x80 != 0 {1} else {0};
        val <<= 1;
        val += carry_flag_bit;
        match _mode {
            Mode::ACC => self.a = val,
            _ => self.write(_address, val),
        };
        if new_cfb != 0 { self.p |= CARRY_FLAG; }
        else { self.p &= 0xFF - CARRY_FLAG; }
        self.set_zero_flag(val);
        self.set_negative_flag(val);
    }

    pub fn ror(&mut self, _address: usize, _mode: Mode) {
        let mut val = match _mode {
            Mode::ACC => self.a,
            _ => {
                self.clock += 2; // extra cycles
                self.read(_address)
            }
        };
        let cfb = if self.p & CARRY_FLAG != 0 {1} else {0};
        let new_cfb = val & 0x1;
        val >>= 1;
        val += cfb * 0x80;
        if new_cfb != 0 { self.p |= CARRY_FLAG; }
        else { self.p &= 0xFF - CARRY_FLAG; }
        match _mode {
            Mode::ACC => self.a = val,
            _ => self.write(_address, val),
        };
        self.set_zero_flag(val);
        self.set_negative_flag(val);
    }

    pub fn rra(&mut self, _address: usize, _mode: Mode) {
        // unofficial
        self.ror(_address, _mode);
        self.adc(_address, _mode);
    }

    pub fn rti(&mut self, _address: usize, _mode: Mode) {
        self.plp(_address, _mode); // pull and set status reg (2 clock cycles)
        self.pc = self.pop() as usize; // low byte
        self.pc += (self.pop() as usize) << 8; // high byte
        self.clock += 2; // +2 from implied
    }

    pub fn rts(&mut self, _address: usize, _mode: Mode) {
        self.pc = self.pop() as usize;
        self.pc += ((self.pop() as usize) << 8) + 1;
        self.clock += 4;
    }

    pub fn sax(&mut self, _address: usize, _mode: Mode) {
        // unofficial combo of stx and sta
        self.write(_address, self.a & self.x);
    }

    pub fn sbc(&mut self, _address: usize, _mode: Mode) {
        let byte = self.read(_address);
        let carry_bit = if self.p & CARRY_FLAG == 0 {1} else {0};
        let mut new_val = self.a.wrapping_sub(byte);
        new_val = new_val.wrapping_sub(carry_bit);
        // if overflow occurs and we subtracted something, CLEAR the carry bit
        if new_val >= self.a && (byte != 0 || carry_bit != 0) {
            self.p &= 0xFF - CARRY_FLAG;
        } else {
            self.p |= CARRY_FLAG;
        }
        self.set_zero_flag(new_val);
        self.set_negative_flag(new_val);
        // if acc is positive, mem is negative, and result is negative
        // or if acc is negative, mem is positive, and result is positive
        let acc = self.a & 0x80 == 0;
        let mem = byte & 0x80 == 0;
        let res = new_val & 0x80 == 0;
        // if sign is wrong, SET overflow flag
        if (acc && !mem && !res) || (!acc && mem && res) {
            self.p |= OVERFLOW_FLAG;
        } else {
            self.p &= 0xFF - OVERFLOW_FLAG;
        }
        self.a = new_val; // actually change the accumulator
    }

    pub fn sec(&mut self, _address: usize, _mode: Mode) {
        self.p |= CARRY_FLAG;
    }

    pub fn sed(&mut self, _address: usize, _mode: Mode) {
        self.p |= DECIMAL_FLAG; // don't think this is necessary since the NES's 6502 doesn't have decimal _mode but whatever
    }

    pub fn sei(&mut self, _address: usize, _mode: Mode) {
        self.p |= INTERRUPT_DISABLE_FLAG;
    }

    pub fn slo(&mut self, _address: usize, _mode: Mode) {
        self.asl(_address, _mode);
        self.ora(_address, _mode);
        // can get away with ignoring that asl handles accumulator addressing mode because slo doesn't handle accumulator addressing mode.
    }

    pub fn sre(&mut self, _address: usize, _mode: Mode) {
        // unofficial
        self.lsr(_address, _mode);
        self.eor(_address, _mode);
    }

    pub fn sta(&mut self, _address: usize, _mode: Mode) {
        // PPU Test 17
        // STA, $2000,Y **must** issue a dummy read to 2007
        if _address == 0x2007 && _mode == Mode::ABY && self.y == 7 {
            self.read(0x2007);
        }

        if _mode == Mode::INY {
            self.clock = self.before_clock + 6; // Special
        } else if _mode == Mode::ABY {
            self.clock = self.before_clock + 5; // Special
        }
        self.write(_address, self.a);
    }

    pub fn stx(&mut self, _address: usize, _mode: Mode) {
        self.write(_address, self.x);
    }

    pub fn sty(&mut self, _address: usize, _mode: Mode) {
        self.write(_address, self.y);
    }

    pub fn tax(&mut self, _address: usize, _mode: Mode) {
        self.x = self.a;
        self.set_zero_flag(self.x);
        self.set_negative_flag(self.x);
    }

    pub fn tay(&mut self, _address: usize, _mode: Mode) {
        self.y = self.a;
        self.set_zero_flag(self.y);
        self.set_negative_flag(self.y);
    }

    pub fn tsx(&mut self, _address: usize, _mode: Mode) {
        self.x = self.s;
        self.set_zero_flag(self.x);
        self.set_negative_flag(self.x);
    }

    pub fn txa(&mut self, _address: usize, _mode: Mode) {
        self.a = self.x;
        self.set_zero_flag(self.a);
        self.set_negative_flag(self.a);
    }

    pub fn txs(&mut self, _address: usize, _mode: Mode) {
        self.s = self.x;
    }

    pub fn tya(&mut self, _address: usize, _mode: Mode) {
        self.a = self.y;
        self.set_zero_flag(self.a);
        self.set_negative_flag(self.a);
    }

    pub fn bad(&mut self, _address: usize, _mode: Mode) {
        panic!("illegal opcode: 0x{:02X}", self.read(self.pc)); // this won't be the illegal opcode because the PC somehow hasn't been updated yet
    }

    // Interrupts
    pub fn nmi(&mut self) {
        self.push((self.pc >> 8) as u8); // push high byte
        self.push((self.pc & 0xFF) as u8); // push low byte
        self.push(self.p | 0b00110000); // push status register with break bits set
        self.p |= INTERRUPT_DISABLE_FLAG; // set interrupt disable flag
        self.pc = ((self.read(NMI_VECTOR + 1) as usize) << 8) // set program counter to NMI vector, taking high byte
            + (self.read(NMI_VECTOR) as usize); // and low byte
        self.clock += 7;
    }

    pub fn irq(&mut self) {
        self.push((self.pc >> 8) as u8); // push high byte
        self.push((self.pc & 0xFF) as u8); // push low byte
        self.push(self.p & 0b11001111); // push status register with break bits cleared
        self.p |= INTERRUPT_DISABLE_FLAG; // set interrupt disable flag
        self.pc = ((self.read(IRQ_VECTOR + 1) as usize) << 8) // set program counter to IRQ/BRK vector, taking high byte
            + (self.read(IRQ_VECTOR) as usize); // and low byte
        self.clock += 7;
    }

}
