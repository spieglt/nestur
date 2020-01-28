mod cpu_registers;
mod memory;
mod rendering;

use crate::cartridge::Mapper;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Ppu {
    line_cycle: usize, // x coordinate
    scanline: usize,   // y coordinate
    frame: usize,

    // Internal registers
    v: u16,
    t: u16,
    x: u8, // Fine X scroll (3 bits)
    w: u8, // First or second write toggle (1 bit)

    // Cartridge things
    pub mapper: Rc<RefCell<dyn Mapper>>,

    // Each nametable byte is a reference to the start of an 8-byte sequence in the pattern table.
    // That sequence represents an 8x8 tile, from top row to bottom.
    // First interpretation of how nametables work was wrong. There are two banks. (Or up to 4 on some games.)
    // Pictures on http://wiki.nesdev.com/w/index.php/Mirroring refer to them as A and B.
    // http://wiki.nesdev.com/w/index.php/MMC1 calls them higher and lower.
    // They can be mirrored at certain memory ranges.
    nametable_A: Vec<u8>,
    nametable_B: Vec<u8>,
    nametable_C: Vec<u8>,
    nametable_D: Vec<u8>,

    // The palette shared by both background and sprites.
    // Consists of 32 bytes, each of which represents an index into the global PALETTE_TABLE.
    // The first 16 bytes are for the background, the second half for the sprites.
    palette_ram: Vec<u8>, // Palette RAM indexes.

    // Background pattern shift registers and latches
    background_pattern_sr_low: u16, // 2 16-bit shift registers - These contain the pattern table data for two tiles. Every 8 cycles, the data for the next tile is loaded
    background_pattern_sr_high: u16, // into the upper 8 bits of this shift register. Meanwhile, the pixel to render is fetched from one of the lower 8 bits.
    nametable_byte: u8, // "The data fetched from these accesses is placed into internal latches,
    attribute_table_byte: u8, // and then fed to the appropriate shift registers when it's time to do so
    low_pattern_table_byte: u8, // (every 8 cycles)."
    high_pattern_table_byte: u8,

    // Background palette shift registers and latches
    background_palette_sr_low: u8,  // 2 8-bit shift registers -
    background_palette_sr_high: u8, // These contain the palette attributes for the lower 8 pixels of the 16-bit [pattern/tile] shift register.
    background_palette_latch: u8, // These registers are fed by a latch which contains the palette attribute for the next tile. Every 8 cycles,
    // the latch is loaded with the palette attribute for the next tile. Because the PPU can only fetch an attribute byte every 8 cycles, each
    // sequential string of 8 pixels is forced to have the same palette attribute.

    // Sprite memory, shift registers, and latch
    pub primary_oam: Vec<u8>, // Primary OAM (holds 64 sprites for the frame)
    secondary_oam: Vec<u8>,   // Secondary OAM (holds 8 sprites for the current scanline)
    sprite_attribute_latches: Vec<u8>, // 8 latches - These contain the attribute bytes [palette data] for up to 8 sprites.
    sprite_counters: Vec<u8>, // 8 counters - These contain the X positions for up to 8 sprites.
    sprite_indexes: Vec<u8>,  // Indexes of the sprites-in-the-attribute-latches' within primary OAM
    sprite_pattern_table_srs: Vec<(u8, u8)>, // 8 pairs of 8-bit shift registers - These contain the pattern table data for up to 8 sprites, to be rendered on the current scanline.
    // Unused sprites are loaded with an all-transparent set of values.
    num_sprites: usize, // Number of sprites in the shift registers for the current scanline

    // Various flags set by registers
    address_increment: u16,
    sprite_pattern_table_base: usize,
    background_pattern_table_base: usize,
    oam_address: usize,
    sprite_size: u8,
    grayscale: bool,
    show_background_left: bool, // 1: Show background in leftmost 8 pixels of screen, 0: Hide
    show_sprites_left: bool,    // 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    show_background: bool,      // 1: Show background
    show_sprites: bool,         // 1: Show sprites
    emphasize_red: bool,        // Emphasize red
    emphasize_green: bool,      // Emphasize green
    emphasize_blue: bool,       // Emphasize blue
    sprite_overflow: bool,      // Set if there are more than 8 sprites on a single line
    sprite_zero_hit: bool, // Set when the first pixel of the sprite in the zero index of primary OAM is rendered
    should_generate_nmi: bool, // Allows CPU to control whether NMIs trigger
    vertical_blank: bool,  // true == in vertical blank, false == not

    // These three: god knows.
    // TODO: experiment more with NMI
    pub trigger_nmi: bool, // triggers NMI in the CPU when it checks in its step()
    previous_nmi: bool,
    nmi_delay: usize,

    read_buffer: u8,     // used with PPUDATA register
    pub recent_bits: u8, // Least significant bits previously written into a PPU register
}

