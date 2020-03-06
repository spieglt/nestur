use super::Mode;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuData {
    mem: Vec<u8>,
    a: u8,
    x: u8,
    y: u8,
    pc: usize,
    s: u8,
    p: u8,
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
            a: self.a,
            x: self.x,
            y: self.y,
            pc: self.pc,
            s: self.s,
            p: self.p,
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
        self.a = data.a;
        self.x = data.x;
        self.y = data.y;
        self.pc = data.pc;
        self.s = data.s;
        self.p = data.p;
        self.clock = data.clock;
        self.delay = data.delay;
        self.strobe = data.strobe;
        self.button_states = data.button_states;
        self.button_number = data.button_number;
        self.mode_table = data.mode_table;
    }
}
