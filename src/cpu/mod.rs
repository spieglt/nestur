mod addressing_modes;
mod opcodes;
mod utility;
pub mod serialize;

use std::cell::RefCell;
use std::rc::Rc;
use serde::{Serialize, Deserialize};
use crate::cartridge::Mapper;

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Mode {
    ABS, ABX, ABY, ACC,
    IMM, IMP, IDX, IND,
    INX, REL, ZPG, ZPX,
    ZPY,
}

type AddressingFunction = fn(&mut Cpu) -> usize;

impl Mode {
    fn get(&self) -> (AddressingFunction, usize) { // usize is number of bytes the instruction takes, used for debug printing
        match self {
            Mode::ABS => (Cpu::absolute, 3),
            Mode::ABX => (Cpu::absolute_x, 3),
            Mode::ABY => (Cpu::absolute_y, 3),
            Mode::ACC => (Cpu::accumulator, 1),
            Mode::IMM => (Cpu::immediate, 2),
            Mode::IMP => (Cpu::implied, 1),
            Mode::IDX => (Cpu::indexed_indirect, 2),
            Mode::IND => (Cpu::indirect, 3),
            Mode::INX => (Cpu::indirect_indexed, 2),
            Mode::REL => (Cpu::relative, 2),
            Mode::ZPG => (Cpu::zero_page, 2),
            Mode::ZPX => (Cpu::zero_page_x, 2),
            Mode::ZPY => (Cpu::zero_page_y, 2),
        }
    }
}

pub struct Cpu {
    mem: Vec<u8>, // CPU's RAM, $0000-$1FFF
    a: u8,        // accumulator
    x: u8,        // general purpose
    y: u8,        // general purpose
    pc: usize,    // 16-bit program counter
    s: u8,        // stack pointer
    p: u8,        // status

    clock: u64, // number of ticks in current cycle
    delay: usize, // for skipping cycles during OAM DMA

    pub mapper: Rc<RefCell<dyn Mapper>>, // cartridge data
    pub ppu: super::Ppu,
    pub apu: super::Apu,

    // controller
    pub strobe: u8, // signals to the controller that button inputs should be read
    pub button_states: u8, // Player 1 controller
    button_number: u8, // counter that scans the bits of the input register serially

    opcode_table: Vec<fn(&mut Self, usize, Mode)>, // function table
    mode_table: Vec<Mode>, // address mode table
}

