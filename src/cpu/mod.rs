mod addressing_modes;
mod opcodes;
mod utility;

// RAM locations
const STACK_OFFSET: usize = 0x100;
const NMI_VECTOR: usize = 0xFFFA;
const RESET_VECTOR: usize = 0xFFFC;
const IRQ_VECTOR: usize = 0xFFFE;

// status register flags
const CARRY_FLAG: u8             = 1 << 0;
const ZERO_FLAG: u8              = 1 << 1;
const INTERRUPT_DISABLE_FLAG: u8 = 1 << 2;
const DECIMAL_FLAG: u8           = 1 << 3;
// bits 4 and 5 are unused except when status register is copied to stack
const OVERFLOW_FLAG: u8          = 1 << 6;
const NEGATIVE_FLAG: u8          = 1 << 7;

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    ABS, ABX, ABY, ACC,
    IMM, IMP, IDX, IND,
    INX, REL, ZPG, ZPX,
    ZPY,
}

type AddressingFunction = fn(&mut Cpu) -> usize;

impl Mode {
    fn get(&self) -> AddressingFunction {
        match self {
            Mode::ABS => Cpu::absolute,
            Mode::ABX => Cpu::absolute_x,
            Mode::ABY => Cpu::absolute_y,
            Mode::ACC => Cpu::accumulator,
            Mode::IMM => Cpu::immediate,
            Mode::IMP => Cpu::implied,
            Mode::IDX => Cpu::indexed_indirect,
            Mode::IND => Cpu::indirect,
            Mode::INX => Cpu::indirect_indexed,
            Mode::REL => Cpu::relative,
            Mode::ZPG => Cpu::zero_page,
            Mode::ZPX => Cpu::zero_page_x,
            Mode::ZPY => Cpu::zero_page_y,
        }
    }
}


pub struct Cpu {
    mem: Vec<u8>,
    A: u8,         // accumulator
    X: u8,         // general purpose
    Y: u8,         // general purpose
    PC: usize,     // 16-bit program counter
    S: u8,         // stack pointer
    P: u8,         // status

    // number of ticks in current cycle
    clock: u64,

    // for skipping cycles during OAM DMA
    delay: usize,

    // function table
    opcode_table: Vec<fn(&mut Self, usize, Mode)>,

    // address mode table
    mode_table: Vec<Mode>,

    // cartridge data
    pub prg_rom: Vec<Vec<u8>>, // one 16 KiB chunk for each specified in iNES header
    mapper_func: crate::cartridge::CpuMapperFunc,

    // ppu
    pub ppu: super::Ppu,

    // apu
    pub apu: super::Apu,

    // controller
    pub strobe: u8,
    pub button_states: u8, // Player 1 controller
    button_number: u8,

    more: usize,
}

