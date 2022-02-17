mod cpu_registers;
mod rendering;
mod memory;
pub mod serialize;
mod new_rendering;

use crate::cartridge::{cache, Mapper, Mirror};

pub struct Ppu {
    pub screen_buffer: Vec<u8>, // raw RGB data for screen
    line_cycle: usize, // x coordinate
    pub scanline:   usize, // y coordinate
    frame:      usize,

    // Internal registers
    v: u16,
    t: u16,
    x: u8, // Fine X scroll (3 bits)
    w: u8, // First or second write toggle (1 bit)

    // Cartridge things
    pub mapper: Box<dyn Mapper>,
    mirroring_type: Mirror,
    cache: cache::Cache, // 128KB cache: 2048 regions of 24 bytes

    // Each nametable byte is a reference to the start of an 8-byte sequence in the pattern table.
    // That sequence represents an 8x8 tile, from top row to bottom.
    // First interpretation of how nametables work was wrong. There are two banks. (Or up to 4 on some games.)
    // Pictures on http://wiki.nesdev.com/w/index.php/Mirroring refer to them as A and B.
    // http://wiki.nesdev.com/w/index.php/MMC1 calls them higher and lower.
    // They can be mirrored at certain memory ranges.
    nametable_a: Vec<u8>,
    nametable_b: Vec<u8>,
    nametable_c: Vec<u8>,
    nametable_d: Vec<u8>,

    // The palette shared by both background and sprites.
    // Consists of 32 bytes, each of which represents an index into the global PALETTE_TABLE.
    // The first 16 bytes are for the background, the second half for the sprites.
    palette_ram: Vec<u8>, // Palette RAM indexes.

    // Background pattern shift registers and latches
    background_pattern_sr_low:  u16, // 2 16-bit shift registers - These contain the pattern table data for two tiles. Every 8 cycles, the data for the next tile is loaded
    background_pattern_sr_high: u16, // into the upper 8 bits of this shift register. Meanwhile, the pixel to render is fetched from one of the lower 8 bits.
    nametable_byte:             u8, // "The data fetched from these accesses is placed into internal latches,
    attribute_table_byte:       u8, // and then fed to the appropriate shift registers when it's time to do so
    low_pattern_table_byte:     u8, // (every 8 cycles)."
    high_pattern_table_byte:    u8,

    // Background palette shift registers and latches
    background_palette_sr_low:  u8,  // 2 8-bit shift registers -
    background_palette_sr_high: u8,  // These contain the palette attributes for the lower 8 pixels of the 16-bit [pattern/tile] shift register.
    background_palette_latch: u8,    // These registers are fed by a latch which contains the palette attribute for the next tile. Every 8 cycles,
    // the latch is loaded with the palette attribute for the next tile. Because the PPU can only fetch an attribute byte every 8 cycles, each
    // sequential string of 8 pixels is forced to have the same palette attribute.

    // Sprite memory, shift registers, and latch
    pub primary_oam: Vec<u8>,          // Primary OAM (holds 64 sprites for the frame)
    secondary_oam: Vec<u8>,            // Secondary OAM (holds 8 sprites for the current scanline)
    sprite_attribute_latches: Vec<u8>, // 8 latches - These contain the attribute bytes [palette data] for up to 8 sprites.
    sprite_counters: Vec<u8>,          // 8 counters - These contain the X positions for up to 8 sprites.
    sprite_indexes: Vec<u8>,           // Indexes of the sprites-in-the-attribute-latches' within primary OAM
    sprite_pattern_table_srs: Vec<(u8, u8)>, // 8 pairs of 8-bit shift registers - These contain the pattern table data for up to 8 sprites, to be rendered on the current scanline.
    // Unused sprites are loaded with an all-transparent set of values.
    num_sprites: usize,                // Number of sprites in the shift registers for the current scanline

    // Various flags set by registers
    address_increment:             u16,
    sprite_pattern_table_base:     usize,
    background_pattern_table_base: usize,
    oam_address:                   usize,
    sprite_size:                   u8,
    grayscale:                     bool,
    show_background_left:          bool, // 1: Show background in leftmost 8 pixels of screen, 0: Hide
    show_sprites_left:             bool, // 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    show_background:               bool, // 1: Show background
    show_sprites:                  bool, // 1: Show sprites
    emphasize_red:                 bool, // Emphasize red
    emphasize_green:               bool, // Emphasize green
    emphasize_blue:                bool, // Emphasize blue
    sprite_overflow:               bool, // Set if there are more than 8 sprites on a single line
    sprite_zero_hit:               bool, // Set when the first pixel of the sprite in the zero index of primary OAM is rendered
    should_generate_nmi:           bool, // Allows CPU to control whether NMIs trigger
    vertical_blank:                bool, // true == in vertical blank, false == not