impl Cpu {
    pub fn new(mapper: Rc<RefCell<dyn Mapper>>, ppu: super::Ppu, apu: super::Apu) -> Self {
        let mut cpu = Cpu{
            mem: vec![0; 0x2000],
            a: 0, x: 0, y: 0,
            pc: 0,
            s: 0xFD,
            p: 0x24,
            clock: 0,
            delay: 0,
            mapper: mapper,
            ppu: ppu,
            apu: apu,
            strobe: 0,
            button_states: 0,
            button_number: 0,
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
        cpu.pc = ((cpu.read(RESET_VECTOR + 1) as usize) << 8) + cpu.read(RESET_VECTOR) as usize;
        cpu
    }

    pub fn step(&mut self) -> u64 {

        // The CPU is stalled for up to 4 CPU cycles to allow the longest possible write (the return address and write after an IRQ) to finish.
        // If OAM DMA is in progress, it is paused for two cycles. The sample fetch always occurs on an even CPU cycle due to its alignment with the APU.
        // Specific delay cases:
        //     4 cycles if it falls on a CPU read cycle.
        //     3 cycles if it falls on a single CPU write cycle (or the second write of a double CPU write).
        //     4 cycles if it falls on the first write of a double CPU write cycle.[4]
        //     2 cycles if it occurs during an OAM DMA, or on the $4014 write cycle that triggers the OAM DMA.
        //     1 cycle if it occurs on the second-last OAM DMA cycle.
        //     3 cycles if it occurs on the last OAM DMA cycle.
        if self.apu.dmc.cpu_stall {
            self.delay = 3; // TODO: not correct
            self.apu.dmc.cpu_stall = false;
        }
        
        // skip cycles from OAM DMA if necessary
        if self.delay > 0 {
            self.delay -= 1;
            return 1;
        }

        self.handle_interrupts();

        // back up clock so we know how many cycles we complete
        let clock = self.clock;
        let opcode = <usize>::from(self.read(self.pc));

        // get addressing mode
        let mode = self.mode_table[opcode].clone();
        let (address_func, _num_bytes) = mode.get();
        let address = address_func(self);
        // self._debug(_num_bytes, opcode);
        // advance program counter according to how many bytes that instruction operated on
        self.advance_pc(mode);
        // look up instruction in table and execute
        self.opcode_table[opcode](self, address, mode);

        // return how many cycles it took
        self.clock - clock
    }

    // memory interface
    pub fn read(&mut self, address: usize) -> u8 {
        let val = match address {
            0x0000..=0x1FFF => self.mem[address % 0x0800],
            0x2000..=0x3FFF => self.read_ppu_reg(address % 8),
            0x4014          => self.read_ppu_reg(8),
            0x4015          => self.apu.read_status(),
            0x4016          => self.read_controller(),
            0x4000..=0x4017 => 0, // can't read from these APU registers
            0x4018..=0x401F => 0, // APU and I/O functionality that is normally disabled. See CPU Test Mode.
            0x4020..=0xFFFF => self.mapper.borrow().read(address),
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
            0x4000..=0x4017 => self.apu.write_reg(address, val),
            0x4018..=0x401F => (), // APU and I/O functionality that is normally disabled. See CPU Test Mode.
            0x4020..=0xFFFF => self.mapper.borrow_mut().write(address, val),
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
        bit | 0x40
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
            _ => self.ppu.recent_bits,
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

    fn handle_interrupts(&mut self) {
        // handle interrupts from ppu
        if self.ppu.trigger_nmi {
            self.nmi();
        }
        self.ppu.trigger_nmi = false;
        // and apu
        if self.apu.trigger_irq && (self.p & INTERRUPT_DISABLE_FLAG == 0) {
            self.irq();
        }
        self.apu.trigger_irq = false;
        // and mapper MMC3
        if self.mapper.borrow_mut().check_irq() && (self.p & INTERRUPT_DISABLE_FLAG == 0) {
            self.irq();
        }
        // TODO: should checks for APU and MMC3 IRQs be combined and acknowledged together?
        
        // At any time, if the interrupt flag is set, the CPU's IRQ line is continuously asserted until the interrupt flag is cleared.
        // The processor will continue on from where it was stalled.
        if self.apu.dmc.interrupt {
            self.irq();
        }
    }

    fn _debug(&mut self, num_bytes: usize, opcode: usize) {
        let pc = self.pc;
        let operands = match num_bytes {
            1 => "     ".to_string(),
            2 => format!("{:02X}   ", self.read(pc + 1)),
            3 => format!("{:02X} {:02X}", self.read(pc + 1), self.read(pc+2)),
            _ => "error".to_string(),
        };
        println!("{:04X}  {:02X} {}  {}           A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            pc, self.read(pc), operands, _OPCODE_DISPLAY_NAMES[opcode],
            self.a, self.x, self.y, self.p, self.s,
        );
    }

    pub fn _memory_at(&mut self, address: usize, amount: usize) -> Vec<u8> {
        let mut ret = vec![];
        for i in 0..amount {
            ret.push(self.read(address+i));
        }
        ret
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

// For debug output
const _OPCODE_DISPLAY_NAMES: [&str; 256] = [
    "BRK", "ORA", "BAD", "SLO", "NOP", "ORA", "ASL", "SLO",
    "PHP", "ORA", "ASL", "ANC", "NOP", "ORA", "ASL", "SLO",
    "BPL", "ORA", "BAD", "SLO", "NOP", "ORA", "ASL", "SLO",
    "CLC", "ORA", "NOP", "SLO", "NOP", "ORA", "ASL", "SLO",
    "JSR", "AND", "BAD", "RLA", "BIT", "AND", "ROL", "RLA",
    "PLP", "AND", "ROL", "ANC", "BIT", "AND", "ROL", "RLA",
    "BMI", "AND", "BAD", "RLA", "NOP", "AND", "ROL", "RLA",
    "SEC", "AND", "NOP", "RLA", "NOP", "AND", "ROL", "RLA",
    "RTI", "EOR", "BAD", "SRE", "NOP", "EOR", "LSR", "SRE",
    "PHA", "EOR", "LSR", "ALR", "JMP", "EOR", "LSR", "SRE",
    "BVC", "EOR", "BAD", "SRE", "NOP", "EOR", "LSR", "SRE",
    "CLI", "EOR", "NOP", "SRE", "NOP", "EOR", "LSR", "SRE",
    "RTS", "ADC", "BAD", "RRA", "NOP", "ADC", "ROR", "RRA",
    "PLA", "ADC", "ROR", "ARR", "JMP", "ADC", "ROR", "RRA",
    "BVS", "ADC", "BAD", "RRA", "NOP", "ADC", "ROR", "RRA",
    "SEI", "ADC", "NOP", "RRA", "NOP", "ADC", "ROR", "RRA",
    "NOP", "STA", "NOP", "SAX", "STY", "STA", "STX", "SAX",
    "DEY", "NOP", "TXA", "XAA", "STY", "STA", "STX", "SAX",
    "BCC", "STA", "BAD", "AHX", "STY", "STA", "STX", "SAX",
    "TYA", "STA", "TXS", "TAS", "SHY", "STA", "SHX", "AHX",
    "LDY", "LDA", "LDX", "LAX", "LDY", "LDA", "LDX", "LAX",
    "TAY", "LDA", "TAX", "LAX", "LDY", "LDA", "LDX", "LAX",
    "BCS", "LDA", "BAD", "LAX", "LDY", "LDA", "LDX", "LAX",
    "CLV", "LDA", "TSX", "LAS", "LDY", "LDA", "LDX", "LAX",
    "CPY", "CMP", "NOP", "DCP", "CPY", "CMP", "DEC", "DCP",
    "INY", "CMP", "DEX", "AXS", "CPY", "CMP", "DEC", "DCP",
    "BNE", "CMP", "BAD", "DCP", "NOP", "CMP", "DEC", "DCP",
    "CLD", "CMP", "NOP", "DCP", "NOP", "CMP", "DEC", "DCP",
    "CPX", "SBC", "NOP", "ISC", "CPX", "SBC", "INC", "ISC",
    "INX", "SBC", "NOP", "SBC", "CPX", "SBC", "INC", "ISC",
    "BEQ", "SBC", "BAD", "ISC", "NOP", "SBC", "INC", "ISC",
    "SED", "SBC", "NOP", "ISC", "NOP", "SBC", "INC", "ISC",
];
