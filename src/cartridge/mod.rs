mod nrom;
mod mmc1;
mod uxrom;
mod cnrom;
mod mmc3;
pub mod serialize;

use nrom::Nrom;
use mmc1::Mmc1;
use uxrom::Uxrom;
use cnrom::Cnrom;
use mmc3::Mmc3;

use std::fs::File;
use std::io::Read;

pub trait Mapper {
    fn read(&self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8);
    fn get_mirroring(&self) -> Mirror;
    fn load_battery_backed_ram(&mut self);
    fn save_battery_backed_ram(&self);
    fn clock(&mut self);
    fn check_irq(&mut self) -> bool;
    fn save_state(&self) -> serialize::MapperData;
    fn load_state(&mut self, mapper_data: serialize::MapperData);
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Mirror {
    LowBank,
    HighBank,
    Horizontal,
    Vertical,
    FourScreen,
}

pub fn get_mapper(filename: String) -> Box<dyn Mapper> {
    let cart = Cartridge::new(filename);
    let num = cart.mapper_num;
    match num {
        0 => Box::new(Nrom::new(cart)),
        1 => Box::new(Mmc1::new(cart)),
        2 => Box::new(Uxrom::new(cart)),
        3 => Box::new(Cnrom::new(cart)),
        4 => Box::new(Mmc3::new(cart)),
        _ => panic!("unimplemented mapper: {}", num),
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Cartridge {
    filename: String,
    prg_rom_size: usize,
    chr_rom_size: usize,
    pub mirroring: Mirror, // 0 horizontal, 1 vertical
    battery_backed_ram: bool, // 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
    trainer_present: bool, // 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
    four_screen_vram: bool, // 1: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
    // TODO: other iNES header flags

    pub prg_rom: Vec<Vec<u8>>, // 16 KiB chunks for CPU
    pub chr_rom: Vec<Vec<u8>>, // 8 KiB chunks for PPU

    all_data: Vec<u8>,
    mapper_num: u8,
}

impl Cartridge {
    pub fn new(filename: String) -> Self {
        let mut f = std::fs::File::open(&filename).expect("could not open {}");
        let mut data = vec![];
        f.read_to_end(&mut data).unwrap();
        assert!(data[0..4] == [0x4E, 0x45, 0x53, 0x1A], "signature mismatch, not an iNES file");
        let mapper_num = ((data[7] >> 4) << 4) + (data[6] >> 4);
        let mut cart = Cartridge {
            filename: filename.to_string(),
            prg_rom_size: data[4] as usize,
            chr_rom_size: data[5] as usize,
            mirroring:       if data[6] & (1 << 0) == 0 {Mirror::Horizontal} else {Mirror::Vertical},
            battery_backed_ram: data[6] & (1 << 1) != 0,
            trainer_present:    data[6] & (1 << 2) != 0,
            four_screen_vram:   data[6] & (1 << 3) != 0,
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
        let prg_offset: usize = 0x10 + if self.trainer_present { 0x200 } else { 0 }; // header plus trainer if present
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
        self.all_data.clear();
    }

}

pub fn check_signature(filename: &str) -> Result<(), String> {
    let mut f = File::open(filename).map_err(|e| e.to_string())?;
    let mut data = [0; 4];
    f.read_exact(&mut data).map_err(|e| e.to_string())?;
    if data == [0x4E, 0x45, 0x53, 0x1A] {
        Ok(())
    } else {
        Err("file signature mismatch: not a valid iNES file".to_string())
    }
}

/*
The mappings above are the fixed addresses from which the PPU uses to fetch data during rendering. The actual device that the PPU fetches data from, however, may be configured by the cartridge.
    $0000-1FFF is normally mapped by the cartridge to a CHR-ROM or CHR-RAM, often with a bank switching mechanism.
    $2000-2FFF is normally mapped to the 2kB NES internal VRAM, providing 2 nametables with a mirroring configuration controlled by the cartridge, but it can be partly or fully remapped to RAM on the cartridge, allowing up to 4 simultaneous nametables.
    $3000-3EFF is usually a mirror of the 2kB region from $2000-2EFF. The PPU does not render from this address range, so this space has negligible utility.
    $3F00-3FFF is not configurable, always mapped to the internal palette control.
*/
