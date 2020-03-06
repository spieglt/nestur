use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PpuData {
    line_cycle: usize,
    scanline: usize,
    frame: usize,
    v: u16,
    t: u16,
    x: u8,
    w: u8,
    nametable_a: Vec<u8>,
    nametable_b: Vec<u8>,
    nametable_c: Vec<u8>,
    nametable_d: Vec<u8>,
    palette_ram: Vec<u8>,
    background_pattern_sr_low: u16,
    background_pattern_sr_high: u16,
    nametable_byte: u8,
    attribute_table_byte: u8,
    low_pattern_table_byte: u8,
    high_pattern_table_byte: u8,
    background_palette_sr_low: u8,
    background_palette_sr_high: u8,
    background_palette_latch: u8,
    primary_oam: Vec<u8>,
    secondary_oam: Vec<u8>,
    sprite_attribute_latches: Vec<u8>,
    sprite_counters: Vec<u8>,
    sprite_indexes: Vec<u8>,
    sprite_pattern_table_srs: Vec<(u8, u8)>,
    num_sprites: usize,
    address_increment: u16,
    sprite_pattern_table_base: usize,
    background_pattern_table_base:usize,
    oam_address: usize,
    sprite_size: u8,
    grayscale: bool,
    show_background_left: bool,
    show_sprites_left: bool,
    show_background: bool,
    show_sprites: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
    sprite_overflow: bool,
    sprite_zero_hit: bool,
    should_generate_nmi: bool,
    vertical_blank: bool,
    trigger_nmi: bool,
    previous_nmi: bool,
    nmi_delay: usize,
    read_buffer: u8,
    recent_bits: u8,
    previous_a12: u8,
}

impl super::Ppu {
    pub fn save_state(&self) -> PpuData {
        PpuData{
            line_cycle: self.line_cycle,
            scanline: self.scanline,
            frame: self.frame,
            v: self.v,
            t: self.t,
            x: self.x,
            w: self.w,
            nametable_a: self.nametable_a.clone(),
            nametable_b: self.nametable_b.clone(),
            nametable_c: self.nametable_c.clone(),
            nametable_d: self.nametable_d.clone(),
            palette_ram: self.palette_ram.clone(),
            background_pattern_sr_low: self.background_pattern_sr_low,
            background_pattern_sr_high: self.background_pattern_sr_high,
            nametable_byte: self.nametable_byte,
            attribute_table_byte: self.attribute_table_byte,
            low_pattern_table_byte: self.low_pattern_table_byte,
            high_pattern_table_byte: self.high_pattern_table_byte,
            background_palette_sr_low: self.background_palette_sr_low,
            background_palette_sr_high: self.background_palette_sr_high,
            background_palette_latch: self.background_palette_latch,
            primary_oam: self.primary_oam.clone(),
            secondary_oam: self.secondary_oam.clone(),
            sprite_attribute_latches: self.sprite_attribute_latches.clone(),
            sprite_counters: self.sprite_counters.clone(),
            sprite_indexes: self.sprite_indexes.clone(),
            sprite_pattern_table_srs: self.sprite_pattern_table_srs.clone(),
            num_sprites: self.num_sprites,
            address_increment: self.address_increment,
            sprite_pattern_table_base: self.sprite_pattern_table_base,
            background_pattern_table_base: self.background_pattern_table_base,
            oam_address: self.oam_address,
            sprite_size: self.sprite_size,
            grayscale: self.grayscale,
            show_background_left: self.show_background_left,
            show_sprites_left: self.show_sprites_left,
            show_background: self.show_background,
            show_sprites: self.show_sprites,
            emphasize_red: self.emphasize_red,
            emphasize_green: self.emphasize_green,
            emphasize_blue: self.emphasize_blue,
            sprite_overflow: self.sprite_overflow,
            sprite_zero_hit: self.sprite_zero_hit,
            should_generate_nmi: self.should_generate_nmi,
            vertical_blank: self.vertical_blank,
            trigger_nmi: self.trigger_nmi,
            previous_nmi: self.previous_nmi,
            nmi_delay: self.nmi_delay,
            read_buffer: self.read_buffer,
            recent_bits: self.recent_bits,
            previous_a12: self.previous_a12,
        }
    }

    pub fn load_state(&mut self, data: PpuData) {
        self.line_cycle = data.line_cycle;
        self.scanline = data.scanline;
        self.frame = data.frame;
        self.v = data.v;
        self.t = data.t;
        self.x = data.x;
        self.w = data.w;
        self.nametable_a = data.nametable_a;
        self.nametable_b = data.nametable_b;
        self.nametable_c = data.nametable_c;
        self.nametable_d = data.nametable_d;
        self.palette_ram = data.palette_ram;
        self.background_pattern_sr_low = data.background_pattern_sr_low;
        self.background_pattern_sr_high = data.background_pattern_sr_high;
        self.nametable_byte = data.nametable_byte;
        self.attribute_table_byte = data.attribute_table_byte;
        self.low_pattern_table_byte = data.low_pattern_table_byte;
        self.high_pattern_table_byte = data.high_pattern_table_byte;
        self.background_palette_sr_low = data.background_palette_sr_low;
        self.background_palette_sr_high = data.background_palette_sr_high;
        self.background_palette_latch = data.background_palette_latch;
        self.primary_oam = data.primary_oam;
        self.secondary_oam = data.secondary_oam;
        self.sprite_attribute_latches = data.sprite_attribute_latches;
        self.sprite_counters = data.sprite_counters;
        self.sprite_indexes = data.sprite_indexes;
        self.sprite_pattern_table_srs = data.sprite_pattern_table_srs;
        self.num_sprites = data.num_sprites;
        self.address_increment = data.address_increment;
        self.sprite_pattern_table_base = data.sprite_pattern_table_base;
        self.background_pattern_table_base = data.background_pattern_table_base;
        self.oam_address = data.oam_address;
        self.sprite_size = data.sprite_size;
        self.grayscale = data.grayscale;
        self.show_background_left = data.show_background_left;
        self.show_sprites_left = data.show_sprites_left;
        self.show_background = data.show_background;
        self.show_sprites = data.show_sprites;
        self.emphasize_red = data.emphasize_red;
        self.emphasize_green = data.emphasize_green;
        self.emphasize_blue = data.emphasize_blue;
        self.sprite_overflow = data.sprite_overflow;
        self.sprite_zero_hit = data.sprite_zero_hit;
        self.should_generate_nmi = data.should_generate_nmi;
        self.vertical_blank = data.vertical_blank;
        self.trigger_nmi = data.trigger_nmi;
        self.previous_nmi = data.previous_nmi;
        self.nmi_delay = data.nmi_delay;
        self.read_buffer = data.read_buffer;
        self.recent_bits = data.recent_bits;
        self.previous_a12 = data.previous_a12;
    }
}
