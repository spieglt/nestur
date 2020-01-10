use super::cpu_registers::set_bit;

impl super::Ppu {

    pub fn perform_memory_fetch(&mut self) {
        match self.line_cycle % 8 {
            0 => self.inc_coarse_x(),
            1 => self.fetch_nametable_byte(),
            3 => self.fetch_attribute_table_byte(),
            5 => self.fetch_low_pattern_table_byte(),
            7 => self.fetch_high_pattern_table_byte(),
            _ => (),
        };
    }

    pub fn shift_registers(&mut self) {
        // Shift pattern output registers
        self.background_pattern_sr_low <<= 1;
        self.background_pattern_sr_high <<= 1;
        // Shift the palette shift registers
        self.background_palette_sr_low <<= 1;
        self.background_palette_sr_high <<= 1;
        // Shift bit 0 of the palette attribute latch into the bottom bit of the low palette shift register,
        self.background_palette_sr_low |= (self.background_palette_latch & 1 << 0 != 0) as u8;
        // and bit 1 of the palette attribute latch into the bottom bit of the high palette shift register
        self.background_palette_sr_high |= (self.background_palette_latch & 1 << 1 != 0) as u8;
    }

    pub fn load_data_into_registers(&mut self) {
        if self.line_cycle % 8 == 1 { // The shifters are reloaded during ticks 9, 17, 25, ..., 257.
            // These contain the pattern table data for two tiles. Every 8 cycles, the data for the next
            // tile is loaded into the upper 8 bits of this shift register. Meanwhile, the pixel to render is fetched from one of the lower 8 bits.
            self.background_pattern_sr_low |= self.low_pattern_table_byte as u16;
            self.background_pattern_sr_high |= self.high_pattern_table_byte as u16;
            self.background_palette_latch = self.attribute_table_byte;
        }
    }

    pub fn fetch_nametable_byte(&mut self) {
        // nametable address is the bottom 12 bits of v in the 0x2000 range
        self.nametable_byte = self.read(0x2000 | (self.v & 0b00001111_11111111) as usize);
    }

