use super::{Cartridge, Mapper, Mirror};

pub struct Nrom {
    cart: Cartridge,
    chr_ram: Vec<u8>,
}

impl Nrom {
    pub fn new(cart: Cartridge) -> Self {
        Nrom{
            cart: cart,
            chr_ram: vec![0; 0x2000],
        }
    }
}

impl Mapper for Nrom {
    fn read(&self, address: usize) -> u8 {
        let addr = address % 0x4000;
        match address {
            0x0000..=0x1FFF => {
                if self.cart.chr_rom_size > 0 {
                    self.cart.chr_rom[0][address]
                } else {
                    self.chr_ram[address]
                }
            },
            0x8000..=0xBFFF => {
                self.cart.prg_rom[0][addr]
            },
            0xC000..=0xFFFF => {
                self.cart.prg_rom[self.cart.prg_rom_size - 1][addr]
            },
            _ => {println!("bad address read from NROM mapper: 0x{:X}", address); 0},
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // ROM isn't written to
                if self.cart.chr_rom_size == 0 {
                    self.chr_ram[address] = value;
                }
            },
            0x8000..=0xBFFF => (),
            0xC000..=0xFFFF => (),
            _ => println!("bad address written to NROM mapper: 0x{:X}", address),
        }
    }

    fn get_mirroring(&self) -> Mirror {
        self.cart.mirroring
    }

    fn load_battery_backed_ram(&mut self) {}
    fn save_battery_backed_ram(&self) {}
    fn clock(&mut self) {}
    fn check_irq(&mut self) -> bool {false}
}
