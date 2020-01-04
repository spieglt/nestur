mod noise;
mod square;
mod triangle;
mod dmc;

use noise::Noise;
use square::Square;
use triangle::Triangle;
use dmc::DMC;

// APU clock ticks every other CPU cycle.
// Frame counter only ticks every 3728.5 APU ticks, and in audio frames of 4 or 5.
// Length counter controls note durations.

// We need to take a sample 44100 times per second. The CPU clocks (not steps) at 1.789773 MHz. Meaning the APU, going half as fast,
// clocks 894,886.5 times per second. 894,886.5/44,100=20.29 APU clocks per audio sample.

// TODO: organize APU structs

const FRAME_COUNTER_STEPS: [usize; 5] = [3728, 7456, 11185, 14914, 18640];
const CYCLES_PER_SAMPLE: f32 = 894_886.5/44_100.0; // APU frequency over sample frequency. May need to turn this down slightly as it's outputting less than 44_100Hz.
// const CYCLES_PER_SAMPLE: f32 = 20.0;
const LENGTH_COUNTER_TABLE: [u8; 32] = [
    10, 254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
    12,  16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct Apu {
    square1:  Square,
    square2:  Square,
    triangle: Triangle,
    noise:    Noise,
    dmc:      DMC,
    
    square_table: Vec<f32>,
    tnd_table: Vec<f32>,

    frame_counter: u8,
    current_frame: u8,
    interrupt_inhibit: bool,
    frame_interrupt: bool,
    cycle: usize,
    remainder: f32, // keep sample at 44100Hz
    pub trigger_irq: bool,
}

impl Apu {
    pub fn new() -> Self {
        let square_table = (0..31).map(|x| 95.52/((8128.0 / x as f32) + 100.0)).collect();
        let tnd_table = (0..203).map(|x| 163.67/((24329.0 / x as f32) + 100.0)).collect();
        Apu {
            square1:    Square::new(false),
            square2:    Square::new(true),
            triangle: Triangle::new(),
            noise:       Noise::new(),
            dmc:           DMC::new(),

            square_table: square_table,
            tnd_table: tnd_table,

            frame_counter: 0,
            current_frame: 0,
            interrupt_inhibit: false,
            frame_interrupt: false,
            cycle: 0,
            remainder: 0_f32,
            trigger_irq: false,
        }
    }

    pub fn clock(&mut self) -> Option<f32> {
        let mut sample = None;

        // Clock each channel
        self.square1.clock();
        self.square2.clock();
        self.triangle.clock();
        self.triangle.clock(); // TODO: hacky. clocking triangle twice because it runs every CPU cycle
        self.noise.clock();
        self.dmc.clock();

        // Send sample to buffer if necessary
        if self.remainder > CYCLES_PER_SAMPLE { 
            sample = Some(self.mix());
            self.remainder -= 20.0;
        }
        self.remainder += 1.0;

        // Step frame counter if necessary
        if FRAME_COUNTER_STEPS.contains(&self.cycle) {
            self.clock_frame_counter();
        }
        self.cycle += 1;
        if (self.frame_counter == 4 && self.cycle == 14915) || self.cycle == 18641 {
            self.cycle = 0;
        }

        sample
    }

    fn mix(&self) -> f32 {
        let square_out = self.square_table[(self.square1.sample + self.square2.sample) as usize];
        let tnd_out = self.tnd_table[((3*self.triangle.sample)+(2*self.noise.sample) + self.dmc.sample) as usize];
        square_out + tnd_out
    }

    pub fn write_reg(&mut self, address: usize, value: u8) {
        // println!("writing 0b{:08b} to 0x{:X}", value, address);
        match address {
            0x4000 => self.square1.write_duty(value),
            0x4001 => self.square1.write_sweep(value),
            0x4002 => self.square1.write_timer_low(value),
            0x4003 => self.square1.write_timer_high(value),
            0x4004 => self.square2.write_duty(value),
            0x4005 => self.square2.write_sweep(value),
            0x4006 => self.square2.write_timer_low(value),
            0x4007 => self.square2.write_timer_high(value),
            0x4008 => self.triangle.write_counter(value),
            0x4009 => (),
            0x400A => self.triangle.write_timer_low(value),
            0x400B => self.triangle.write_timer_high(value),
            0x400C => self.noise.write_envelope(value),
            0x400D => (),
            0x400E => self.noise.write_loop_noise(value),
            0x400F => self.noise.write_length_counter(value),
            0x4010 => self.dmc.write_control(value),
            0x4011 => self.dmc.direct_load(value),
            0x4012 => self.dmc.write_sample_address(value),
            0x4013 => self.dmc.write_sample_length(value),
            0x4014 => (),
            0x4015 => self.write_control(value),
            0x4016 => (),
            0x4017 => self.write_frame_counter(value),
            _ => panic!("bad address written: 0x{:X}", address),
        }
    }

    //   mode 0:    mode 1:       function
    // ---------  -----------  -----------------------------
    // - - - f    - - - - -    IRQ (if bit 6 is clear)
    // - l - l    - l - - l    Length counter and sweep
    // e e e e    e e e - e    Envelope and linear counter
    fn clock_frame_counter(&mut self) {
        if !(self.frame_counter == 5 && self.current_frame == 3) {
            // step envelopes
            self.square1.clock_envelope();
            self.square2.clock_envelope();
            self.triangle.clock_linear_counter();
            self.noise.clock_envelope();
        }
        if (self.current_frame == 1)
            || (self.frame_counter == 4 && self.current_frame == 3)
            || (self.frame_counter == 5 && self.current_frame == 4)
        {
            // step length counters and sweep units
            self.square1.clock_sweep();
            self.square2.clock_sweep();
            self.square1.clock_length_counter();
            self.square2.clock_length_counter();
            self.triangle.clock_length_counter();
            self.noise.clock_length_counter();
        }
        if self.frame_counter == 4 && self.current_frame == 3 && !self.interrupt_inhibit {
            self.trigger_irq = true;
        }
        // advance counter
        self.current_frame += 1;
        if self.current_frame == self.frame_counter {
            self.current_frame = 0;
        }
    }

    // CPU writes to $4015
    fn write_control(&mut self, value: u8) {
        // Writing to this register clears the DMC interrupt flag.
        self.dmc.interrupt = false;
        // Writing a zero to any of the channel enable bits will silence that channel and immediately set its length counter to 0.
        if value & (1<<0) != 0 {
            self.square1.enabled = true;
        } else {
            self.square1.enabled = false;
            self.square1.length_counter = 0;
        }
        if value & (1<<1) != 0 {
            self.square2.enabled = true;
        } else {
            self.square2.enabled = false;
            self.square2.length_counter = 0;
        }
        if value & (1<<2) != 0 {
            self.triangle.enabled = true;
        } else {
            self.triangle.enabled = false;
            self.triangle.length_counter = 0;
        }
        if value & (1<<3) != 0 {
            self.noise.enabled = true;
        } else {
            self.noise.enabled = false;
            self.noise.length_counter = 0;
        }
        if value & (1<<4) != 0 {
            self.dmc.enabled = true;
            // If the DMC bit is set, the DMC sample will be restarted only if its bytes remaining is 0.
            // If there are bits remaining in the 1-byte sample buffer, these will finish playing before the next sample is fetched.
            if self.dmc.bytes_remaining != 0 {
                // TODO: how does dmc repeat?
            }
        } else {
            self.dmc.enabled = false;
            self.dmc.length_counter = 0;
            // If the DMC bit is clear, the DMC bytes remaining will be set to 0 and the DMC will silence when it empties.
            self.dmc.bytes_remaining = 0;
        }
    }

    // CPU reads from $4015
    pub fn read_status(&mut self) -> u8 {
        // IF-D NT21: 	DMC interrupt (I), frame interrupt (F), DMC active (D), length counter > 0 (N/T/2/1)
        let mut val = 0;
        // N/T/2/1 will read as 1 if the corresponding length counter is greater than 0. For the triangle channel, the status of the linear counter is irrelevant.
        if self.square1.length_counter != 0 {
            val |= 1<<0;
        }
        if self.square2.length_counter != 0 {
            val |= 1<<1;
        }
        if self.triangle.length_counter != 0 {
            val |= 1<<2;
        }
        if self.noise.length_counter != 0 {
            val |= 1<<3;
        }
        // D will read as 1 if the DMC bytes remaining is more than 0.
        if self.dmc.bytes_remaining != 0 {
            val |= 1<<4;
        }
        if self.frame_interrupt {
            val |= 1<<6;
        }
        if self.dmc.interrupt {
            val |= 1<<7;
        }
        // Reading this register clears the frame interrupt flag (but not the DMC interrupt flag).
        self.frame_interrupt = false;
        // TODO: If an interrupt flag was set at the same moment of the read, it will read back as 1 but it will not be cleared.
        val
    }

    // $4017
    fn write_frame_counter(&mut self, value: u8) {
        // 0 selects 4-step sequence, 1 selects 5-step sequence
        self.frame_counter = if value & (1<<7) == 0 { 4 } else { 5 };
        // If set, the frame interrupt flag is cleared, otherwise it is unaffected. 
        if value & (1<<6) != 0 {
            self.interrupt_inhibit = false;
        }
        // If the mode flag is set, then both "quarter frame" and "half frame" signals are also generated.
        if self.frame_counter == 5 {
            // Clock envelopes, length counters, and sweep units
            self.square1.clock_envelope();
            self.square1.clock_sweep();
            self.square1.clock_length_counter();
            self.square2.clock_envelope();
            self.square2.clock_sweep();
            self.square2.clock_length_counter();
            self.triangle.clock_linear_counter();
            self.triangle.clock_length_counter();
            self.noise.clock_envelope();
            self.noise.clock_length_counter();
        }
    }
}
