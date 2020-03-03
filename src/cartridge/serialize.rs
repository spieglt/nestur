use super::{Cartridge, Mirror};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum MapperData {
    Nrom(NromData),
    Mmc1(Mmc1Data),
    Uxrom(UxromData),
    Cnrom(CnromData),
    Mmc3(Mmc3Data),
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct NromData {
    pub cart: Cartridge,
    pub chr_ram: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Mmc1Data {
    pub cart: Cartridge,
    pub step: u8,
    pub shift_register: u8,
    pub mirroring: Mirror,
    pub control: u8,
    pub prg_ram_bank: Vec<u8>,
    pub prg_ram_enabled: bool,
    pub prg_bank_mode: u8,
    pub prg_bank_select: usize,
    pub chr_ram_bank: Vec<u8>,
    pub chr_low_bank: usize,
    pub chr_high_bank: usize,
    pub chr_bank_mode: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UxromData {
    pub cart: Cartridge,
    pub chr_ram: Vec<u8>,
    pub bank_select: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CnromData {
    pub cart: Cartridge,
    pub chr_bank_select: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Mmc3Data {
    pub cart: Cartridge,
    pub mirroring: Mirror,
    pub bank_registers: Vec<usize>,
    pub next_bank: u8,
    pub irq_latch: u8,
    pub irq_counter: u8,
    pub irq_enable: bool,
    pub trigger_irq: bool,
    pub reload_counter: bool,
    pub irq_delay: u8,
    pub prg_ram_bank: Vec<u8>,
    pub prg_rom_bank_mode: bool,
    pub chr_rom_bank_mode: bool,
    pub chr_ram_bank: Vec<u8>, 
}
