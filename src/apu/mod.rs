mod noise;
mod square;
mod triangle;
mod dmc;

pub struct Apu {
    square1:  Square,
    square2:  Square,
    triangle: Triangle,
    noise:    Noise,
    dmc:      DMC,
}

struct Square {
    duty_cycle: usize,
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
    timer: usize,
    length_counter: usize, // (this bit is also the linear counter's control flag) 
    linear_counter: usize,
}

// $400E 	M---.PPPP 	Mode and period (write)
// bit 7 	M--- ---- 	Mode flag 
struct Noise {
    timer: usize,
    length_counter: usize,
    envelope: usize,
    linear_feedback_sr: u16,
    mode: bool, // also called loop noise, bit 7 of $400E
}

struct DMC {

}

impl Apu {
    fn new() -> Self {
        Apu {
            square1:    Square::new(),
            square2:    Square::new(),
            triangle: Triangle::new(),
            noise:       Noise::new(),
            dmc:           DMC::new(),
        }
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
            0x4017 => self.frame_counter(value),
        }
    }
}
