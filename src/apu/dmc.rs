
pub const SAMPLE_RATES: [u16; 16] = [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54];

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DMC {
    pub sample: u16,
    pub enabled: bool,
    pub interrupt: bool,
    pub loop_flag: bool,
    pub rate_index: usize,
    pub length_counter: usize,
    pub cpu_cycles_left: u16,

    // Memory reader
    sample_byte: u8, // passed in every APU clock cycle, need to think of a better way to read CPU from APU
    sample_buffer: Option<u8>, // buffer that the output unit draws into its shift register
    sample_address: usize, // start of sample in memory
    sample_length: usize,
    pub current_address: usize, // address of the next byte of the sample to play
    pub bytes_remaining: usize, // bytes left in the sample

    // Output unit

}

impl DMC {
    pub fn new() -> Self {
        DMC {
            sample: 0,
            enabled: false,
            interrupt: false,
            loop_flag: false,
            rate_index: 0,
            length_counter: 0,
            cpu_cycles_left: 0,
            sample_byte: 0,
            sample_buffer: None,
            sample_address: 0,
            sample_length: 0,
            current_address: 0,
            bytes_remaining: 0,
        }
    }

    pub fn clock(&mut self, sample_byte: u8) {
        // self.sample_byte = sample_byte;
        self.clock_memory_reader(sample_byte);
    }

    fn clock_memory_reader(&mut self, sample_byte: u8) {
        // When the sample buffer is emptied, the memory reader fills the sample buffer
        // with the next byte from the currently playing sample. It has an address counter and a bytes remaining counter.
        // if self.sample_buffer.is_none() {
        //     self.sample_buffer = Some(sample_byte);
        // }
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
            // Specific delay cases:
            //     4 cycles if it falls on a CPU read cycle.
            //     3 cycles if it falls on a single CPU write cycle (or the second write of a double CPU write).
            //     4 cycles if it falls on the first write of a double CPU write cycle.[4]
            //     2 cycles if it occurs during an OAM DMA, or on the $4014 write cycle that triggers the OAM DMA.
            //     1 cycle if it occurs on the second-last OAM DMA cycle.
            //     3 cycles if it occurs on the last OAM DMA cycle.

            // The sample buffer is filled with the next sample byte read from the current address, subject to whatever mapping hardware is present.
            self.sample_buffer = Some(sample_byte);
            // The address is incremented; if it exceeds $FFFF, it is wrapped around to $8000.
            if self.current_address == 0xFFFF {
                self.current_address = 0x8000
            } else {
                self.current_address += 1;
            }
            // The bytes remaining counter is decremented; if it becomes zero and the loop flag is set, the sample is restarted (see above); otherwise, if the bytes remaining counter becomes zero and the IRQ enabled flag is set, the interrupt flag is set.
            self.bytes_remaining -= 1;
        // At any time, if the interrupt flag is set, the CPU's IRQ line is continuously asserted until the interrupt flag is cleared.
        // The processor will continue on from where it was stalled.
    }
   
    pub fn write_control(&mut self, value: u8) {
        // $4010 	IL--.RRRR 	Flags and Rate (write)
        
    }
   
    pub fn direct_load(&mut self, value: u8) {
       
    }
   
    pub fn write_sample_address(&mut self, value: u8) {
       
    }
   
    pub fn write_sample_length(&mut self, value: u8) {
       
    }
}