    // These three: god knows.
    // TODO: experiment more with NMI
    pub trigger_nmi:               bool, // triggers NMI in the CPU when it checks in its step()
    previous_nmi:                  bool,
    nmi_delay:                     usize,

    read_buffer:                   u8,   // used with PPUDATA register
    pub recent_bits:               u8,   // Least significant bits previously written into a PPU register

    previous_a12:                  u8,

    background_pixels:             [u8; 16],
    palette_offsets:               [u8; 16],
}

impl Ppu {
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
    	let mirroring_type = mapper.get_mirroring();
        Ppu {
            screen_buffer:                 vec![0; 256 * 240 * 3],
            line_cycle:                    0,
            scanline:                      0,
            frame:                         0,
            v:                             0,
            t:                             0,
            x:                             0,
            w:                             0,
            mapper:                        mapper,
            mirroring_type:                mirroring_type,
            cache:                         cache::Cache::new(),
            nametable_a:                   vec![0u8; 0x0400],
            nametable_b:                   vec![0u8; 0x0400],
            nametable_c:                   vec![0u8; 0x0400],
            nametable_d:                   vec![0u8; 0x0400],
            palette_ram:                   vec![0u8; 0x0020],
            background_pattern_sr_low:     0,
            background_pattern_sr_high:    0,
            nametable_byte:                0,
            attribute_table_byte:          0,
            low_pattern_table_byte:        0,
            high_pattern_table_byte:       0,
            background_palette_sr_low:     0,
            background_palette_sr_high:    0,
            background_palette_latch:      0,
            primary_oam:                   vec![0u8; 0x0100],
            secondary_oam:                 vec![0u8; 0x0020],
            sprite_attribute_latches:      vec![0u8; 8],
            sprite_counters:               vec![0u8; 8],
            sprite_indexes:                vec![0u8; 8],
            sprite_pattern_table_srs:      vec![(0u8, 0u8); 8],
            num_sprites:                   0,
            address_increment:             0,
            sprite_pattern_table_base:     0,
            background_pattern_table_base: 0,
            oam_address:                   0,
            sprite_size:                   0,
            grayscale:                     false,
            show_background_left:          false,
            show_sprites_left:             false,
            show_background:               false,
            show_sprites:                  false,
            emphasize_red:                 false,
            emphasize_green:               false,
            emphasize_blue:                false,
            sprite_overflow:               false,
            sprite_zero_hit:               false,
            should_generate_nmi:           false,
            vertical_blank:                false,
            trigger_nmi:                   false,
            previous_nmi:                  false,
            nmi_delay:                     0,
            read_buffer:                   0,
            recent_bits:                   0,
            previous_a12:                  0,
            background_pixels:             [0; 16],
            palette_offsets:               [0; 16],
        }
    }
}

const PALETTE_TABLE: [[u8; 3]; 64] = [
    [ 84,  84,  84], [  0,  30, 116], [  8,  16, 144], [ 48,   0, 136], [ 68,   0, 100], [ 92,   0,  48], [ 84,   4,   0], [ 60,  24,   0], [ 32,  42,   0], [  8,  58,   0], [  0,  64,   0], [  0,  60,   0], [  0,  50,  60], [  0,   0,   0], [  0,   0,   0], [  0,   0,   0],
    [152, 150, 152], [  8,  76, 196], [ 48,  50, 236], [ 92,  30, 228], [136,  20, 176], [160,  20, 100], [152,  34,  32], [120,  60,   0], [ 84,  90,   0], [ 40, 114,   0], [  8, 124,   0], [  0, 118,  40], [  0, 102, 120], [  0,   0,   0], [  0,   0,   0], [  0,   0,   0],
    [236, 238, 236], [ 76, 154, 236], [120, 124, 236], [176,  98, 236], [228,  84, 236], [236,  88, 180], [236, 106, 100], [212, 136,  32], [160, 170,   0], [116, 196,   0], [ 76, 208,  32], [ 56, 204, 108], [ 56, 180, 204], [ 60,  60,  60], [  0,   0,   0], [  0,   0,   0],
    [236, 238, 236], [168, 204, 236], [188, 188, 236], [212, 178, 236], [236, 174, 236], [236, 174, 212], [236, 180, 176], [228, 196, 144], [204, 210, 120], [180, 222, 120], [168, 226, 144], [152, 226, 180], [160, 214, 228], [160, 162, 160], [  0,   0,   0], [  0,   0,   0],
];
