pub struct DMC {
    pub sample: u16,
    pub enabled: bool,
    pub interrupt: bool,
    pub length_counter: usize,
    pub bytes_remaining: usize,
}

impl DMC {
    pub fn new() -> Self {
        DMC {
            sample: 0,
            enabled: false,
            interrupt: false,
            bytes_remaining: 0,
            length_counter: 0,
        }
    }

    pub fn clock(&mut self) {
       
    }
   
    pub fn write_control(&mut self, value: u8) {
       
    }
   
    pub fn direct_load(&mut self, value: u8) {
       
    }
   
    pub fn write_sample_address(&mut self, value: u8) {
       
    }
   
    pub fn write_sample_length(&mut self, value: u8) {
       
    }
}