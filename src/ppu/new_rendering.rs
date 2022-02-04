
impl super::Ppu {

    #[inline(always)]
    pub fn eight_sprite_pixels(&mut self) -> ([u8; 8], [usize; 8]) {
        // what we do now is: look at each sprite in secondary OAM, look at its counter
        // the counter is decremented each time: its value is distance from line_cycle
        // so we just need to look at whether its x-value > line_cycle. if it's not, sprite is live,
        // and the pixel we're concerned with is line_cycle - x-value?

        let mut pixels = [0u8; 8];
        let mut secondary_indices = [0usize; 8];

        for p in 0..8 {
            let mut frozen = false;
            for i in 0..self.num_sprites {
                // If the counter is zero, the sprite becomes "active", and the respective pair of shift registers for the sprite is shifted once every cycle.
                // This output accompanies the data in the sprite's latch, to form a pixel.
                if self.sprite_counters[i] <= self.line_cycle as u8 && self.sprite_counters[i] + 8 > self.line_cycle as u8 {
                    let diff = 7 - p;
                    // point is that the sprite is stuck in colums of 8 or 16, because we're not taking self.x into account before calculating if conditional above?

                    // The current pixel for each "active" sprite is checked (from highest to lowest priority),
                    // and the first non-transparent pixel moves on to a multiplexer, where it joins the BG pixel.
                    if !frozen {
                        secondary_indices[p as usize] = i;
                        let lb = ((self.sprite_pattern_table_srs[i].0 & 1 << diff) >> diff) as u8;
                        let hb = ((self.sprite_pattern_table_srs[i].1 & 1 << diff) >> diff) as u8;
                        if !(lb == 0 && hb == 0) {
                            pixels[p as usize] = (hb << 1) + lb;
                            frozen = true;
                        }
                    }
                }
            }
        }
        (pixels, secondary_indices)
    }

    #[inline(always)]
    pub fn render_eight_pixels(&mut self) {
        let (sprite_pixels, current_sprites) = self.eight_sprite_pixels();
        for i in 0..8 {
            let (x, _y) = (self.line_cycle - 1 + i, self.scanline);
            let background_pixel = if self.show_background && !(x < 8 && !self.show_background_left) {
                add_bits(self.background_pattern_shift_register_low, self.background_pattern_shift_register_high, 15 - (self.x + i as u8))
            } else {
                0
            };
            let (sprite_pixel, current_sprite) = if self.show_sprites && !(x < 8 && !self.show_sprites_left) {
                (sprite_pixels[i], current_sprites[i])
            } else {
                (0, 0)
            };

            // extract low and high bits from palette shift registers according to fine x, starting from left
            let val = 15 - (self.x + i as u8);
            let low_palette_bit = (self.background_palette_shift_register_low & (1 << val)) >> val;
            let high_palette_bit = (self.background_palette_shift_register_high & (1 << val)) >> val;
            let palette_offset = ((high_palette_bit << 1) | low_palette_bit) as u8;

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
            let pixel = self.palette_ram[palette_address as usize] as usize;
            let color: [u8; 3] = super::PALETTE_TABLE[pixel];
            let offset = (self.scanline * 256 * 3) + ((self.line_cycle - 1) * 3);
            self.screen_buffer[offset + (i*3) + 0] = color[0];
            self.screen_buffer[offset + (i*3) + 1] = color[1];
            self.screen_buffer[offset + (i*3) + 2] = color[2];
        }
    }

    // pixel is rendered, then if cycle 1, data is loaded into registers...
    // then registers are shifted, then memory fetch is performed
    // ldr puts lptb in bpsrl, then atb in bpl
    // shift registers affects pattern regs, palette regs, and feeds from latch
    // memory fetch does: inx_coarse_x on 0

    // cycle 0: render pixel, shift registers, inc coarse x
    // cycle 1: render pixel, load data into registers, shift registers, fetch nametable byte
    // cycle 2: render pixel, shift registers
    // cycle 3: render pixel, shift registers, fetch atb
    // cycle 4: render pixel, shift registers
    // cycle 5: render pixel, shift registers, fetch lptb
    // cycle 6: render pixel, shift registers
    // cycle 7: render pixel, shift registers, fetch hptb

    // render pixel and shift registers are semi-combined when batch rendering
    // seems like pixel selection can occur before other things, except load data into registers?
    // ok so on cycle 1, atb is set to palette latch

    pub fn new_shift_registers(&mut self) {
        self.background_pattern_shift_register_low <<= 8;
        self.background_pattern_shift_register_high <<= 8;
        self.background_pattern_shift_register_low |= self.low_pattern_table_byte as u16;
        self.background_pattern_shift_register_high |= self.high_pattern_table_byte as u16;
        self.background_palette_shift_register_low <<= 8;
        self.background_palette_shift_register_high <<= 8;
        self.background_palette_latch = self.attribute_table_byte; // palette latch is unnecessary
        self.background_palette_shift_register_low |= if self.background_palette_latch & 1 == 1 { 0b11111111 } else { 0 };
        self.background_palette_shift_register_high |= if self.background_palette_latch & 0b10 == 0b10 { 0b11111111 } else { 0 };
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
    pub fn new_clock(&mut self) -> (bool, bool) {
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
            // background-related things
            if self.scanline < 240 || self.scanline == 261 {
                match self.line_cycle {
                    0 => (), // This is an idle cycle.
                    1..=256 => {
                        if self.line_cycle % 8 == 1 {
                            if self.scanline != 261 {
                                self.render_eight_pixels();
                                rendered_scanline = true;
                            }
                            self.new_perform_memory_fetch();
                            self.new_shift_registers();
                        }
                    },
                    257 => self.copy_horizontal(), // At dot 257 of each scanline, if rendering is enabled, the PPU copies all bits related to horizontal position from t to v
                    321..=336 => if self.line_cycle % 8 == 1 {
                        self.new_perform_memory_fetch();
                        self.new_shift_registers();
                    },
                    x if x > 340 => panic!("cycle beyond 340: {}", x),
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
        } else {
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

        (end_of_frame, rendered_scanline)
    }
}

#[inline(always)]
fn add_bits(low: u16, high: u16, bit: u8) -> u8 {
    let low_bit = (low & 1 << bit) >> bit;
    let high_bit = ((high & 1 << bit) >> bit) << 1;
    (high_bit + low_bit) as u8
}

            // if sprite_pixel != 0 {
            //     if background_pixel != 0 {
            //         if self.sprite_indexes[current_sprite] == 0 { // don't access index current_sprite. need to know which sprite we're on horizontally.
            //             self.sprite_zero_hit = true;
            //         }
            //         if self.sprite_attribute_latches[current_sprite] & (1 << 5) == 0 { // sprite has high priority
            //             palette_address += 0x10;
            //             palette_address += (self.sprite_attribute_latches[current_sprite] & 0b11) << 2;
            //             palette_address += sprite_pixel;
            //         } else {
            //             palette_address += palette_offset << 2;
            //             palette_address += background_pixel;
            //         }
            //     } else { // displaying the sprite
            //         palette_address += 0x10; // second half of palette table, "Background/Sprite select"
            //         palette_address += (self.sprite_attribute_latches[current_sprite] & 0b11) << 2; // bottom two bits of attribute byte, left shifted by two
            //         palette_address += sprite_pixel; // bottom two bits are the value of the sprite pixel from pattern table
            //     }
            // } else { // displaying the background pixel
            //     palette_address += palette_offset << 2; // Palette number from attribute table or OAM
            //     palette_address += background_pixel; // Pixel value from tile data
            // }