impl Ppu {
    pub fn new(mapper: Rc<RefCell<dyn Mapper>>) -> Self {
        Ppu {
            line_cycle: 0,
            scanline: 0,
            frame: 0,
            v: 0,
            t: 0,
            x: 0,
            w: 0,
            mapper: mapper,
            nametable_A: vec![0u8; 0x0400],
            nametable_B: vec![0u8; 0x0400],
            nametable_C: vec![0u8; 0x0400],
            nametable_D: vec![0u8; 0x0400],
            palette_ram: vec![0u8; 0x0020],
            background_pattern_sr_low: 0,
            background_pattern_sr_high: 0,
            nametable_byte: 0,
            attribute_table_byte: 0,
            low_pattern_table_byte: 0,
            high_pattern_table_byte: 0,
            background_palette_sr_low: 0,
            background_palette_sr_high: 0,
            background_palette_latch: 0,
            primary_oam: vec![0u8; 0x0100],
            secondary_oam: vec![0u8; 0x0020],
            sprite_attribute_latches: vec![0u8; 8],
            sprite_counters: vec![0u8; 8],
            sprite_indexes: vec![0u8; 8],
            sprite_pattern_table_srs: vec![(0u8, 0u8); 8],
            num_sprites: 0,
            address_increment: 0,
            sprite_pattern_table_base: 0,
            background_pattern_table_base: 0,
            oam_address: 0,
            sprite_size: 0,
            grayscale: false,
            show_background_left: false,
            show_sprites_left: false,
            show_background: false,
            show_sprites: false,
            emphasize_red: false,   //
            emphasize_green: false, // TODO: implement these
            emphasize_blue: false,  //
            sprite_overflow: false,
            sprite_zero_hit: false,
            should_generate_nmi: false,
            vertical_blank: false,
            trigger_nmi: false,
            previous_nmi: false,
            nmi_delay: 0,
            read_buffer: 0,
            recent_bits: 0,
        }
    }

