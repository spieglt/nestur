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
    pub fn new(cart: Cartridge) -> Self {
        let m = cart.mirroring;
        Mmc1 {
            cart: cart,
            step: 0,
            shift_register: 0,
            prg_low_bank: 0,
            prg_high_bank: 0,
            chr_low_bank: 0,
            chr_high_bank: 0,
            mirroring: m,
        }
    }
}

impl Mapper for Mmc1 {
    fn read(&mut self, address: usize) -> u8 {
        0
    }

    fn write(&mut self, address: usize, value: u8) {

    }

    fn get_mirroring(&mut self) -> Mirror {
        self.mirroring
    }
}
