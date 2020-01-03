
const NOISE_TABLE: [u16; 16] = [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

// $400E 	M---.PPPP 	Mode and period (write)
// bit 7 	M--- ---- 	Mode flag 
pub struct Noise {
    pub sample: u16,
    pub enabled: bool,

    envelope: u16, // constant volume/envelope period. reflects what was last written to $4000/$4004
    envelope_divider: u16,
    decay_counter: u16, // remainder of envelope divider
    constant_volume_flag: bool, // (0: use volume from envelope; 1: use constant volume)
    start: bool, // restarts envelope

    timer: u16,
    timer_period: u16,

    pub length_counter: u8,
    length_counter_halt: bool,

    linear_feedback_sr: u16,
    mode: bool, // also called loop noise, bit 7 of $400E
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            sample: 0,
            enabled: false,
            envelope: 0,
            envelope_divider: 0,
            decay_counter: 0,
            constant_volume_flag: false,
            start: false,
            timer: 0,
            timer_period: 0,
            length_counter: 0,
            length_counter_halt: false,
            linear_feedback_sr: 1, // On power-up, the shift register is loaded with the value 1. 
            mode: false, // also called loop noise, bit 7 of $400E
        }
    }

    pub fn clock(&mut self) {
        if self.timer == 0 {
            self.clock_linear_counter();
        } else {
            self.timer -= 1;
        }
        // The mixer receives the current envelope volume except when 
        // Bit 0 of the shift register is set, or the length counter is zero
        self.sample = if self.linear_feedback_sr & 1 == 1 || self.length_counter == 0 {
            0
        } else if self.constant_volume_flag {
            self.envelope
        } else {
            self.decay_counter
        };
    }

    pub fn clock_linear_counter(&mut self) {
        // When the timer clocks the shift register, the following actions occur in order:
        // Feedback is calculated as the exclusive-OR of bit 0
        let bit0 = self.linear_feedback_sr & (1 << 0);
        // and one other bit: bit 6 if Mode flag is set, otherwise bit 1.
        let bit_num = if self.mode { 6 } else { 1 };
        let other_bit = (self.linear_feedback_sr & (1 << bit_num)) >> bit_num;
        let feedback = bit0 ^ other_bit;
        // The shift register is shifted right by one bit.
        self.linear_feedback_sr >>= 1;
        // Bit 14, the leftmost bit, is set to the feedback calculated earlier.
        self.linear_feedback_sr |= feedback << 14;
    }

    pub fn clock_envelope(&mut self) {
        // When clocked by the frame counter, one of two actions occurs:
        // if the start flag is clear, the divider is clocked,
        if !self.start {
            self.clock_envelope_divider();
        } else {
            self.start = false; // otherwise the start flag is cleared,
            self.decay_counter = 15; // the decay level counter is loaded with 15,
            self.envelope_divider = self.envelope; // and the divider's period is immediately reloaded
        }
    }

    fn clock_envelope_divider(&mut self) {
        // When the divider is clocked while at 0, it is loaded with V and clocks the decay level counter.
        if self.envelope_divider == 0 {
            self.envelope_divider = self.envelope;
            // Then one of two actions occurs: If the counter is non-zero, it is decremented,
            if self.decay_counter != 0 {
                self.decay_counter -= 1;
            } else if self.length_counter_halt {
                // otherwise if the loop flag is set, the decay level counter is loaded with 15.
                self.decay_counter = 15;
            }
        } else {
            self.envelope_divider -= 1;
        }
    }

    pub fn clock_length_counter(&mut self) {
        if !(self.length_counter == 0 || self.length_counter_halt) {
            self.length_counter -= 1;
        }
    }

    // $400C
    pub fn write_envelope(&mut self, value: u8) {
        self.length_counter_halt = (value >> 5) & 1 == 1;
        self.constant_volume_flag = (value >> 4) & 1 == 1;
        self.envelope = value as u16 & 0b1111;
    }

    // $400E
    pub fn write_loop_noise(&mut self, value: u8) {
        self.mode = value >> 7 == 1;
        self.timer_period = NOISE_TABLE[(value & 0b1111) as usize];
    }

    // $400F
    pub fn write_length_counter(&mut self, value: u8) {
        self.length_counter = value >> 3;
        self.start = true;
    }
}

// When the timer clocks the shift register, the following actions occur in order:

//    1. Feedback is calculated as the exclusive-OR of bit 0 and one other bit: bit 6 if Mode flag is set, otherwise bit 1.
//    2. The shift register is shifted right by one bit.
//    3. Bit 14, the leftmost bit, is set to the feedback calculated earlier.

// This results in a pseudo-random bit sequence, 32767 steps long when Mode flag is clear,
// and randomly 93 or 31 steps long otherwise. (The particular 31- or 93-step sequence depends
// on where in the 32767-step sequence the shift register was when Mode flag was set).
