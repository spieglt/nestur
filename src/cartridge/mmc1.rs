use super::{Cartridge, Mapper, Mirror};

pub struct Mmc1 {
    cart: Cartridge,
    step: u8,
    shift_register: u8,
    prg_low_bank: usize,
    prg_high_bank: usize,
    chr_low_bank: usize,
    chr_high_bank: usize,
    mirroring: Mirror,
}

impl Mmc1 {
    fn new(cart: Cartridge) -> Self {
        Mmc1
    }
}

impl Mapper for Mmc1 {
    fn read(&mut self, address: usize) -> u8 {
        0
    }

    fn write(&mut self, address: usize, value: u8) {

    }
}
