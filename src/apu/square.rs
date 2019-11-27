impl super::Square {
    pub fn new() -> Self {
        super::Square {
            duty_cycle: 0,
            length_counter_halt: false,
            constant_volume_flag: false,
            timer: 0,
            length_counter: 0,
            envelope: 0,
            sweep: 0,
        }
    }
}