impl Cpu {
    pub fn new(cart: &super::Cartridge, ppu: super::Ppu, apu: super::Apu) -> Self {
        let mut cpu = Cpu{
            mem: vec![0; 0x2000],
            A: 0, X: 0, Y: 0,
            PC: 0,
            S: 0xFD,
            P: 0x24, // TODO: change this back to 0x34? nestest.nes ROM has it as 0x24 at start.
            clock: 0,
            delay: 0,
            prg_rom: cart.prg_rom.clone(),
            mapper_func: cart.cpu_mapper_func,
            ppu: ppu,
            apu: apu,
            strobe: 0,
            button_states: 0,
            button_number: 0,
            more: 0,
            opcode_table: vec![
        //         00        01        02        03        04        05        06        07        08        09        0A        0B        0C        0D        0E        0F
        /*00*/  Cpu::brk, Cpu::ora, Cpu::bad, Cpu::slo, Cpu::nop, Cpu::ora, Cpu::asl, Cpu::slo, Cpu::php, Cpu::ora, Cpu::asl, Cpu::nop, Cpu::nop, Cpu::ora, Cpu::asl, Cpu::slo,  /*00*/
        /*10*/  Cpu::bpl, Cpu::ora, Cpu::bad, Cpu::slo, Cpu::nop, Cpu::ora, Cpu::asl, Cpu::slo, Cpu::clc, Cpu::ora, Cpu::nop, Cpu::slo, Cpu::nop, Cpu::ora, Cpu::asl, Cpu::slo,  /*10*/
        /*20*/  Cpu::jsr, Cpu::and, Cpu::bad, Cpu::rla, Cpu::bit, Cpu::and, Cpu::rol, Cpu::rla, Cpu::plp, Cpu::and, Cpu::rol, Cpu::nop, Cpu::bit, Cpu::and, Cpu::rol, Cpu::rla,  /*20*/
        /*30*/  Cpu::bmi, Cpu::and, Cpu::bad, Cpu::rla, Cpu::nop, Cpu::and, Cpu::rol, Cpu::rla, Cpu::sec, Cpu::and, Cpu::nop, Cpu::rla, Cpu::nop, Cpu::and, Cpu::rol, Cpu::rla,  /*30*/
        /*40*/  Cpu::rti, Cpu::eor, Cpu::bad, Cpu::sre, Cpu::nop, Cpu::eor, Cpu::lsr, Cpu::sre, Cpu::pha, Cpu::eor, Cpu::lsr, Cpu::nop, Cpu::jmp, Cpu::eor, Cpu::lsr, Cpu::sre,  /*40*/
        /*50*/  Cpu::bvc, Cpu::eor, Cpu::bad, Cpu::sre, Cpu::nop, Cpu::eor, Cpu::lsr, Cpu::sre, Cpu::cli, Cpu::eor, Cpu::nop, Cpu::sre, Cpu::nop, Cpu::eor, Cpu::lsr, Cpu::sre,  /*50*/
        /*60*/  Cpu::rts, Cpu::adc, Cpu::bad, Cpu::rra, Cpu::nop, Cpu::adc, Cpu::ror, Cpu::rra, Cpu::pla, Cpu::adc, Cpu::ror, Cpu::nop, Cpu::jmp, Cpu::adc, Cpu::ror, Cpu::rra,  /*60*/
        /*70*/  Cpu::bvs, Cpu::adc, Cpu::bad, Cpu::rra, Cpu::nop, Cpu::adc, Cpu::ror, Cpu::rra, Cpu::sei, Cpu::adc, Cpu::nop, Cpu::rra, Cpu::nop, Cpu::adc, Cpu::ror, Cpu::rra,  /*70*/
        /*80*/  Cpu::nop, Cpu::sta, Cpu::nop, Cpu::sax, Cpu::sty, Cpu::sta, Cpu::stx, Cpu::sax, Cpu::dey, Cpu::nop, Cpu::txa, Cpu::nop, Cpu::sty, Cpu::sta, Cpu::stx, Cpu::sax,  /*80*/
        /*90*/  Cpu::bcc, Cpu::sta, Cpu::bad, Cpu::nop, Cpu::sty, Cpu::sta, Cpu::stx, Cpu::sax, Cpu::tya, Cpu::sta, Cpu::txs, Cpu::nop, Cpu::nop, Cpu::sta, Cpu::nop, Cpu::nop,  /*90*/
        /*A0*/  Cpu::ldy, Cpu::lda, Cpu::ldx, Cpu::lax, Cpu::ldy, Cpu::lda, Cpu::ldx, Cpu::lax, Cpu::tay, Cpu::lda, Cpu::tax, Cpu::nop, Cpu::ldy, Cpu::lda, Cpu::ldx, Cpu::lax,  /*A0*/
        /*B0*/  Cpu::bcs, Cpu::lda, Cpu::bad, Cpu::lax, Cpu::ldy, Cpu::lda, Cpu::ldx, Cpu::lax, Cpu::clv, Cpu::lda, Cpu::tsx, Cpu::nop, Cpu::ldy, Cpu::lda, Cpu::ldx, Cpu::lax,  /*B0*/
        /*C0*/  Cpu::cpy, Cpu::cmp, Cpu::nop, Cpu::dcp, Cpu::cpy, Cpu::cmp, Cpu::dec, Cpu::dcp, Cpu::iny, Cpu::cmp, Cpu::dex, Cpu::nop, Cpu::cpy, Cpu::cmp, Cpu::dec, Cpu::dcp,  /*C0*/
        /*D0*/  Cpu::bne, Cpu::cmp, Cpu::bad, Cpu::dcp, Cpu::nop, Cpu::cmp, Cpu::dec, Cpu::dcp, Cpu::cld, Cpu::cmp, Cpu::nop, Cpu::dcp, Cpu::nop, Cpu::cmp, Cpu::dec, Cpu::dcp,  /*D0*/
        /*E0*/  Cpu::cpx, Cpu::sbc, Cpu::nop, Cpu::isc, Cpu::cpx, Cpu::sbc, Cpu::inc, Cpu::isc, Cpu::inx, Cpu::sbc, Cpu::nop, Cpu::sbc, Cpu::cpx, Cpu::sbc, Cpu::inc, Cpu::isc,  /*E0*/
        /*F0*/  Cpu::beq, Cpu::sbc, Cpu::bad, Cpu::isc, Cpu::nop, Cpu::sbc, Cpu::inc, Cpu::isc, Cpu::sed, Cpu::sbc, Cpu::nop, Cpu::isc, Cpu::nop, Cpu::sbc, Cpu::inc, Cpu::isc,  /*F0*/
            ],
            mode_table: vec![
        //          00         01         02         03         04         05         06         07         08         09         0A         0B         0C         0D         0E         0F
        /*00*/  Mode::IMP, Mode::IDX, Mode::IMP, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::IMP, Mode::IMM, Mode::ACC, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,  /*00*/
        /*10*/  Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,  /*10*/
        /*20*/  Mode::ABS, Mode::IDX, Mode::IMP, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::IMP, Mode::IMM, Mode::ACC, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,  /*20*/
        /*30*/  Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,  /*30*/
        /*40*/  Mode::IMP, Mode::IDX, Mode::IMP, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::IMP, Mode::IMM, Mode::ACC, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,  /*40*/
        /*50*/  Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,  /*50*/
        /*60*/  Mode::IMP, Mode::IDX, Mode::IMP, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::IMP, Mode::IMM, Mode::ACC, Mode::IMM, Mode::IND, Mode::ABS, Mode::ABS, Mode::ABS,  /*60*/
        /*70*/  Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,  /*70*/
        /*80*/  Mode::IMM, Mode::IDX, Mode::IMM, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::IMP, Mode::IMM, Mode::IMP, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,  /*80*/
        /*90*/  Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPY, Mode::ZPY, Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABY, Mode::ABY,  /*90*/
        /*A0*/  Mode::IMM, Mode::IDX, Mode::IMM, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::IMP, Mode::IMM, Mode::IMP, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,  /*A0*/
        /*B0*/  Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPY, Mode::ZPY, Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABY, Mode::ABY,  /*B0*/
        /*C0*/  Mode::IMM, Mode::IDX, Mode::IMM, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::IMP, Mode::IMM, Mode::IMP, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,  /*C0*/
        /*D0*/  Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,  /*D0*/
        /*E0*/  Mode::IMM, Mode::IDX, Mode::IMM, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::IMP, Mode::IMM, Mode::IMP, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,  /*E0*/
        /*F0*/  Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,  /*F0*/
            ],
        };
        cpu.PC = ((cpu.read(RESET_VECTOR + 1) as usize) << 8) + cpu.read(RESET_VECTOR) as usize;
        cpu
    }

