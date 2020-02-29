use super::Mode;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuData {
    mem: Vec<u8>,
    A: u8,
    X: u8,
    Y: u8,
    PC: usize,
    S: u8,
    P: u8,
    clock: u64,
    delay: usize,
    strobe: u8,
    button_states: u8,
    button_number: u8,
    mode_table: Vec<Mode>,
}

impl super::Cpu {
    pub fn save_state(&self) -> CpuData {
        CpuData{
            mem: self.mem.clone(),
            A: self.A,
            X: self.X,
            Y: self.Y,
            PC: self.PC,
            S: self.S,
            P: self.P,
            clock: self.clock,
            delay: self.delay,
            strobe: self.strobe,
            button_states: self.button_states,
            button_number: self.button_number,
            mode_table: self.mode_table.clone(),
        }
    }

    pub fn load_state(&mut self, data: CpuData) {
        self.mem = data.mem;
        self.A = data.A;
        self.X = data.X;
        self.Y = data.Y;
        self.PC = data.PC;
        self.S = data.S;
        self.P = data.P;
        self.clock = data.clock;
        self.delay = data.delay;
        self.strobe = data.strobe;
        self.button_states = data.button_states;
        self.button_number = data.button_number;
        self.mode_table = data.mode_table;
    }
}
