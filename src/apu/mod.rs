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
    
    square_table: Vec<f32>,
    tnd_table: Vec<f32>,
}

struct Square {
    sample: u16,
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

}

impl Apu {
    fn new() -> Self {
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

    fn mix(&self) -> f32 {
        let square_out = self.square_table[self.square1.sample + self.square2.sample as usize];
        let tnd_out = self.tnd_table[(3*self.triangle.sample)+(2*self.noise.sample)+self.dmc.sample as usize];
        square_out + tnd_out
    }

    fn frame_counter(value: u8) {
        
    }
}
