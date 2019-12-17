
// $400E 	M---.PPPP 	Mode and period (write)
// bit 7 	M--- ---- 	Mode flag 
pub struct Noise {
    pub sample: u16,
    timer: usize,
    pub length_counter: usize,
    envelope: usize,
    linear_feedback_sr: u16,
    mode: bool, // also called loop noise, bit 7 of $400E
    pub enabled: bool,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            timer: 0,
            length_counter: 0,
            envelope: 0,
            linear_feedback_sr: 1, // On power-up, the shift register is loaded with the value 1. 
            mode: false, // also called loop noise, bit 7 of $400E
            sample: 0,
            enabled: false,
        }
    }

    pub fn clock(&mut self) {
        let bit0 = self.linear_feedback_sr & (1 << 0);
        let other_bit = match self.mode {
            false => (self.linear_feedback_sr & (1 << 1)) >> 1,
            true  => (self.linear_feedback_sr & (1 << 6)) >> 6,
        };
        let feedback = bit0 ^ other_bit;
        self.linear_feedback_sr >>= 1;
        self.linear_feedback_sr |= feedback << 14;
    }

    pub fn envelope(&mut self, value: u8) {
        
    }

    pub fn loop_noise(&mut self, value: u8) {
        
    }
    pub fn load_length_counter(&mut self, value: u8) {
        
    }
}

// When the timer clocks the shift register, the following actions occur in order:

//    1. Feedback is calculated as the exclusive-OR of bit 0 and one other bit: bit 6 if Mode flag is set, otherwise bit 1.
//    2. The shift register is shifted right by one bit.
//    3. Bit 14, the leftmost bit, is set to the feedback calculated earlier.

// This results in a pseudo-random bit sequence, 32767 steps long when Mode flag is clear,
// and randomly 93 or 31 steps long otherwise. (The particular 31- or 93-step sequence depends
// on where in the 32767-step sequence the shift register was when Mode flag was set).
