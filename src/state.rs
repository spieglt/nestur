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
    let mut f = File::create(&save_file)
        .expect("could not create output file for save state");
    f.write_all(serialized.as_bytes())
        .map_err(|_| "couldn't write serialized data to file".to_string())?;
    println!("state saved to file: {:?}", save_file);
    Ok(())
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

pub fn change_file_extension(filename: &str, extension: &str) -> Option<PathBuf> {
    let path = Path::new(filename).parent()?;
    let stem = Path::new(&filename).file_stem()?;
    let mut save_file = path.join(stem);
    save_file.set_extension(extension);
    Some(save_file)
}

pub fn find_next_filename(filepath: &PathBuf) -> Option<PathBuf> {
    let path = filepath.parent()?.to_str()?;
    let stem = filepath.file_stem()?.to_str()?;
    let ext = filepath.extension()?.to_str()?;
    let sep = std::path::MAIN_SEPARATOR.to_string();
    let mut i = 0;
    loop {
        let current_name = format!("{}{}{}-{}.{}", path, sep, stem, i, ext);
        let save_file = PathBuf::from(&current_name);
        if !save_file.exists() {
            return Some(save_file)
        }
        i += 1;
    }
}

pub fn find_last_filename(filepath: &PathBuf) -> Option<PathBuf> {
    let path = filepath.parent()?;
    let stem = filepath.file_stem()?.to_str()?;
    let files = std::fs::read_dir(path).expect("couldn't read directory");
    let save_states = files.filter(|f| {
        let n = f.unwrap().file_name().to_str().unwrap();
        &n[..stem.len()] == stem && &n[n.len()-4..] == ".dat"
    }).collect::<Result<Vec<_>, std::io::Error>>().unwrap();
    save_states.sort();
}
