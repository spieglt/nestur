
// so instead of shifting, we just need to access the next index in the shift registers.
// and the background palette shift register is just fed from the latch for the next 8 pixels.
// the palette latch is just the attribute_table_byte. 

// to render an 8-pixel stretch, we need:
// the background pixel, which comes from the background pattern shift registers, low and high. those are 16 bits, so we can work with 16 bits maybe?
// the sprite pixel, which comes from the sprite pattern shift registers
// the secondary index, blah blah

// then decide priority, 

// render whole background row, then sprites, then filter?

// so to get 8 background pixels we need to, load data into registers? perform memory fetch, and shift registers

impl super::Ppu {

    pub fn render_background_scanline(&mut self) -> Vec<u8> {
        // render 8 pixels, then shift and such 64 times?
        let mut scanline = vec![];
        for _ in 0..64 {
            scanline.append(&mut self.render_eight_background_pixels());
        }
        scanline
    }

    pub fn render_eight_background_pixels(&mut self) -> Vec<u8> {
        let mut eight_pixels = vec![];
        if self.show_background && self.show_background_left {
            for i in 0..8 {
                let mut palette_address = add_bits(self.low_pattern_table_byte, self.high_pattern_table_byte, i as u8);
                palette_address += self.attribute_table_byte << 2;
                let pixel = self.palette_ram[palette_address as usize] as usize;
                eight_pixels.append(&mut super::PALETTE_TABLE[pixel].to_vec());
            }
        }
        self.line_cycle += 8;
        self.new_perform_memory_fetch();
        eight_pixels
    }

    #[inline(always)]
    pub fn new_perform_memory_fetch(&mut self) {
        self.inc_coarse_x();
        self.fetch_nametable_byte();
        self.fetch_attribute_table_byte();
        self.fetch_background_pattern_table_bytes();
    }

    #[inline(always)]
    pub fn fetch_background_pattern_table_bytes(&mut self) {
        // pattern table address should be the pattern table base (0x0 or 0x1000)
        let mut address = self.background_pattern_table_base;
        // plus the value of the nametable byte left-shifted by 4 (which the wiki doesn't really explain)
        address += (self.nametable_byte as usize) << 4;
        // plus the fine Y scroll
        address += ((self.v as usize) >> 12) & 7;
        self.low_pattern_table_byte = self.read(address);
        self.high_pattern_table_byte = self.read(address + 8);
    }


    #[inline(always)]
    pub fn new_clock(&mut self) -> bool {
        if self.nmi_delay > 0 {
            self.nmi_delay -= 1;
            if self.nmi_delay == 0 && self.should_generate_nmi && self.vertical_blank {
                self.trigger_nmi = true;
            }
        }

        let rendering = self.rendering();
        let mut rendered_scanline = false;

        // Visible scanlines (0-239)
        if rendering {
            if self.scanline < 240 || self.scanline == 261 {
                // background-related things
                match self.line_cycle {
                    0 => (), // This is an idle cycle.
                    1 => {
                        let scanline = self.render_background_scanline();
                        let y_offset = self.scanline * 3 * 256;
                        self.screen_buffer.splice(y_offset..y_offset+256, scanline);
                        let rendered_scanline = true;
                    },
                    2..=256 => (),
                    257 => self.copy_horizontal(), // At dot 257 of each scanline, if rendering is enabled, the PPU copies all bits related to horizontal position from t to v
                    321..=336 => {
                        if self.line_cycle % 8 == 1 { // The shifters are reloaded during ticks 9, 17, 25, ..., 257.
                            self.load_data_into_registers();
                        }
                        self.shift_registers();
                        self.perform_memory_fetch();
                    },
                    x if x > 340 => panic!("cycle beyond 340"),
                    _ => (),
                }
            }

            // sprite-related things
            if self.scanline < 240 {
                match self.line_cycle {
                    1 => self.secondary_oam = vec![0xFF; 0x20],
                    257 => {
                        self.evaluate_sprites(); // ignoring all timing details
                        self.fetch_sprites();
                    },
                    321..=340 => (), // Read the first byte in secondary OAM (while the PPU fetches the first two background tiles for the next scanline)
                    _ => (),
                }
            }

            // During dots 280 to 304 of the pre-render scanline (end of vblank)
            // If rendering is enabled, at the end of vblank, shortly after the horizontal bits
            // are copied from t to v at dot 257, the PPU will repeatedly copy the vertical bits
            // from t to v from dots 280 to 304, completing the full initialization of v from t:
            if self.scanline == 261 && self.line_cycle >= 280 && self.line_cycle <= 304 {
                self.copy_vertical();
            }
            // At dot 256 of each scanline, if rendering is enabled, the PPU increments the vertical position in v.
            if self.line_cycle == 256 && (self.scanline < 240 || self.scanline == 261) {
                self.inc_y();
            }
        }

        // v blank
        if self.scanline == 241 && self.line_cycle == 1 {
            self.vertical_blank = true;
            self.nmi_change();
        }
        if self.scanline == 261 && self.line_cycle == 1 {
            self.vertical_blank = false;
            self.nmi_change();
            self.sprite_zero_hit = false;
            self.sprite_overflow = false;
        }

        // signal time to draw frame
        let end_of_frame = self.line_cycle == 256 && self.scanline == 240;

        // advance clock
        // For odd frames, the cycle at the end of the pre-render scanline is skipped
        if self.line_cycle == 339 && self.scanline == 261 && self.frame % 2 != 0 {
            self.line_cycle = 0;
            self.scanline = 0;
            self.frame = self.frame.wrapping_add(1);
        // Otherwise, if at the last cycle of the last row of a frame, advance it.
        } else if self.line_cycle == 340 && self.scanline == 261 {
            self.line_cycle = 0;
            self.scanline = 0;
            self.frame = self.frame.wrapping_add(1);
        // If at a normal line end, advance to next
        } else if self.line_cycle == 340 {
            self.line_cycle = 0;
            self.scanline += 1;
        // If none of the above, just go to next cycle in the row
        } else if !rendered_scanline {
            self.line_cycle += 1;
        }

        // deal with mapper MMC3
        let current_a12 = ((self.v & 1 << 12) >> 12) as u8;
        if rendering 
            && (0..241).contains(&self.scanline)
            && current_a12 != self.previous_a12
        {
            self.mapper.clock()
        }
        self.previous_a12 = current_a12;

        end_of_frame
    }
}

#[inline(always)]
fn add_bits(low: u8, high: u8, bit: u8) -> u8 {
    let low_bit = (low & 1 << bit) >> bit;
    let high_bit = ((high & 1 << bit) >> bit) << 1;
    high_bit + low_bit
}
