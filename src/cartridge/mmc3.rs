use super::{Cartridge, Mapper, Mirror, serialize::*};

pub struct Mmc3 {
    cart: Cartridge,
    mirroring: Mirror,

    bank_registers: Vec<usize>,
    next_bank: u8,

    irq_latch: u8,
    irq_counter: u8,
    irq_enable: bool,
    trigger_irq: bool, // signal to send to CPU
    reload_counter: bool,
    irq_delay: u8,

    prg_ram_bank: Vec<u8>, // CPU $6000-$7FFF
    // 0: $8000-$9FFF swappable, $C000-$DFFF fixed to second-last bank
    // 1: $C000-$DFFF swappable, $8000-$9FFF fixed to second-last bank
    prg_rom_bank_mode: bool,
    // 0: two 2 KB banks at $0000-$0FFF, four 1 KB banks at $1000-$1FFF
    // 1: two 2 KB banks at $1000-$1FFF, four 1 KB banks at $0000-$0FFF

    chr_rom_bank_mode: bool,
    chr_ram_bank: Vec<u8>, // used if cartridge doesn't have any CHR-ROM, 8KB, $0000-$1FFF
}

impl Mmc3 {
    pub fn new(cart: Cartridge) -> Self {
        let m = cart.mirroring;
        Mmc3{
            cart: cart,
            mirroring: m,
            bank_registers: vec![0, 0, 0, 0, 0, 0, 0, 0],
            next_bank: 0,
            irq_latch: 0,
            irq_counter: 0,
            irq_enable: false,
            trigger_irq: false,
            reload_counter: false,
            irq_delay: 0,
            prg_ram_bank: vec![0; 0x2000],
            prg_rom_bank_mode: false,
            chr_rom_bank_mode: false,
            chr_ram_bank: vec![0; 0x2000],
        }
    }

    fn bank_select(&mut self, value: u8) {
        self.next_bank = value & 0b111;
        // ?? = value & (1<<5); // Nothing on the MMC3, see MMC6
        self.prg_rom_bank_mode = value & (1<<6) != 0;
        self.chr_rom_bank_mode = value & (1<<7) != 0;
    }

    fn bank_data(&mut self, value: u8) {
        // R6 and R7 will ignore the top two bits, as the MMC3 has only 6 PRG ROM address lines.
        // R0 and R1 ignore the bottom bit, as the value written still counts banks in 1KB units but odd numbered banks can't be selected.
        self.bank_registers[self.next_bank as usize] = match self.next_bank {
            0 | 1 => value & 0b1111_1110,
            6 | 7 => value & 0b0011_1111,
            _ => value,
        } as usize;
    }

    fn prg_ram_protect(&mut self) {}
}

impl Mapper for Mmc3 {
    fn read(&self, address: usize) -> u8 {
        let val = match address {
            0x0000..=0x1FFF => { // reading from CHR-ROM
                let offset_1k = address % 0x400;
                let offset_2k = address % 0x800;
                let bank_reg_num = match self.chr_rom_bank_mode {
                    true => {
                        match address {
                            0x0000..=0x03FF => 2,
                            0x0400..=0x07FF => 3,
                            0x0800..=0x0BFF => 4,
                            0x0C00..=0x0FFF => 5,
                            0x1000..=0x17FF => 0,
                            0x1800..=0x1FFF => 1,
                            _ => panic!("oh no"),
                        }
                    },
                    false => {
                        match address {
                            0x0000..=0x07FF => 0,
                            0x0800..=0x0FFF => 1,
                            0x1000..=0x13FF => 2,
                            0x1400..=0x17FF => 3,
                            0x1800..=0x1BFF => 4,
                            0x1C00..=0x1FFF => 5,
                            _ => panic!("oh no"),
                        }
                    },
                };
                let bank_num = self.bank_registers[bank_reg_num];
                let chunk_num = bank_num / 8;
                let chunk_eighth = (bank_num % 8) * 0x400;
                if bank_reg_num == 0 || bank_reg_num == 1 { // dealing with 2K banks of 8K chunks
                    self.cart.chr_rom[chunk_num][chunk_eighth + offset_2k]
                } else { // dealing with 1K banks of 8K chunks
                    self.cart.chr_rom[chunk_num][chunk_eighth + offset_1k]
                }
            },

            0x6000..=0x7FFF => self.prg_ram_bank[address % 0x2000], // PRG-RAM

            0x8000..=0xFFFF => { // reading from PRG ROM, dealing with 8K banks of 16K chunks
                let offset_8k = address % 0x2000;
                let num_banks = self.cart.prg_rom_size * 2;
                let bank_num = match self.prg_rom_bank_mode {
                    true => {
                        match address {
                            0x8000..=0x9FFF => num_banks - 2,
                            0xA000..=0xBFFF => self.bank_registers[7],
                            0xC000..=0xDFFF => self.bank_registers[6],
                            0xE000..=0xFFFF => num_banks - 1,
                            _ => panic!("oh no"),
                        }
                    },
                    false => {
                        match address {
                            0x8000..=0x9FFF => self.bank_registers[6],
                            0xA000..=0xBFFF => self.bank_registers[7],
                            0xC000..=0xDFFF => num_banks - 2,
                            0xE000..=0xFFFF => num_banks - 1,
                            _ => panic!("oh no"),
                        }
                    },
                };
                let chunk_num = bank_num / 2;
                let chunk_half = (bank_num % 2) * 0x2000;
                self.cart.prg_rom[chunk_num][chunk_half + offset_8k]

            },
            _ => {
                println!("bad address read from MMC3: 0x{:X}", address);
                0
            },
        };
        val
    }

