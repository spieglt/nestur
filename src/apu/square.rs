const DUTY_CYCLE_SEQUENCES: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

pub struct Square {
    pub sample: u16,
    pub enabled: bool,

    duty_cycle: [u8; 8], // "sequencer", set to one of the lines in DUTY_CYCLE_SEQUENCES
    duty_counter: usize, // current index within the duty_cycle
    
    envelope: u16, // constant volume/envelope period. reflects what was last written to $4000/$4004
    envelope_divider: u16, //
    decay_counter: u16, // remainder of envelope divider
    constant_volume_flag: bool, // (0: use volume from envelope; 1: use constant volume)
    start: bool, // restarts envelope

    length_counter_halt: bool, // (this bit is also the envelope's loop flag)
    pub length_counter: u8,

    timer: u16,
    timer_period: u16,

    target_period: u16,
    sweep_divider: u16, // Period, P
    sweep_counter: u16,
    shift_count: u8,
    sweep_enabled: bool,
    sweep_negate: bool,
    sweep_reload: bool,

    second_channel: bool, // hack to detect timing difference in clock_sweep()
}

impl Square {
    pub fn new(second_channel: bool) -> Self {
        Square {
            sample: 0,
            enabled: false,

            duty_cycle: DUTY_CYCLE_SEQUENCES[0],
            duty_counter: 0,

            envelope: 0,
            envelope_divider: 0,
            decay_counter: 0,
            constant_volume_flag: false,
            start: false,

            timer: 0,
            timer_period: 0,

            target_period: 0,
            sweep_divider: 0,
            sweep_counter: 0,
            shift_count: 0,
            sweep_enabled: false,
            sweep_negate: false,
            sweep_reload: false,

            length_counter: 0,
            length_counter_halt: false,

            second_channel: second_channel,
        }
    }

    pub fn clock(&mut self) {
        // The sequencer is clocked by an 11-bit timer. Given the timer value t = HHHLLLLLLLL formed by timer high and timer low, this timer is updated every APU cycle
        // (i.e., every second CPU cycle), and counts t, t-1, ..., 0, t, t-1, ..., clocking the waveform generator when it goes from 0 to t.
        if self.timer == 0 {
            self.timer = self.timer_period;
            self.duty_counter = (self.duty_counter + 1) % 8;
        } else {
            self.timer -= 1;
        }
        // Update volume for this channel
        // The mixer receives the current envelope volume except when
        self.sample = if self.duty_cycle[self.duty_counter] == 0 // the sequencer output is zero, or
            || self.timer_period > 0x7FF // overflow from the sweep unit's adder is silencing the channel,
            || self.length_counter == 0 // the length counter is zero, or
            || self.timer_period < 8 // the timer has a value less than eight.
            {
                0
            } else if self.constant_volume_flag {
                self.envelope
            } else {
                self.decay_counter
            };
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
                // println!("decaying to {}", self.decay_counter);
            } else if self.length_counter_halt {
                // otherwise if the loop flag is set, the decay level counter is loaded with 15. 
                println!("looping decay counter");
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

    pub fn clock_sweep(&mut self) {
        self.calculate_target_period();
        // When the frame counter sends a half-frame clock (at 120 or 96 Hz), two things happen.
        // If the divider's counter is zero, the sweep is enabled, and the sweep unit is not muting the channel: The pulse's period is adjusted.
        if self.sweep_counter == 0 && self.sweep_enabled && !(self.timer_period < 8 || self.target_period > 0x7FF) {
            self.timer_period = self.target_period;
            println!("timer period adjusted to {}", self.timer_period);
        }
        // If the divider's counter is zero or the reload flag is true: The counter is set to P and the reload flag is cleared. Otherwise, the counter is decremented.
        if self.sweep_counter == 0 || self.sweep_reload {
            self.sweep_counter = self.sweep_divider;
            self.sweep_reload = false;
            if self.sweep_enabled { self.timer_period = self.target_period; } // This fixes the DK walking sound. Why? Not reflected in documentation.
        } else {
            self.sweep_counter -= 1;
        }
    }

    // Whenever the current period changes for any reason, whether by $400x writes or by sweep, the target period also changes. 
    pub fn calculate_target_period(&mut self) {
        // The sweep unit continuously calculates each channel's target period in this way:
        // A barrel shifter shifts the channel's 11-bit raw timer period right by the shift count, producing the change amount.
        let change = self.timer_period >> self.shift_count;
        // If the negate flag is true, the change amount is made negative.
        // The target period is the sum of the current period and the change amount.
        if self.sweep_negate {
            self.target_period = self.timer_period - change;
            // The two pulse channels have their adders' carry inputs wired differently,
            // which produces different results when each channel's change amount is made negative:
            // Pulse 1 adds the ones' complement (−c − 1). Making 20 negative produces a change amount of −21.
            // Pulse 2 adds the two's complement (−c). Making 20 negative produces a change amount of −20.
            if !self.second_channel {
                self.target_period -= 1;
            }
        } else {
            self.target_period = self.timer_period + change;
        }
    }

    // $4000/$4004
    pub fn write_duty(&mut self, value: u8) {
        // The duty cycle is changed (see table below), but the sequencer's current position isn't affected.
        self.duty_cycle = DUTY_CYCLE_SEQUENCES[(value >> 6) as usize];
        self.length_counter_halt = value & (1<<5) != 0;
        self.constant_volume_flag = value & (1<<4) != 0;
        self.envelope = value as u16 & 0b1111;
    }

    // $4001/$4005
    pub fn write_sweep(&mut self, value: u8) {
        // println!("writing sweep 0b{:08b}", value);
        self.sweep_enabled = value >> 7 == 1;
        self.sweep_divider = ((value as u16 >> 4) & 0b111) + 1;
        self.sweep_negate = value & 0b1000 != 0;
        self.shift_count = value & 0b111;
        self.sweep_reload = true;
    }

    // $4002/$4006
    pub fn write_timer_low(&mut self, value: u8) {
        self.timer_period &= 0b00000111_00000000; // mask off everything but high 3 bits of 11-bit timer
        self.timer_period |= value as u16; // apply low 8 bits
        self.calculate_target_period();
    }

    // $4003/$4007
    pub fn write_timer_high(&mut self, value: u8) {
        // LLLL.Lttt 	Pulse channel 1 length counter load and timer (write)
        // TODO: thought the below meant that the length counter was only set if the channel was enabled, but apparently not as not having it fixes start game noise in DK.
        // When the enabled bit is cleared (via $4015), the length counter is forced to 0 and cannot be changed until enabled is set again (the length counter's previous value is lost).
        if self.enabled {
            self.length_counter = super::LENGTH_COUNTER_TABLE[value as usize >> 3];
            // println!("val: 0b{:08b}, wrote length_counter {}", value, self.length_counter);
        }
        let timer_high = value as u16 & 0b0000_0111;
        self.timer_period &= 0b11111000_11111111; // mask off high 3 bits of 11-bit timer
        self.timer_period |= timer_high << 8; // apply high timer bits in their place
        self.calculate_target_period();
        // The sequencer is immediately restarted at the first value of the current sequence. The envelope is also restarted. The period divider is not reset.
        self.duty_counter = 0;
        self.start = true;
    }
}