    pub fn step(&mut self) -> u64 {
        
        // skip cycles from OAM DMA if necessary
        if self.delay > 0 {
            self.delay -= 1;
            return 1;
        }

        // handle interrupts
        if self.ppu.trigger_nmi {
            self.nmi();
        }
        self.ppu.trigger_nmi = false;
        if self.apu.trigger_irq && (self.P & INTERRUPT_DISABLE_FLAG == 0) {
            self.irq();
        }
        self.apu.trigger_irq = false;

        // back up clock so we know how many cycles we complete
        let clock = self.clock;
        let opcode = <usize>::from(self.read(self.PC));

        // get addressing mode
        let mode = self.mode_table[opcode].clone();
        let address = mode.get()(self);

        // debugging
        let pc = self.PC;
        if address == 0x06d6 { // if we're doing something with the WarpZoneControl global
            println!("===========================\n0x{:04x} {:?}", address, mode);
            if self.more == 0 {
                self.more += 10;
            }
        }
        if self.more > 0 {
            print!("{:04X} {:02X} [{:02x} {:02x}] A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                pc, self.read(pc), self.read(pc + 1), self.read(pc+2),
                self.A, self.X, self.Y, self.P, self.S,
            );
            let mut zpg = Vec::<u8>::new();
            for i in 0..32 {
                zpg.push(self.read(i));
            }
            print!(" zpg: {:x?}", zpg);
            print!("\n");
            self.more -= 1;
        }

        // advance program counter according to how many bytes that instruction operated on
        self.advance_pc(mode);
        // look up instruction in table and execute
        self.opcode_table[opcode](self, address, mode);

        // return how many cycles it took
        self.clock - clock
    }

