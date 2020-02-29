use super::cpu;
use super::ppu;
use super::apu;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SaveState {
    cpu: cpu::serialize::CpuData,
    ppu: ppu::serialize::PpuData,
    apu: apu::serialize::ApuData,
}

pub fn save_state(cpu: &cpu::Cpu, filename: &str) -> Result<(), String> {
    let data = SaveState{
        cpu: cpu.save_state(),
        ppu: cpu.ppu.save_state(),
        apu: cpu.apu.save_state(),
    };
    let serialized = serde_json::to_string(&data)
        .map_err(|e| e.to_string())?;
    let path = match Path::new(&filename).parent() {
        Some(p) => p,
        None => return Err("couldn't convert filename to path".to_string()),
    };
    let stem = match Path::new(&filename).file_stem() {
        Some(s) => s,
        None => return Err("couldn't get file stem".to_string()),
    };
    let mut save_file = path.join(stem);
    save_file.set_extension("dat");
    println!("state saved to file: {:?}", save_file);
    let mut f = File::create(&save_file)
        .expect("could not create output file for save state");
    f.write_all(serialized.as_bytes())
        .map_err(|_| "couldn't write serialized data to file".to_string())
}

pub fn load_state(cpu: &mut cpu::Cpu, filename: &str) -> Result<(), String> {
    // load file, deserialize to cpudata, set cpu fields to data fields
    let path = match Path::new(&filename).parent() {
        Some(p) => p,
        None => return Err("couldn't convert filename to path".to_string()),
    };
    let stem = match Path::new(&filename).file_stem() {
        Some(s) => s,
        None => return Err("couldn't get file stem".to_string()),
    };
    let mut save_file = path.join(stem);
    save_file.set_extension("dat");

    if Path::new(&save_file).exists() {
        let mut f = File::open(save_file.clone())
            .map_err(|e| e.to_string())?;
        let mut serialized_data = vec![];
        f.read_to_end(&mut serialized_data)
            .map_err(|e| e.to_string())?;
        let serialized_string = std::str::from_utf8(&serialized_data)
            .map_err(|e| e.to_string())?;
        let state: SaveState = serde_json::from_str(serialized_string)
            .map_err(|e| e.to_string())?;
        cpu.load_state(state.cpu);
        cpu.ppu.load_state(state.ppu);
        cpu.apu.load_state(state.apu);
        println!("loading save state from file: {:?}", save_file);
        Ok(())
    } else {
        Err(format!("no save state file at {:?}", save_file))
    }
}
