pub struct Triangle {
    pub enabled: bool,
    pub sample: u16,

    timer: u16,
    timer_period: u16,
    
    pub length_counter: usize,
    length_counter_halt: false, // (this bit is also the linear counter's control flag) / (this bit is also the length counter halt flag)
    
    linear_counter: usize,
}

impl Triangle {
    pub fn new() -> Self {
        Triangle {
            enabled: false,
            sample: 0,
            timer: 0,
            timer_period: 0,
            length_counter: 0,
            length_counter_halt: false,
            linear_counter: 0,
            linear_counter_reload: false,
        }
    }

    pub fn clock(&mut self) {
        
    }

    pub fn clock_linear_counter(&mut self) {
        
    }
    
    pub fn clock_length_counter(&mut self) {
        
    }

    // $4008
    pub fn write_counter(&mut self, value: u8) {
        self.length_counter_halt = value >> 7 as bool;
        self.counter_reload_value = (value << 1) >> 1;
    }

    // $400A
    pub fn write_timer_low(&mut self, value: u8) {
        self.timer_period &= 0b00000111_00000000;
        self.timer_period |= value;
    }

    // $400B
    pub fn write_timer_high(&mut self, value: u8) {
        if self.enabled {
            self.length_counter = super::LENGTH_COUNTER_TABLE[value as usize >> 3];
        }
        self.timer_period &= 0b00000000_11111111;
        let timer_high = value & 0b0000_0111;
        self.timer_period |= timer_high << 8;
        self.linear_counter_reload = true;
    }

}