    // memory interface
    fn read(&mut self, address: usize) -> u8 {
        let val = match address {
            0x0000..=0x1FFF => self.mem[address % 0x0800],
            0x2000..=0x3FFF => self.read_ppu_reg(address % 8),
            0x4014          => self.read_ppu_reg(8),
            0x4015          => self.apu.read_status(),
            0x4016          => self.read_controller(),
            0x4000..=0x4017 => 0, // can't read from these APU registers
            0x4018..=0x401F => 0, // APU and I/O functionality that is normally disabled. See CPU Test Mode.
            0x4020..=0xFFFF => {  // Cartridge space: PRG ROM, PRG RAM, and mapper registers
                *(self.mapper_func)(self, address, false).unwrap() // unwrapping because mapper funcs won't return None for reads.
            },
            _ => panic!("invalid read from 0x{:02x}", address),
        };
        val
    }

    // memory interface
    fn write(&mut self, address: usize, val: u8) {
        match address {
            0x0000..=0x1FFF => self.mem[address % 0x0800] = val,
            0x2000..=0x3FFF => self.write_ppu_reg(address % 8, val),
            0x4014          => self.write_ppu_reg(8, val),
            0x4016          => self.write_controller(val),
            0x4000..=0x4017 => self.apu.write_reg(address, val), // APU stuff
            0x4018..=0x401F => (), // APU and I/O functionality that is normally disabled. See CPU Test Mode.
            0x4020..=0xFFFF => {   // Cartridge space: PRG ROM, PRG RAM, and mapper registers
                match (self.mapper_func)(self, address, true) {
                    Some(loc) => *loc = val,
                    None => (),
                };
            },
            _ => panic!("invalid write to {:02x}", address),
        }
    }

    fn read_controller(&mut self) -> u8 {
        let bit = match self.button_number < 8 {
            true => (self.button_states & (1<<self.button_number) != 0) as u8,
            false => 1,
        };
        if self.strobe & 1 != 0 {
            self.button_number = 0;
        } else {
            self.button_number += 1;
        }
        bit
    }

    fn write_controller(&mut self, val: u8) {
        self.strobe = val;
        if self.strobe & 1 != 0 {
            self.button_number = 0;
        }
    }

    fn read_ppu_reg(&mut self, reg_num: usize) -> u8 {
        match reg_num {
            2 => self.ppu.read_status(),
            4 => self.ppu.read_oam_data(),
            7 => self.ppu.read_data(),
            _ => 0,
        }
    }

    fn write_ppu_reg(&mut self, reg_num: usize, val: u8) {
        self.ppu.recent_bits = val;
        match reg_num {
            0 => self.ppu.write_controller(val),
            1 => self.ppu.write_mask(val),
            3 => self.ppu.write_oam_address(val as usize),
            4 => self.ppu.write_oam_data(val),
            5 => self.ppu.write_scroll(val),
            6 => self.ppu.write_address(val),
            7 => self.ppu.write_data(val),
            8 => {
                let page = (val as usize) << 8;
                let mut data = vec![];
                for i in 0..=255 {
                    data.push(self.read(page + i));
                }
                self.ppu.write_oam_dma(data);
                let is_odd = self.clock % 2 != 0;
                self.delay = 513 + if is_odd {1} else {0};
            },
            _ => panic!("wrote to bad ppu reg: {}", reg_num),
        }
    }

}


/*
Address range 	Size 	Device
$0000-$07FF 	$0800 	2KB internal RAM
$0800-$0FFF 	$0800 	]---+
$1000-$17FF 	$0800       |---- Mirrors of $0000-$07FF
$1800-$1FFF 	$0800   ]---+
$2000-$2007 	$0008 	NES PPU registers
$2008-$3FFF 	$1FF8 	Mirrors of $2000-2007 (repeats every 8 bytes)
$4000-$4017 	$0018 	NES APU and I/O registers
$4018-$401F 	$0008 	APU and I/O functionality that is normally disabled. See CPU Test Mode.
$4020-$FFFF 	$BFE0 	Cartridge space: PRG ROM, PRG RAM, and mapper registers (See Note)
*/
