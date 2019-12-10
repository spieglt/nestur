mod noise;
mod square;
mod triangle;
mod dmc;

// APU clock ticks every other CPU cycle.
// Frame counter only ticks every 3728.5 APU ticks, and in audio frames of 4 or 5.
// Length counter controls note durations.

pub struct Apu {
    square1:  Square,
    square2:  Square,
    triangle: Triangle,
    noise:    Noise,
    dmc:      DMC,
    
    square_table: Vec<f32>,
    tnd_table: Vec<f32>,

    frame_counter: u8,
    mode: u8,
    interrupt_inhibit: u8,
}

struct Square {
    sample: u16,
    duty_cycle: u8,
    length_counter_halt: bool, // (this bit is also the envelope's loop flag)
    constant_volume_flag: bool, // (0: use volume from envelope; 1: use constant volume)
    timer: usize,
    length_counter: usize,
    envelope: usize,
    sweep: usize,
}

// $4008 	Hlll.llll 	Triangle channel length counter halt and linear counter load (write)
// bit 7 	H--- ---- 	Halt length counter (this bit is also the linear counter's control flag) 
struct Triangle {
    sample: u16,
    timer: usize,
    length_counter: usize, // (this bit is also the linear counter's control flag) 
    linear_counter: usize,
}

// $400E 	M---.PPPP 	Mode and period (write)
// bit 7 	M--- ---- 	Mode flag 
struct Noise {
    sample: u16,
    timer: usize,
    length_counter: usize,
    envelope: usize,
    linear_feedback_sr: u16,
    mode: bool, // also called loop noise, bit 7 of $400E
}

struct DMC {
    sample: u16,
}

struct Envelope {
    start_flag: bool,
    divider: usize,
    delay_level_counter: usize,
}

struct FrameCounter {

}

impl Apu {
    pub fn new() -> Self {
        let square_table = (0..31).map(|x| 95.52/(8128.0 / x as f32) + 100.0).collect();
        let tnd_table = (0..203).map(|x| 163.67/(24329.0 / x as f32) + 100.0).collect();
        Apu {
            square1:    Square::new(),
            square2:    Square::new(),
            triangle: Triangle::new(),
            noise:       Noise::new(),
            dmc:           DMC::new(),

            square_table: square_table,
            tnd_table: tnd_table,

            frame_counter: 0,
            mode: 0,
            interrupt_inhibit: 0,
        }
    }

    pub fn clock(&mut self) {

    }

    fn write_reg(&mut self, address: usize, value: u8) {
        match address {
            0x4000 => self.square1.duty(value),
            0x4001 => self.square1.sweep(value),
            0x4002 => self.square1.timer_low(value),
            0x4003 => self.square1.timer_high(value),
            0x4004 => self.square2.duty(value),
            0x4005 => self.square2.sweep(value),
            0x4006 => self.square2.timer_low(value),
            0x4007 => self.square2.timer_high(value),
            0x4008 => self.triangle.counter(value),
            0x400A => self.triangle.timer_low(value),
            0x400B => self.triangle.timer_high(value),
            0x400C => self.noise.envelope(value),
            0x400D => (),
            0x400E => self.noise.loop_noise(value),
            0x400F => self.noise.load_length_counter(value),
            0x4010 => self.dmc.control(value),
            0x4011 => self.dmc.direct_load(value),
            0x4012 => self.dmc.sample_address(value),
            0x4013 => self.dmc.sample_length(value),
            0x4014 => (),
            0x4015 => self.control(value),
            0x4016 => (),
            0x4017 => self.step_frame_counter(value),
            _ => panic!("bad address written: 0x{:X}", address),
        }
    }

    fn mix(&self) -> f32 {
        let square_out = self.square_table[(self.square1.sample + self.square2.sample) as usize];
        let tnd_out = self.tnd_table[((3*self.triangle.sample)+(2*self.noise.sample) + self.dmc.sample) as usize];
        square_out + tnd_out
    }

    //     mode 0:    mode 1:       function
    // ---------  -----------  -----------------------------
    // - - - f    - - - - -    IRQ (if bit 6 is clear)
    // - l - l    - l - - l    Length counter and sweep
    // e e e e    e e e - e    Envelope and linear counter
    fn step_frame_counter(&mut self, value: u8) {
        // 0 selects 4-step sequence, 1 selects 5-step sequence
        if value & (1<<7) == 0 { 
            self.mode = 0;
            self.frame_counter = 4;
        } else {
            self.mode = 1;
            self.frame_counter = 5;
        }
        // If set, the frame interrupt flag is cleared, otherwise it is unaffected. 
        if value & (1<<6) != 0 {
            self.interrupt_inhibit = 0;
        }

    }

    fn control(&mut self, value: u8) {

    }
}
