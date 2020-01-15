const WAVEFORM: [u16; 32] = [
    15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
];

pub struct Triangle {
    pub sample: u16,
    pub enabled: bool,

    timer: u16,
    timer_period: u16,
    waveform_counter: usize,
    pub length_counter: u8,
    length_counter_halt: bool, // (this bit is also the linear counter's control flag)
    
    linear_counter: u8,
    counter_reload_value: u8,
    linear_counter_reload: bool,
}

impl Triangle {
    pub fn new() -> Self {
        Triangle {
            sample: 0,
            enabled: false,
            timer: 0,
            timer_period: 0,
            waveform_counter: 0,
            length_counter: 0,
            length_counter_halt: false,
            linear_counter: 0,
            counter_reload_value: 0,
            linear_counter_reload: false,
        }
    }

    pub fn clock(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            // The sequencer is clocked by the timer as long as both the linear counter and the length counter are nonzero. 
            if self.linear_counter != 0 && self.length_counter != 0 {
                self.waveform_counter = (self.waveform_counter + 1) % 32;   
            }
        } else {
            self.timer -= 1;
        }
        self.sample = WAVEFORM[self.waveform_counter];
    }

    pub fn clock_linear_counter(&mut self) {
        // When the frame counter generates a linear counter clock, the following actions occur in order:
        // If the linear counter reload flag is set, the linear counter is reloaded with the counter reload value,
        if self.linear_counter_reload {
            self.linear_counter = self.counter_reload_value;
        } else if self.linear_counter != 0 { // otherwise if the linear counter is non-zero, it is decremented.
            self.linear_counter -= 1;
        }
        // If the control flag is clear, the linear counter reload flag is cleared.
        if !self.length_counter_halt {
            self.linear_counter_reload = false;
        }
    }
    
    pub fn clock_length_counter(&mut self) {
        if !(self.length_counter == 0 || self.length_counter_halt) {
            self.length_counter -= 1;
        }
    }

    // $4008
    pub fn write_counter(&mut self, value: u8) {
        self.length_counter_halt = value >> 7 != 0;
        self.counter_reload_value = (value << 1) >> 1;
    }

    // $400A
    pub fn write_timer_low(&mut self, value: u8) {
        self.timer_period &= 0b00000111_00000000;
        self.timer_period |= value as u16;
    }

    // $400B
    pub fn write_timer_high(&mut self, value: u8) {
        if self.enabled {
            self.length_counter = super::LENGTH_COUNTER_TABLE[value as usize >> 3];
        }
        self.timer_period &= 0b00000000_11111111;
        let timer_high = value & 0b0000_0111;
        self.timer_period |= (timer_high as u16) << 8;
        self.linear_counter_reload = true;
    }

}
