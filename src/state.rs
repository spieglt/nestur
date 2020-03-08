use super::cpu;
use super::ppu;
use super::apu;
use super::cartridge;

use std::fs::{DirEntry, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(serde::Serialize, serde::Deserialize)]
struct SaveState {
    cpu: cpu::serialize::CpuData,
    ppu: ppu::serialize::PpuData,
    apu: apu::serialize::ApuData,
    mapper: cartridge::serialize::MapperData,
}

pub fn save_state(cpu: &cpu::Cpu, save_file: &PathBuf) -> Result<(), String> {
    let data = SaveState{
        cpu: cpu.save_state(),
        ppu: cpu.ppu.save_state(),
        apu: cpu.apu.save_state(),
        mapper: cpu.mapper.borrow().save_state(),
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

pub fn load_state(cpu: &mut cpu::Cpu, save_file: &PathBuf) -> Result<(), String> {
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
        cpu.mapper.borrow_mut().load_state(state.mapper);
        println!("loading save state from file: {:?}", save_file);
        Ok(())
    } else {
        Err(format!("no save state file at {:?}", save_file))
    }
}

pub fn find_next_filename(filepath: &PathBuf, new_ext: Option<&str>) -> Option<PathBuf> {
    let path = filepath.parent()?.to_str()?;
    let stem = filepath.file_stem()?.to_str()?;
    let ext = new_ext.or(Some(filepath.extension()?.to_str()?)).unwrap();
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

pub fn find_last_save_state(filepath: &PathBuf, new_ext: Option<&str>) -> Option<PathBuf> {
    let path = filepath.parent()?;
    let stem = filepath.file_stem()?.to_str()?;
    let ext = new_ext.or(Some(filepath.extension()?.to_str()?)).unwrap();
    let files = std::fs::read_dir(path).expect("couldn't read directory");
    let mut save_states = files
        .filter(|dir_entry| {
            let os_name = dir_entry.as_ref().unwrap().file_name();
            let name = os_name.to_str().unwrap();
            name.len() >= stem.len()
                && name.len() >= ext.len()
                && &name[..stem.len()] == stem
                && &name[name.len()-ext.len()..] == ext
        })
        .collect::<Vec<std::io::Result<DirEntry>>>();
    save_states.sort_by(|a, b| {
        let a_mod_time = a.as_ref().unwrap().metadata().unwrap().modified().unwrap();
        let b_mod_time = b.as_ref().unwrap().metadata().unwrap().modified().unwrap();
        b_mod_time.cmp(&a_mod_time) // puts in reverse order by last modified time
    });
    match save_states.len() {
        0 => None,
        _ => Some(save_states[0].as_ref().unwrap().path()),
    }
}
