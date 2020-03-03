use super::{Cartridge, Mapper, Mirror, serialize::*};

pub struct Cnrom {
    cart: Cartridge,
    chr_bank_select: usize,
}

impl Cnrom {
    pub fn new(cart: Cartridge) -> Self {
        Cnrom{
            cart: cart,
            chr_bank_select: 0,
        }
    }
}

impl Mapper for Cnrom {
    fn read(&self, address: usize) -> u8 {
        let pl = self.cart.prg_rom.len();
        let addr = address % 0x4000;
        match address {
            0x0000..=0x1FFF => self.cart.chr_rom[self.chr_bank_select][address],
            0x8000..=0xBFFF => self.cart.prg_rom[0][addr],
            0xC000..=0xFFFF => self.cart.prg_rom[pl-1][addr],
            _ => {println!("bad address read from CNROM mapper: 0x{:X}", address); 0},
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x8000..=0xFFFF => self.chr_bank_select = (value & 0b11) as usize,
            _ => println!("bad address written to CNROM mapper: 0x{:X}", address),
        }
    }

    fn get_mirroring(&self) -> Mirror {
        self.cart.mirroring
    }

    fn load_battery_backed_ram(&mut self) {}
    fn save_battery_backed_ram(&self) {}
    fn clock(&mut self) {}
    fn check_irq(&mut self) -> bool {false}

    fn save_state(&self) -> MapperData {
        MapperData::Cnrom(
            CnromData {
                cart: self.cart.clone(),
                chr_bank_select: self.chr_bank_select,
            }
        )
    }

    fn load_state(&mut self, mapper_data: MapperData) {
        if let MapperData::Cnrom(cnrom_data) = mapper_data {
            self.cart = cnrom_data.cart;
            self.chr_bank_select = cnrom_data.chr_bank_select;
        }
    }
}