    fn write(&mut self, address: usize, value: u8) {
        if (0..=0x1FFF).contains(&address) {
            if self.cart.chr_rom_size == 0 {
                self.chr_ram_bank[address] = value;
            }
            return
        }
        match address % 2 == 0 {
            true => { // even
                match address {
                    0x6000..=0x7FFF => self.prg_ram_bank[address % 0x2000] = value, // PRG-RAM
                    0x8000..=0x9FFF => self.bank_select(value),
                    0xA000..=0xBFFF => self.mirroring = if value & 1 == 0 {Mirror::Vertical} else {Mirror::Horizontal},
                    0xC000..=0xDFFF => self.irq_latch = value,
                    0xE000..=0xFFFF => {self.irq_enable = false; self.trigger_irq = false}, // Writing any value to this register will disable MMC3 interrupts AND acknowledge any pending interrupts.
                    _ => println!("bad address written to MMC3: 0x{:X}", address),
                }
            },
            false => { // odd
                match address {
                    0x6000..=0x7FFF => self.prg_ram_bank[address % 0x2000] = value, // PRG-RAM
                    0x8000..=0x9FFF => self.bank_data(value),
                    0xA000..=0xBFFF => self.prg_ram_protect(),
                    0xC000..=0xDFFF => self.reload_counter = true, // Writing any value to this register reloads the MMC3 IRQ counter at the NEXT rising edge of the PPU address, presumably at PPU cycle 260 of the current scanline.
                    0xE000..=0xFFFF => self.irq_enable = true,
                    _ => println!("bad address written to MMC3: 0x{:X}", address),
                }
            },
        }
    }

    fn get_mirroring(&self) -> Mirror {
        if self.cart.four_screen_vram {
            Mirror::FourScreen
        } else {
            self.mirroring
        }
    }

    fn load_battery_backed_ram(&mut self) {}
    fn save_battery_backed_ram(&self) {}

    // This function is called by the PPU when the A12 address line changes.
    // It's supposed to only be called when A12 goes from 0 to 1, but that doesn't work
    // for my emulator for some reason.
    fn clock(&mut self) {
        if self.reload_counter {
            self.irq_counter = self.irq_latch;
            self.reload_counter = false;
        }
        if self.irq_counter == 0 {
            self.irq_counter = self.irq_latch;
        } else {
            self.irq_counter -= 1;
        }
        if self.irq_counter == 0 && self.irq_enable {
            self.trigger_irq = true;
        }
    }

    // This function is called by the CPU every step (which takes more than one CPU clock cycle).
    // I think I'm supposed to be tracking IRQ delays by the PPU, not letting an IRQ fire if
    // there was one within the last 15 PPU cycles, but that didn't work and this does.
    fn check_irq(&mut self) -> bool {
        if self.trigger_irq {
            self.trigger_irq = false;
            if self.irq_delay == 0 {
                self.irq_delay = 5;
            }
        }
        if self.irq_delay > 0 {
            self.irq_delay -= 1;
            if self.irq_delay == 0 {
                return true;
            }
        }
        false
    }

    fn save_state(&self) -> MapperData {
        MapperData::Mmc3(
            Mmc3Data {
                cart: self.cart.clone(),
                mirroring: self.mirroring,
                bank_registers: self.bank_registers.clone(),
                next_bank: self.next_bank,
                irq_latch: self.irq_latch,
                irq_counter: self.irq_counter,
                irq_enable: self.irq_enable,
                trigger_irq: self.trigger_irq,
                reload_counter: self.reload_counter,
                irq_delay: self.irq_delay,
                prg_ram_bank: self.prg_ram_bank.clone(),
                prg_rom_bank_mode: self.prg_rom_bank_mode,
                chr_rom_bank_mode: self.chr_rom_bank_mode,
                chr_ram_bank: self.chr_ram_bank.clone(),
            }
        )
    }

    fn load_state(&mut self, mapper_data: MapperData) {
        if let MapperData::Mmc3(mmc3_data) = mapper_data {
            self.cart = mmc3_data.cart;
            self.mirroring = mmc3_data.mirroring;
            self.bank_registers = mmc3_data.bank_registers;
            self.next_bank = mmc3_data.next_bank;
            self.irq_latch = mmc3_data.irq_latch;
            self.irq_counter = mmc3_data.irq_counter;
            self.irq_enable = mmc3_data.irq_enable;
            self.trigger_irq = mmc3_data.trigger_irq;
            self.reload_counter = mmc3_data.reload_counter;
            self.irq_delay = mmc3_data.irq_delay;
            self.prg_ram_bank = mmc3_data.prg_ram_bank;
            self.prg_rom_bank_mode = mmc3_data.prg_rom_bank_mode;
            self.chr_rom_bank_mode = mmc3_data.chr_rom_bank_mode;
            self.chr_ram_bank = mmc3_data.chr_ram_bank;
        }
    }
}
