use super::envelope::Envelope;

const NOISE_TABLE: [u16; 16] = [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

// $400E 	M---.PPPP 	Mode and period (write)
// bit 7 	M--- ---- 	Mode flag
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Noise {
    pub sample: u16, // output value that gets sent to the mixer
    pub enabled: bool,
    constant_volume_flag: bool,
    mode: bool, // also called loop noise, bit 7 of $400E
    timer: u16,
    timer_period: u16,
    pub length_counter: u8,
    linear_feedback_sr: u16,
    pub envelope: Envelope,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            sample: 0,
            enabled: false,
            constant_volume_flag: false,
            mode: false,
            timer: 0,
            timer_period: 0,
            length_counter: 0,
            linear_feedback_sr: 1, // On power-up, the shift register is loaded with the value 1.
            envelope: Envelope::new(),
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
            self.envelope.period
        } else {
            self.envelope.decay_counter
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

    pub fn clock_length_counter(&mut self) {
        if !(self.length_counter == 0 || self.envelope.length_counter_halt) {
            self.length_counter -= 1;
        }
    }

    // $400C
    pub fn write_envelope(&mut self, value: u8) {
        self.envelope.length_counter_halt = (value >> 5) & 1 == 1;
        self.constant_volume_flag = (value >> 4) & 1 == 1;
        self.envelope.period = value as u16 & 0b1111;
    }

    // $400E
    pub fn write_loop_noise(&mut self, value: u8) {
        self.mode = value >> 7 == 1;
        self.timer_period = NOISE_TABLE[(value & 0b1111) as usize];
    }

    // $400F
    pub fn write_length_counter(&mut self, value: u8) {
        self.length_counter = value >> 3;
        self.envelope.start = true;
    }
}

// When the timer clocks the shift register, the following actions occur in order:

//    1. Feedback is calculated as the exclusive-OR of bit 0 and one other bit: bit 6 if Mode flag is set, otherwise bit 1.
//    2. The shift register is shifted right by one bit.
//    3. Bit 14, the leftmost bit, is set to the feedback calculated earlier.

// This results in a pseudo-random bit sequence, 32767 steps long when Mode flag is clear,
// and randomly 93 or 31 steps long otherwise. (The particular 31- or 93-step sequence depends
// on where in the 32767-step sequence the shift register was when Mode flag was set).
