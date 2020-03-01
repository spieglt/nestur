pub type ApuData = super::Apu;

impl super::Apu{
    pub fn save_state(&self) -> ApuData {
        self.clone()
    }

    pub fn load_state(&mut self, data: ApuData) {
        self.square1 = data.square1;
        self.square2 = data.square2;
        self.triangle = data.triangle;
        self.noise = data.noise;
        self.dmc = data.dmc;
        self.square_table = data.square_table;
        self.tnd_table = data.tnd_table;
        self.frame_sequence = data.frame_sequence;
        self.frame_counter = data.frame_counter;
        self.interrupt_inhibit = data.interrupt_inhibit;
        self.frame_interrupt = data.frame_interrupt;
        self.cycle = data.cycle;
        self.trigger_irq = data.trigger_irq;
    }
}