    pub fn clock(&mut self) -> (Option<(usize, usize, (u8, u8, u8))>, bool) {
        if self.nmi_delay > 0 {
            self.nmi_delay -= 1;
            if self.nmi_delay == 0 && self.should_generate_nmi && self.vertical_blank {
                self.trigger_nmi = true;
            }
        }

        let mut pixel: Option<(usize, usize, (u8, u8, u8))> = None;
        let rendering = self.rendering();

        // Visible scanlines (0-239)
        if rendering && (self.scanline < 240 || self.scanline == 261) {
            // background-related things
            match self.line_cycle {
                0 => (), // This is an idle cycle.
                1..=256 => {
                    if self.scanline != 261 {
                        pixel = Some(self.render_pixel());
                    }
                    self.load_data_into_registers();
                    self.shift_registers();
                    self.perform_memory_fetch();
                }
                257 => self.copy_horizontal(), // At dot 257 of each scanline, if rendering is enabled, the PPU copies all bits related to horizontal position from t to v
                321..=336 => {
                    self.load_data_into_registers();
                    self.shift_registers();
                    self.perform_memory_fetch();
                }
                x if x > 340 => panic!("cycle beyond 340"),
                _ => (),
            }
        }

        // sprite-related things
        if rendering && self.scanline < 240 {
            match self.line_cycle {
                1 => self.secondary_oam = vec![0xFF; 0x20],
                257 => {
                    self.evaluate_sprites(); // ignoring all timing details
                    self.fetch_sprites();
                }
                321..=340 => (), // Read the first byte in secondary OAM (while the PPU fetches the first two background tiles for the next scanline)
                _ => (),
            }
        }

        // During dots 280 to 304 of the pre-render scanline (end of vblank)
        // If rendering is enabled, at the end of vblank, shortly after the horizontal bits
        // are copied from t to v at dot 257, the PPU will repeatedly copy the vertical bits
        // from t to v from dots 280 to 304, completing the full initialization of v from t:
        if rendering && self.scanline == 261 && self.line_cycle >= 280 && self.line_cycle <= 304 {
            self.copy_vertical();
        }
        // At dot 256 of each scanline, if rendering is enabled, the PPU increments the vertical position in v.
        if rendering && self.line_cycle == 256 && (self.scanline < 240 || self.scanline == 261) {
            self.inc_y();
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
        // if self.rendering()
        // && (1..241).contains(&self.scanline)
        // && (
        //     (
        //         self.line_cycle == 260
        //         && self.sprite_size == 8
        //         && self.background_pattern_table_base == 0x0000
        //         && self.sprite_pattern_table_base == 0x1000
        //     ) || (
        //         self.line_cycle == 324
        //         && self.sprite_size == 8
        //         && self.background_pattern_table_base == 0x1000
        //         && self.sprite_pattern_table_base == 0x0000
        //     ) || (
        //         self.line_cycle == 260
        //         && self.sprite_size == 16
        //         // TODO: figure out exact conditions here
        //     )
        // )
        if self.rendering() && self.line_cycle == 260 && (1..241).contains(&self.scanline) {
            self.mapper.borrow_mut().clock()
        }

        (pixel, end_of_frame)
    }
}

const PALETTE_TABLE: [(u8, u8, u8); 64] = [
    (84, 84, 84),
    (0, 30, 116),
    (8, 16, 144),
    (48, 0, 136),
    (68, 0, 100),
    (92, 0, 48),
    (84, 4, 0),
    (60, 24, 0),
    (32, 42, 0),
    (8, 58, 0),
    (0, 64, 0),
    (0, 60, 0),
    (0, 50, 60),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (152, 150, 152),
    (8, 76, 196),
    (48, 50, 236),
    (92, 30, 228),
    (136, 20, 176),
    (160, 20, 100),
    (152, 34, 32),
    (120, 60, 0),
    (84, 90, 0),
    (40, 114, 0),
    (8, 124, 0),
    (0, 118, 40),
    (0, 102, 120),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (236, 238, 236),
    (76, 154, 236),
    (120, 124, 236),
    (176, 98, 236),
    (228, 84, 236),
    (236, 88, 180),
    (236, 106, 100),
    (212, 136, 32),
    (160, 170, 0),
    (116, 196, 0),
    (76, 208, 32),
    (56, 204, 108),
    (56, 180, 204),
    (60, 60, 60),
    (0, 0, 0),
    (0, 0, 0),
    (236, 238, 236),
    (168, 204, 236),
    (188, 188, 236),
    (212, 178, 236),
    (236, 174, 236),
    (236, 174, 212),
    (236, 180, 176),
    (228, 196, 144),
    (204, 210, 120),
    (180, 222, 120),
    (168, 226, 144),
    (152, 226, 180),
    (160, 214, 228),
    (160, 162, 160),
    (0, 0, 0),
    (0, 0, 0),
];
