impl super::Square {
    pub fn new() -> Self {
        super::Square {
            duty_cycle: 0,
            length_counter_halt: false,
            constant_volume_flag: false,
            timer: 0,
            length_counter: 0,
            envelope: 0,
            sweep: 0,
            sample: 0,
        }
    }

    pub fn clock(&mut self) {

    }

    pub fn duty(&mut self, value: u8) {

    }

    pub fn sweep(&mut self, value: u8) {
        
    }

    pub fn timer_low(&mut self, value: u8) {

    }

    pub fn timer_high(&mut self, value: u8) {
        
    }
}