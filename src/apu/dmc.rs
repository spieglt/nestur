impl super::DMC {
    pub fn new() -> Self {
        super::DMC {
            sample: 0,
            enabled: false,
            bytes_remaining: 0,
            interrupt: false,
            length_counter: 0,
        }
    }

    
    pub fn control(&mut self, value: u8) {
        
    }
    
    pub fn direct_load(&mut self, value: u8) {
        
    }
    
    pub fn sample_address(&mut self, value: u8) {
        
    }
    
    pub fn sample_length(&mut self, value: u8) {
        
    }
}