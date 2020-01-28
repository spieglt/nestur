pub struct Envelope {
    pub period: u16, // constant volume/envelope period
    divider: u16,
    pub decay_counter: u16,        // remainder of envelope divider
    pub start: bool,               // restarts envelope
    pub length_counter_halt: bool, // also the envelope loop flag
}

impl Envelope {
    pub fn new() -> Self {
        Envelope {
            period: 0,
            divider: 0,
            decay_counter: 0,
            start: false,
            length_counter_halt: false,
        }
    }

    pub fn clock(&mut self) {
        // When clocked by the frame counter, one of two actions occurs:
        // if the start flag is clear, the divider is clocked,
        if !self.start {
            self.clock_divider();
        } else {
            self.start = false; // otherwise the start flag is cleared,
            self.decay_counter = 15; // the decay level counter is loaded with 15,
            self.divider = self.period; // and the divider's period is immediately reloaded
        }
    }

    fn clock_divider(&mut self) {
        // When the divider is clocked while at 0, it is loaded with V and clocks the decay level counter.
        if self.divider == 0 {
            self.divider = self.period;
            // Then one of two actions occurs: If the counter is non-zero, it is decremented,
            if self.decay_counter != 0 {
                self.decay_counter -= 1;
            } else if self.length_counter_halt {
                // otherwise if the loop flag is set, the decay level counter is loaded with 15.
                self.decay_counter = 15;
            }
        } else {
            self.divider -= 1;
        }
    }
}
