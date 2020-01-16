use super::{Cartridge, Mapper, Mirror};

pub struct Mmc3 {
    cart: Cartridge,

    reg0: u8,
    reg1: u8,
    reg2: u8,
    reg3: u8,
    reg4: u8,
    reg5: u8,
    reg6: u8,
    reg7: u8,

    irq_counter: u8,
    irq_latch: u8,
}

impl Mmc3 {
    pub fn new(cart: Cartridge) -> Self {
        Mmc3{
            cart: cart,

            reg0: 0,
            reg1: 0,
            reg2: 0,
            reg3: 0,
            reg4: 0,
            reg5: 0,
            reg6: 0,
            reg7: 0,
        }
    }
}

impl Mmc3 {

}

impl Mapper for Mmc3 {
    fn read(&mut self, address: usize) -> u8 {
        match address {
            0x0000..=0x07FF => ,
            0x0800..=0x0FFF => ,
            0x1000..=0x13FF => ,
            0x1400..=0x17FF => ,
            0x1800..=0x1BFF => ,
            0x1C00..=0x1FFF => ,
            0x6000..=0x7FFF => ,
            0x6000..=0x7FFF => ,
            0x6000..=0x7FFF => ,
            0x6000..=0x7FFF => ,
            0x6000..=0x7FFF => ,
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address % 2 == 0 {
            true => { // even
                match address {
                    0x8000..=0x9FFF => self.bank_select(address),
                    0xA000..=0xBFFF => self.mirroring = if value & 1 == 0 {Mirror::Vertical} else {Mirror::Horizontal},
                    0xC000..=0xDFFF => ,
                    0x1400..=0x17FF => ,
                    0x1800..=0x1BFF => ,
                    0x1C00..=0x1FFF => ,
                    0x6000..=0x7FFF => ,
                    0x6000..=0x7FFF => ,
                    0x6000..=0x7FFF => ,
                    0x6000..=0x7FFF => ,
                    0x6000..=0x7FFF => ,
                }
            },
            false => { // odd
                match address {
                    0x8000..=0x9FFF => self.bank_data(value),
                    0xA000..=0xBFFF => self.prg_ram_protect(value),
                    0xC000..=0xDFFF => ,
                    0x1400..=0x17FF => ,
                    0x1800..=0x1BFF => ,
                    0x1C00..=0x1FFF => ,
                    0x6000..=0x7FFF => ,
                    0x6000..=0x7FFF => ,
                    0x6000..=0x7FFF => ,
                    0x6000..=0x7FFF => ,
                    0x6000..=0x7FFF => ,
                }
            },
    }

    fn get_mirroring(&mut self) -> Mirror {}

    fn load_battery_backed_ram(&mut self) {}

    fn save_battery_backed_ram(&self) {}
}
