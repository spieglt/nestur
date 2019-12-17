const duty_cycle_sequences: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

pub struct Square {
    pub sample: u16,
    duty_cycle: [u8; 8],
    duty_counter: u8,
    length_counter_halt: bool, // (this bit is also the envelope's loop flag)
    constant_volume_flag: bool, // (0: use volume from envelope; 1: use constant volume)
    timer: usize,
    pub length_counter: usize,
    envelope: usize,
    sweep: usize,
    pub enabled: bool,
}

impl Square {
    pub fn new() -> Self {
        Square {
            duty_cycle: duty_cycle_sequences[0],
            duty_counter: 0,
            length_counter_halt: false,
            constant_volume_flag: false,
            timer: 0,
            length_counter: 0,
            envelope: 0,
            sweep: 0,
            sample: 0,
            enabled: false,
        }
    }

    pub fn clock(&mut self) {

    }

    pub fn duty(&mut self, value: u8) {
        self.duty_cycle = duty_cycle_sequences[(value >> 6) as usize];
        self.length_counter_halt = value & (1<<5) != 0;
        self.constant_volume_flag = value & (1<<4) != 0;
        
    }

    pub fn sweep(&mut self, value: u8) {
        
    }

    pub fn timer_low(&mut self, value: u8) {

    }

    pub fn timer_high(&mut self, value: u8) {
        
    }
}

struct EnvelopeGenerator {

}

struct SweepUnit {

}

struct Timer {

}

struct Sequencer {

}

struct LengthCounter {

}
