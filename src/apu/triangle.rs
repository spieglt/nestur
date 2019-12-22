
// $4008 	Hlll.llll 	Triangle channel length counter halt and linear counter load (write)
// bit 7 	H--- ---- 	Halt length counter (this bit is also the linear counter's control flag) 
pub struct Triangle {
    pub sample: u16,
    timer: usize,
    pub length_counter: usize, // (this bit is also the linear counter's control flag) 
    linear_counter: usize,
    pub enabled: bool,
}

impl Triangle {
    pub fn new() -> Self {
        Triangle {
            timer: 0,
            length_counter: 0,
            linear_counter: 0,
            sample: 0,
            enabled: false,
        }
    }

    pub fn write_timer_low(&mut self, value: u8) {

    }

    pub fn write_timer_high(&mut self, value: u8) {
        
    }

    pub fn write_counter(&mut self, value: u8) {
        
    }

    pub fn clock_linear_counter(&mut self) {
        
    }
    
    pub fn clock_length_counter(&mut self) {
        
    }
}