    pub fn fetch_attribute_table_byte(&mut self) {
        let address = 0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07);
        let byte = self.read(address as usize);
        // figure out which two bits are being represented, ignoring fine x and fine y
        // left or right:
        let coarse_x =  self.v & 0b00000000_00011111;
        let coarse_y = (self.v & 0b00000011_11100000) >> 5;
        let left_or_right = (coarse_x / 2) % 2; // 0 == left, 1 == right
        let top_or_bottom = (coarse_y / 2) % 2; // 0 == top, 1 == bottom
        // grab the needed two bits
        self.attribute_table_byte = match (top_or_bottom, left_or_right) {
            (0,0) => (byte >> 0) & 0b00000011,
            (0,1) => (byte >> 2) & 0b00000011,
            (1,0) => (byte >> 4) & 0b00000011,
            (1,1) => (byte >> 6) & 0b00000011,
            _ => panic!("should not get here"),
        };
    }

    pub fn fetch_low_pattern_table_byte(&mut self) {
        // pattern table address should be the pattern table base (0x0 or 0x1000)
        let mut address = self.background_pattern_table_base;
        // plus the value of the nametable byte left-shifted by 4 (which the wiki doesn't really explain)
        address += (self.nametable_byte as usize) << 4;
        // plus the fine Y scroll
        address += ((self.v as usize) >> 12) & 7;
        self.low_pattern_table_byte = self.read(address);
    }

    pub fn fetch_high_pattern_table_byte(&mut self) {
        // same as low pattern table byte, but "Fetch the high-order byte of this sliver from an address 8 bytes higher."
        let mut address = self.background_pattern_table_base;
        address += (self.nametable_byte as usize) << 4;
        address += (self.v as usize >> 12) & 7;
        self.high_pattern_table_byte = self.read(address + 8);
    }

    pub fn render_pixel(&mut self) -> (usize, usize, (u8, u8, u8)) {
        let (x, y) = (self.line_cycle - 1, self.scanline);
        let mut background_pixel = self.select_background_pixel();
        let (mut sprite_pixel, current_sprite) = self.select_sprite_pixel();

        // extract low and high bits from palette shift registers according to fine x, starting from left
        let low_palette_bit = (self.background_palette_sr_low & (1 << (7-self.x)) != 0) as u8;
        let high_palette_bit = (self.background_palette_sr_high & (1 << (7-self.x)) != 0) as u8;
        let palette_offset = (high_palette_bit << 1) | low_palette_bit;
        
        if x < 8 && !self.show_background_left {
            background_pixel = 0;
        }
        if x < 8 && !self.show_sprites_left {
            sprite_pixel = 0;
        }
        let mut palette_address = 0;
        if background_pixel == 0 && sprite_pixel != 0 { // displaying the sprite
            palette_address += 0x10; // second half of palette table, "Background/Sprite select"
            palette_address += (self.sprite_attribute_latches[current_sprite] & 0b11) << 2; // bottom two bits of attribute byte, left shifted by two
            palette_address += sprite_pixel; // bottom two bits are the value of the sprite pixel from pattern table
        } else if background_pixel != 0 && sprite_pixel == 0 { // displaying the background pixel
            palette_address += palette_offset << 2; // Palette number from attribute table or OAM
            palette_address += background_pixel; // Pixel value from tile data
        } else if background_pixel != 0 && sprite_pixel != 0 {
            if self.sprite_indexes[current_sprite] == 0 { // don't access index current_sprite. need to know which sprite we're on horizontally.
                self.sprite_zero_hit = true;
            }
            if self.sprite_attribute_latches[current_sprite] & (1 << 5) == 0 { // sprite has high priority
                palette_address += 0x10;
                palette_address += (self.sprite_attribute_latches[current_sprite] & 0b11) << 2;
                palette_address += sprite_pixel;
            } else {
                palette_address += palette_offset << 2;
                palette_address += background_pixel;
            }
        }
        // let pixel = self.read(palette_address as usize) as usize;
        let pixel = self.palette_ram[palette_address as usize] as usize;
        let color: (u8, u8, u8) = super::PALETTE_TABLE[pixel];
        
        (x,y,color)
    }

    pub fn select_background_pixel(&mut self) -> u8 {
        if self.show_background {
            // Returned background pixel is a value between 0 and 3.
            // the bit from background_pattern_sr_low (low pattern table byte) in the 0th place,
            // and the value of the background_pattern_sr_high (high pattern table byte) in the 1st place.
            let low_bit  = (self.background_pattern_sr_low & (1 << (15 - self.x)) != 0) as u8;
            let high_bit = (self.background_pattern_sr_high & (1 << (15 - self.x)) != 0) as u8;
            (high_bit << 1) | low_bit
        } else {
            0
        }
    }

    pub fn select_sprite_pixel(&mut self) -> (u8, usize) {
        // Returns (sprite_pixel, index of sprite_pixel within secondary_oam/shift registers)
        if self.show_sprites {
            // sprite pixel is a value between 0 and 3 representing the two sprite pattern table shift registers
            let mut low_bit  = 0;
            let mut high_bit = 0;
            let mut secondary_index = 0;
            for i in 0..self.num_sprites {
                // If the counter is zero, the sprite becomes "active", and the respective pair of shift registers for the sprite is shifted once every cycle.
                // This output accompanies the data in the sprite's latch, to form a pixel.
                if self.sprite_counters[i] == 0 {
                    // The current pixel for each "active" sprite is checked (from highest to lowest priority),
                    // and the first non-transparent pixel moves on to a multiplexer, where it joins the BG pixel.
                    secondary_index = i;
                    low_bit  = (self.sprite_pattern_table_srs[i].0 & 1<<7 != 0) as u8;
                    high_bit = (self.sprite_pattern_table_srs[i].1 & 1<<7 != 0) as u8;
                    if !(low_bit == 0 && high_bit == 0) {
                        break;
                    }
                }
            }
            // Have to shift pixels of all sprites with counter 0, whether or not they're the selected pixel. otherwise the pixels get pushed to the right.
            for i in 0..self.num_sprites {
                if self.sprite_counters[i] == 0 {
                    self.sprite_pattern_table_srs[i].0 <<= 1;
                    self.sprite_pattern_table_srs[i].1 <<= 1;
                }
            }
            // Every cycle, the 8 x-position counters for the sprites are decremented by one.
            for i in 0..self.num_sprites {
                if self.sprite_counters[i] > 0 {
                    self.sprite_counters[i] -= 1;
                }
            }
            ((high_bit << 1) | low_bit, secondary_index)
        } else {
            (0, 0)
        }
    }

    pub fn evaluate_sprites(&mut self) {        
        let mut sprite_count = 0;
        for n in 0..64 {
            let y_coord = self.primary_oam[(n*4)+0];
            if self.y_in_range(y_coord) {
                for i in 0..4 {
                    self.secondary_oam[(sprite_count*4)+i] = self.primary_oam[(n*4)+i];
                }
                self.sprite_indexes[sprite_count] = n as u8;
                sprite_count += 1;
            } else {
                // TODO: sprite evaluation bug
            }
            if sprite_count == 8 {
                break;
            }
        }
        self.num_sprites = sprite_count;

    }

    pub fn fetch_sprites(&mut self) {
        for i in 0..self.num_sprites {
            let mut address: usize;
            let sprite_y_position = self.secondary_oam[(4*i)+0] as usize; // byte 0 of sprite, sprite's vertical position on screen
            let sprite_tile_index = self.secondary_oam[(4*i)+1] as usize; // byte 1 of sprite, sprite's location within pattern table
            let sprite_attributes = self.secondary_oam[(4*i)+2];          // byte 2 of sprite, sprite's palette, priority, and flip attributes
            let sprite_x_position = self.secondary_oam[(4*i)+3];          // byte 3 of sprite, sprite's horizontal position on screen
            // For 8x8 sprites, this is the tile number of this sprite within the pattern table selected in bit 3 of PPUCTRL ($2000).
            if self.sprite_size == 8 {
                address = self.sprite_pattern_table_base;
                address += sprite_tile_index*16;
            // For 8x16 sprites, the PPU ignores the pattern table selection and selects a pattern table from bit 0 of this number. 
            } else {
                address = if sprite_tile_index & 1 == 0 { 0x0 } else { 0x1000 };
                address += (sprite_tile_index*16) & (0xFF - 1); // turn off bottom bit
            }
            let fine_y: usize;
            // Handle vertical and horizontal flips, then write to shift registers
            if sprite_attributes & (1<<7) == 0 { // if vertical flip bit not set
                fine_y = self.scanline - sprite_y_position; // row-within-sprite offset is difference between current scanline and top of sprite
            } else { // if flipped vertically
                fine_y = self.sprite_size as usize - 1 - (self.scanline - sprite_y_position);
            }
            address += fine_y;
            let low_pattern_table_byte = self.read(address);
            let high_pattern_table_byte = self.read(address + 8);
            let mut shift_reg_vals = (0, 0);
            for j in 0..8 {
                let current_bits = (low_pattern_table_byte & (1 << j), high_pattern_table_byte & (1 << j));
                if sprite_attributes & (1<<6) == 0 { // if horizontal flip bit not set
                    // just copy each bit in same order
                    shift_reg_vals.0 |= current_bits.0;
                    shift_reg_vals.1 |= current_bits.1;
                } else { // if flipped horizontally
                    // get bit of pattern table byte, left shift it by 7 - bit position
                    shift_reg_vals.0 |= ((current_bits.0 != 0) as u8) << (7 - j);
                    shift_reg_vals.1 |= ((current_bits.1 != 0) as u8) << (7 - j);
                }
            }
            // put pattern table bytes into the shift registers, ready to be rendered
            self.sprite_pattern_table_srs[i] = shift_reg_vals;
            // In addition to this, the X positions and attributes for each sprite are loaded from the secondary OAM into their respective counters/latches.
            // This happens during the second garbage nametable fetch, with the attribute byte loaded during the first tick and the X coordinate during the second.
            self.sprite_attribute_latches[i] = sprite_attributes;
            self.sprite_counters[i] = sprite_x_position;
        }
    }

    pub fn inc_coarse_x(&mut self) {
        if self.v & 0x001F == 0x001F { // if coarse X == 31
            self.v &= !0x001F;         // coarse X = 0
            self.v ^= 1<<10;           // switch horizontal nametable
        } else {
            self.v += 1;
        }
    }

    pub fn inc_y(&mut self) {
        // If rendering is enabled, fine Y is incremented at dot 256 of each scanline, 
        // overflowing to coarse Y, and finally adjusted to wrap among the nametables vertically. 
        let mut fine_y   = (self.v & 0b01110000_00000000) >> 12;
        let mut coarse_y = (self.v & 0b00000011_11100000) >> 5;
        if fine_y < 7 {
            fine_y += 1;
        } else {
            fine_y = 0;
            // Row 29 is the last row of tiles in a nametable. To wrap to the next nametable when
            // incrementing coarse Y from 29, the vertical nametable is switched by toggling bit 
            // 11, and coarse Y wraps to row 0.
            if coarse_y == 29 {
                self.v ^= 1<<11;
                coarse_y = 0;
            // Coarse Y can be set out of bounds (> 29), which will cause the PPU to read the
            // attribute data stored there as tile data. If coarse Y is incremented from 31,
            // it will wrap to 0, but the nametable will not switch.
            } else if coarse_y == 32 {
                coarse_y = 0;
            } else {
                coarse_y += 1;
            }
        }
        // set resulting coarse y
        set_bit(&mut self.v, 0x5, coarse_y, 0x0);
        set_bit(&mut self.v, 0x6, coarse_y, 0x1);
        set_bit(&mut self.v, 0x7, coarse_y, 0x2);
        set_bit(&mut self.v, 0x8, coarse_y, 0x3);
        set_bit(&mut self.v, 0x9, coarse_y, 0x4);
        // and fine y
        set_bit(&mut self.v, 0xC, fine_y, 0x0);
        set_bit(&mut self.v, 0xD, fine_y, 0x1);
        set_bit(&mut self.v, 0xE, fine_y, 0x2);
    }

    pub fn copy_horizontal(&mut self) {
        // v: ....F.. ...EDCBA = t: ....F.. ...EDCBA
        let mask = 0b00000100_00011111;
        let t_vals = self.t & mask; // grab bits of t
        self.v &= !mask;            // turn off bits of v
        self.v |= t_vals;           // apply bits of t
    }

    pub fn copy_vertical(&mut self) {
        // v: IHGF.ED CBA..... = t: IHGF.ED CBA.....
        let mask = 0b01111011_11100000;
        let t_vals = self.t & mask;
        self.v &= !mask;
        self.v |= t_vals;
    }

    pub fn rendering(&self) -> bool {
        (self.show_background || self.show_sprites)
    }

    pub fn y_in_range(&self, y_coord: u8) -> bool {
        self.scanline >= (y_coord as usize) && 
            self.scanline - (y_coord as usize) < self.sprite_size as usize
    }

    pub fn nmi_change(&mut self) {
        let nmi = self.should_generate_nmi && self.vertical_blank;
        if nmi && !self.previous_nmi {
            self.nmi_delay = 1;
        }
        self.previous_nmi = nmi;
    }
}
