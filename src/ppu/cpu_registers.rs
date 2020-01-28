impl super::Ppu {
    // cpu writes to 0x2000, PPUCTRL
    pub fn write_controller(&mut self, byte: u8) {
        // VRAM address increment per CPU read/write of PPUDATA
        self.address_increment = match byte & (1 << 2) == 0 {
            // (0: add 1, going across; 1: add 32, going down)
            true => 1,
            false => 32,
        };
        // Sprite pattern table address for 8x8 sprites
        self.sprite_pattern_table_base = match byte & (1 << 3) == 0 {
            true => 0x0,
            false => 0x1000,
        };
        // Background pattern table address
        self.background_pattern_table_base = match byte & (1 << 4) == 0 {
            true => 0x0,
            false => 0x1000,
        };
        // Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
        self.sprite_size = if byte & (1 << 5) != 0 { 16 } else { 8 };
        // Ignoring PPU master/slave select for now
        self.should_generate_nmi = byte & (1 << 7) != 0;
        self.nmi_change();
        // Take care of t
        set_bit(&mut self.t, 10, byte as u16, 0);
        set_bit(&mut self.t, 11, byte as u16, 1);
    }

    // cpu writes to 0x2001, PPUMASK
    pub fn write_mask(&mut self, byte: u8) {
        self.grayscale = byte & (1 << 0) != 0;
        self.show_background_left = byte & (1 << 1) != 0;
        self.show_sprites_left = byte & (1 << 2) != 0;
        self.show_background = byte & (1 << 3) != 0;
        self.show_sprites = byte & (1 << 4) != 0;
        self.emphasize_red = byte & (1 << 5) != 0;
        self.emphasize_blue = byte & (1 << 6) != 0;
        self.emphasize_green = byte & (1 << 7) != 0;
    }

    // cpu reads ppu status from 0x2002, PPUSTATUS
    pub fn read_status(&mut self) -> u8 {
        let mut byte = self.recent_bits & 0b0001_1111;
        byte |= if self.sprite_overflow { 0b0010_0000 } else { 0 };
        byte |= if self.sprite_zero_hit { 0b0100_0000 } else { 0 };
        byte |= if self.vertical_blank { 0b1000_0000 } else { 0 };
        self.w = 0;
        self.vertical_blank = false;
        self.nmi_change();
        byte
    }

    // cpu writes to 0x2003, OAMADDR
    pub fn write_oam_address(&mut self, addr: usize) {
        self.oam_address = addr;
    }

    // cpu reads from 0x2004, OAMDATA
    pub fn read_oam_data(&mut self) -> u8 {
        self.primary_oam[self.oam_address]
    }

    // cpu writes to 0x2004, OAMDATA
    pub fn write_oam_data(&mut self, val: u8) {
        // Writes will increment OAMADDR after the write
        self.primary_oam[self.oam_address] = val;
        self.oam_address += 1;
    }

    // cpu writes to 0x2005, PPUSCROLL
    pub fn write_scroll(&mut self, val: u8) {
        match self.w {
            // first write
            0 => {
                // t: ....... ...HGFED = d: HGFED...
                self.t &= !((1 << 5) - 1); // turn off bottom 5 bits of t
                self.t |= val as u16 >> 3; // set bottom 5 bits of t to top 5 bits of d
                                           // x:              CBA = d: .....CBA
                self.x = val & ((1 << 3) - 1);
                self.w = 1;
            }
            1 => {
                // second write
                let d = val as u16;
                // t: CBA..HG FED..... = d: HGFEDCBA
                set_bit(&mut self.t, 0xC, d, 0x0);
                set_bit(&mut self.t, 0xD, d, 0x1);
                set_bit(&mut self.t, 0xE, d, 0x2);
                set_bit(&mut self.t, 0x5, d, 0x3);
                set_bit(&mut self.t, 0x6, d, 0x4);
                set_bit(&mut self.t, 0x7, d, 0x5);
                set_bit(&mut self.t, 0x8, d, 0x6);
                set_bit(&mut self.t, 0x9, d, 0x7);
                self.w = 0;
            }
            _ => panic!("uh oh, somehow w was incremented past 1 to {}", self.w),
        }
    }

    // cpu writes to 0x2006, PPUADDR
    pub fn write_address(&mut self, val: u8) {
        self.mapper.borrow_mut().clock();
        let d = val as u16;
        match self.w {
            // first write
            0 => {
                // t: .FEDCBA ........ = d: ..FEDCBA
                set_bit(&mut self.t, 0x8, d, 0x0);
                set_bit(&mut self.t, 0x9, d, 0x1);
                set_bit(&mut self.t, 0xA, d, 0x2);
                set_bit(&mut self.t, 0xB, d, 0x3);
                set_bit(&mut self.t, 0xC, d, 0x4);
                set_bit(&mut self.t, 0xD, d, 0x5);
                // t: X...... ........ = 0
                set_bit(&mut self.t, 0xF, 0, 0);
                self.w = 1;
            }
            1 => {
                // second write
                // t: ....... HGFEDCBA = d: HGFEDCBA
                self.t &= 0xFF00; // mask off bottom byte
                self.t += d; // apply d
                self.v = self.t; // After t is updated, contents of t copied into v
                self.w = 0;
            }
            _ => panic!("uh oh, somehow w was incremented past 1 to {}", self.w),
        }
    }

    // cpu reads from 0x2007, PPUDATA
    pub fn read_data(&mut self) -> u8 {
        /*
        The PPUDATA read buffer (post-fetch)
        When reading while the VRAM address is in the range 0-$3EFF (i.e., before the palettes),
        the read will return the contents of an internal read buffer. This internal buffer is
        updated only when reading PPUDATA, and so is preserved across frames. After the CPU reads
        and gets the contents of the internal buffer, the PPU will immediately update the internal
        buffer with the byte at the current VRAM address. Thus, after setting the VRAM address,
        one should first read this register and discard the result.
        Reading palette data from $3F00-$3FFF works differently. The palette data is placed
        immediately on the data bus, and hence no dummy read is required. Reading the palettes
        still updates the internal buffer though, but the data placed in it is the mirrored nametable
        data that would appear "underneath" the palette. (Checking the PPU memory map should make this clearer.)

        So: reading returns value of buffer, not v. Therefore, if v has changed since it was last read, which it
        probably has, need to read twice, but that's advice to the programmer, not the emulator developer.
        As for 0x3F00 through 0x3FFF, the palette RAM indexes and their mirrors, need to find corresponding nametable?
        There are 4 nametables, duplicated once, so 8. There is one palette RAM index, mirrored 7 times, so 8.
        So to get from the fifth pallete RAM mirror, which would be 0x3F80, you'd select the 5th nametable,
        which would be the first mirrored nametable, 0x3000?
        No, just subtract 0x1000. https://forums.nesdev.com/viewtopic.php?f=3&t=18627:

            "However, I couldn't find any info on exactly which address should be used to populate the read buffer in this scenario.
            From other emulators, it appears to be PPU_ADDR - 0x1000, but I can't really intuit why that is the case."

            "It's the case because the majority of the time (that is, on just about every board but GTROM),
            video memory $3000-$3FFF mirrors $2000-$2FFF. When PA13 is high ($2000-$3FFF), nothing is listening
            to PA12 (the line that distinguishes $0000-$0FFF from $1000-$1FFF and distinguishes $2000-$2FFF from $3000-$3FFF)."
        */

        // read value at v, current VRAM address (15 bits)
        let mem_val = self.read(self.v as usize);
        let ret_val;
        match self.v % 0x4000 {
            0x0000..=0x3EFF => {
                ret_val = self.read_buffer;
                self.read_buffer = mem_val;
            }
            0x3F00..=0x3FFF => {
                ret_val = mem_val;
                self.read_buffer = self.read(self.v as usize - 0x1000);
            }
            _ => panic!("reading from invalid PPU address: 0x{:04x}", self.v),
        };

        if self.rendering() && (self.scanline < 240 || self.scanline == 261) {
            // During rendering (on the pre-render line and the visible lines 0-239, provided either background or sprite rendering is enabled),
            // it will update v in an odd way, triggering a coarse X increment and a Y increment simultaneously (with normal wrapping behavior).
            self.inc_coarse_x();
            self.inc_y();
        } else {
            // Outside of rendering, reads from or writes to $2007 will add either 1 or 32 to v depending on the VRAM increment bit set via $2000.
            self.v += self.address_increment;
        }
        ret_val
    }

    // cpu writes to 0x2007, PPUDATA
    pub fn write_data(&mut self, val: u8) {
        self.write(self.v as usize, val);
        if self.rendering() && (self.scanline < 240 || self.scanline == 261) {
            // During rendering (on the pre-render line and the visible lines 0-239, provided either background or sprite rendering is enabled),
            // it will update v in an odd way, triggering a coarse X increment and a Y increment simultaneously (with normal wrapping behavior).
            self.inc_coarse_x();
            self.inc_y();
        } else {
            // Outside of rendering, reads from or writes to $2007 will add either 1 or 32 to v depending on the VRAM increment bit set via $2000.
            self.v += self.address_increment;
        }
    }

    // cpu writes to 0x4014, OAMDATA
    pub fn write_oam_dma(&mut self, data: Vec<u8>) {
        self.primary_oam = data;
    }
}

pub fn set_bit(dest: &mut u16, dest_pos: usize, src: u16, src_pos: usize) {
    *dest &= 0xFFFF - (1 << dest_pos); // mask off destination bit
    *dest += (if src & (1 << src_pos) == 0 { 0 } else { 1 }) << dest_pos; // apply bit from src in src position
}
