mod nrom;
mod mmc1;

use nrom::Nrom;
use mmc1::Mmc1;

use std::cell::RefCell;
use std::rc::Rc;
use std::io::Read;

pub trait Mapper {
    fn read(&mut self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8);
    fn get_mirroring(&mut self) -> Mirror;
}

#[derive(Copy, Clone, PartialEq)]
pub enum Mirror {
    LowBank,
    HighBank,
    Horizontal,
    Vertical,
}

// To avoid separate read and write functions for every mapper, the mapper functions returns a reference to the CPU or PPU's own
// byte of memory, which the calling method can dereference to read or set the value.
pub type CpuMapperFunc = fn(&mut crate::cpu::Cpu, usize, bool) -> Option<&mut u8>;
pub type PpuMapperFunc = fn(&mut crate::ppu::Ppu, usize, bool) -> Option<&mut u8>;

pub fn get_mapper() -> Rc<RefCell<dyn Mapper>> {
    let cart = Cartridge::new();
    let num = cart.mapper_num;
    match num {
        0 => Rc::new(RefCell::new(Nrom::new(cart))),
        1 => Rc::new(RefCell::new(Mmc1::new(cart))),
        _ => panic!("unimplemented mapper: {}", num),
    }
}

pub struct Cartridge {
    prg_rom_size: usize,
    chr_rom_size: usize,
    pub mirroring: Mirror, // 0 horizontal, 1 vertical
    _bb_prg_ram_present: u8, // 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
    trainer_present: u8, // 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
    _four_screen_vram: u8, // 1: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
    // TODO: other iNES header flags

    pub prg_rom: Vec<Vec<u8>>, // 16 KiB chunks for CPU
    pub chr_rom: Vec<Vec<u8>>, // 8 KiB chunks for PPU

    all_data: Vec<u8>,
    mapper_num: u8,
}

impl Cartridge {
    pub fn new() -> Self {
        let argv: Vec<String> = std::env::args().collect();
        assert!(argv.len() > 1, "must include .nes ROM as argument");
        let filename = &argv[1];
        let mut f = std::fs::File::open(filename).unwrap();
        let mut data = vec![];
        f.read_to_end(&mut data).unwrap();
        assert!(data[0..4] == [0x4E, 0x45, 0x53, 0x1A], "signature mismatch, not an iNES file");
        let mapper_num = ((data[7] >> 4) << 4) + (data[6] >> 4);
        let mut cart = Cartridge {
            prg_rom_size: data[4] as usize,
            chr_rom_size: data[5] as usize,
            mirroring:         if data[6] & (1 << 0) == 0 {Mirror::Horizontal} else {Mirror::Vertical},
            _bb_prg_ram_present: (data[6] & (1 << 1) != 0) as u8,
            trainer_present:     (data[6] & (1 << 2) != 0) as u8,
            _four_screen_vram:   (data[6] & (1 << 3) != 0) as u8,
            prg_rom: Vec::new(),
            chr_rom: Vec::new(),
            all_data: data,
            mapper_num: mapper_num,
        };
        cart.fill();
        cart
    }

    fn fill(&mut self) {
        let prg_chunk_size: usize = 1<<14;
        let chr_chunk_size: usize = 1<<13;
        let prg_offset: usize = 0x10 + if self.trainer_present == 1 { 0x200 } else { 0 }; // header plus trainer if present
        let chr_offset: usize = prg_offset + (self.prg_rom_size * prg_chunk_size); // chr comes after prg
        // fill vecs with chunks
        for i in 0..self.prg_rom_size {
            let offset = prg_offset + (i * prg_chunk_size);
            let chunk = self.all_data[offset..(offset + prg_chunk_size)].to_vec();
            self.prg_rom.push(chunk.clone());
        };
        for i in 0..self.chr_rom_size {
            let offset = chr_offset + (i * chr_chunk_size);
            let chunk = self.all_data[offset..offset + chr_chunk_size].to_vec();
            self.chr_rom.push(chunk);
        };
    }
}

/*
The mappings above are the fixed addresses from which the PPU uses to fetch data during rendering. The actual device that the PPU fetches data from, however, may be configured by the cartridge.
    $0000-1FFF is normally mapped by the cartridge to a CHR-ROM or CHR-RAM, often with a bank switching mechanism.
    $2000-2FFF is normally mapped to the 2kB NES internal VRAM, providing 2 nametables with a mirroring configuration controlled by the cartridge, but it can be partly or fully remapped to RAM on the cartridge, allowing up to 4 simultaneous nametables.
    $3000-3EFF is usually a mirror of the 2kB region from $2000-2EFF. The PPU does not render from this address range, so this space has negligible utility.
    $3F00-3FFF is not configurable, always mapped to the internal palette control.
*/
