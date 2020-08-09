// number of CPU cycles between sample output level being adjusted
pub const SAMPLE_RATES: [u16; 16] = [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54];

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DMC {
    pub sample: u16, // "output value" that goes to the mixer
    pub enabled: bool, // TODO: what does this do for the DMC channel?
    irq_enabled: bool,
    pub interrupt: bool,
    loop_flag: bool,
    pub cpu_stall: bool,
    rate_index: usize,
    cpu_cycles_left: u16,

    // Memory reader
    sample_byte: u8, // passed in every APU clock cycle, need to think of a better way to read CPU from APU
    sample_buffer: Option<u8>, // buffer that the output unit draws into its shift register, wrapped in Option to denote 'emptiness'
    pub sample_address: usize, // start of sample in memory
    pub sample_length: usize, // number of bytes starting from sample_address that constitute the sample. each byte has 8 bits that can raise or lower the output level, at a speed determined by rate_index
    pub current_address: usize, // address of the next byte of the sample to play
    pub bytes_remaining: usize, // bytes left in the sample

    // Output unit
    shift_register: u8,
    bits_remaining: usize,
    silence: bool,
}

impl DMC {
    pub fn new() -> Self {
        DMC {
            sample: 0,
            enabled: false,
            irq_enabled: false,
            interrupt: false,
            loop_flag: false,
            cpu_stall: false,
            rate_index: 0,
            cpu_cycles_left: 0,
            sample_byte: 0,
            sample_buffer: None,
            sample_address: 0,
            sample_length: 0,
            current_address: 0,
            bytes_remaining: 0,
            shift_register: 0,
            bits_remaining: 0,
            silence: false,
        }
    }

    pub fn clock(&mut self, sample_byte: u8) {
        self.clock_memory_reader(sample_byte);
        self.clock_output_unit();
    }

    fn clock_memory_reader(&mut self, sample_byte: u8) {
        // When a sample is (re)started, the current address is set to the sample address, and bytes remaining is set to the sample length.
        if self.bytes_remaining == 0 && self.loop_flag {
            self.current_address = self.sample_address;
            self.bytes_remaining = self.sample_length;
        }
        // Any time the sample buffer is in an empty state and bytes remaining is not zero (including just after a write to $4015 that enables the channel,
        // regardless of where that write occurs relative to the bit counter mentioned below), the following occur:
        if self.sample_buffer.is_none() && self.bytes_remaining != 0 {
            // The CPU is stalled for up to 4 CPU cycles to allow the longest possible write (the return address and write after an IRQ) to finish.
            // If OAM DMA is in progress, it is paused for two cycles. The sample fetch always occurs on an even CPU cycle due to its alignment with the APU.
            self.cpu_stall = true;
            // The sample buffer is filled with the next sample byte read from the current address, subject to whatever mapping hardware is present.
            self.sample_buffer = Some(sample_byte);
            // The address is incremented; if it exceeds $FFFF, it is wrapped around to $8000.
            if self.current_address == 0xFFFF {
                self.current_address = 0x8000
            } else {
                self.current_address += 1;
            }
            // The bytes remaining counter is decremented; if it becomes zero and the loop flag is set, the sample is restarted (see above);
            // otherwise, if the bytes remaining counter becomes zero and the IRQ enabled flag is set, the interrupt flag is set.
            self.bytes_remaining -= 1;
        } else if self.sample_buffer.is_none() && self.irq_enabled {
            self.interrupt = true;
        }
    }

    fn clock_output_unit(&mut self) {
        // When the timer outputs a clock, the following actions occur in order:
        // If the silence flag is clear, the output level changes based on bit 0 of the shift register.
        // If the bit is 1, add 2; otherwise, subtract 2. But if adding or subtracting 2 would cause the output level to leave the 0-127 range,
        // leave the output level unchanged. This means subtract 2 only if the current level is at least 2, or add 2 only if the current level is at most 125.
        // The right shift register is clocked.
        // As stated above, the bits-remaining counter is decremented. If it becomes zero, a new output cycle is started.
        self.cpu_cycles_left -= 2;
        if self.cpu_cycles_left == 0 {
            self.cpu_cycles_left = SAMPLE_RATES[self.rate_index];
            if self.silence {
                self.sample = 0;
            } else {
                match self.shift_register & 1 {
                    0 => if self.sample >= 2 { self.sample -= 2},
                    1 => if self.sample <= 125 { self.sample += 2 },
                    _ => panic!("uh oh! magical bits!"),
                }
            }
            self.shift_register >>= 1;
            self.bits_remaining -= 1;
            // When an output cycle ends, a new cycle is started as follows:
            // The bits-remaining counter is loaded with 8.
            // If the sample buffer is empty, then the silence flag is set; otherwise, the silence flag is cleared and the sample buffer is emptied into the shift register.
            if self.bits_remaining == 0 {
                self.bits_remaining = 8;
                match self.sample_buffer {
                    Some(s) => {
                        self.silence = false;
                        self.shift_register = s;
                        self.sample_buffer = None;
                    },
                    None => self.silence = true,
                }
            }
        }
    }

    pub fn write_control(&mut self, value: u8) {
        // $4010 	IL--.RRRR 	Flags and Rate (write)
        self.irq_enabled = value & 0b1000_0000 != 0;
        if !self.irq_enabled {
            self.interrupt = false;
        }
        self.loop_flag = value & 0b0100_0000 != 0;
        self.rate_index = value as usize & 0b0000_1111;
    }
   
    pub fn direct_load(&mut self, value: u8) {
        // $4011 	-DDD.DDDD 	Direct load (write)
        self.sample = value as u16 & 0b0111_1111;
    }
   
    pub fn write_sample_address(&mut self, value: u8) {
        // $4012 	AAAA.AAAA 	Sample address (write)
        // bits 7-0 	AAAA.AAAA 	Sample address = %11AAAAAA.AA000000 = $C000 + (A * 64)
        self.sample_address = ((value as usize) << 6) + 0xC000;
    }
   
    pub fn write_sample_length(&mut self, value: u8) {
        // $4013 	LLLL.LLLL 	Sample length (write)
        // bits 7-0 	LLLL.LLLL 	Sample length = %LLLL.LLLL0001 = (L * 16) + 1 bytes
        self.sample_length = ((value as usize) << 4) + 1;
    }
}