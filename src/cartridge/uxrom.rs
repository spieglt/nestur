use super::{Cartridge, Mapper, Mirror, serialize::*};

pub struct Uxrom {
    cart: Cartridge,
    chr_ram: Vec<u8>,
    bank_select: usize,
}

impl Uxrom {
    pub fn new(cart: Cartridge) -> Self {
        Uxrom{
            cart: cart,
            chr_ram: vec![0; 0x2000],
            bank_select: 0,
        }
    }
}

impl Mapper for Uxrom {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                if self.cart.chr_rom_size > 0 {
                    self.cart.chr_rom[0][address]
                } else {
                    self.chr_ram[address]
                }
            },
            0x8000..=0xBFFF => self.cart.prg_rom[self.bank_select][address % 0x4000],
            0xC000..=0xFFFF => self.cart.prg_rom[self.cart.prg_rom.len()-1][address % 0x4000],
            _ => {println!("bad address read from UxROM mapper: 0x{:X}", address); 0},
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                if self.cart.chr_rom_size == 0 {
                    self.chr_ram[address] = value;
                }
            },
            0x8000..=0xFFFF => self.bank_select = value as usize,
            _ => println!("bad address written to UxROM mapper: 0x{:X}", address),
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
        MapperData::Uxrom(
            UxromData {
                cart: self.cart.clone(),
                chr_ram: self.chr_ram.clone(),
                bank_select: self.bank_select,
            }
        )
    }

    fn load_state(&mut self, mapper_data: MapperData) {
        if let MapperData::Uxrom(uxrom_data) = mapper_data {
            self.cart = uxrom_data.cart;
            self.chr_ram = uxrom_data.chr_ram;
            self.bank_select = uxrom_data.bank_select;
        }
    }
}
