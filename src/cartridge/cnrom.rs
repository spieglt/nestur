use super::{Cartridge, Mapper, Mirror};

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
    fn read(&mut self, address: usize) -> u8 {
        let cl = self.cart.chr_rom.len();
        let pl = self.cart.prg_rom.len();
        let addr = address % 0x4000;
        match address {
            0x0000..=0x1FFF => self.cart.chr_rom[self.chr_bank_select][address],
            0x8000..=0xBFFF => self.cart.prg_rom[0][addr],
            0xC000..=0xFFFF => self.cart.prg_rom[pl-1][addr],
            _ => panic!("bad address read from CNROM mapper: 0x{:X}", address),
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x8000..=0xFFFF => self.chr_bank_select = (value & 0b11) as usize,
            _ => println!("bad address written to CNROM mapper: 0x{:X}", address),
        }
    }

    fn get_mirroring(&mut self) -> Mirror {
        self.cart.mirroring
    }

    fn load_battery_backed_ram(&mut self) {}
    fn save_battery_backed_ram(&self) {}
    fn clock(&mut self) {}
    fn check_irq(&mut self) -> bool {false}
}
