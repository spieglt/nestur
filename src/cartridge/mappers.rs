
pub fn nrom_cpu(cpu: &mut crate::cpu::Cpu, address: usize, writing: bool) -> Option<&mut u8> {
    // PRG-ROM, not -RAM
    if writing { return None };

    // CPU $8000-$BFFF: First 16 KB of ROM.
    // CPU $C000-$FFFF: Last 16 KB of ROM (NROM-256) or mirror of $8000-$BFFF (NROM-128).
    let l = cpu.prg_rom.len();
    match address {
        0x8000..=0xBFFF => Some(&mut cpu.prg_rom[0][address % 0x4000]),
        0xC000..=0xFFFF => Some(&mut cpu.prg_rom[l - 1][address % 0x4000]),
        _ => panic!("bad cpu address passed to nrom mapper"),
    }
}

pub fn nrom_ppu(ppu: &mut crate::ppu::Ppu, address: usize, writing: bool) -> Option<&mut u8> {
    let l = ppu.pattern_tables.len();
    // NROM/mapper 0 doesn't allow writes to CHR-ROM
    if writing || l == 0 { return None };
    match address {
        0x0000..=0x1FFF => Some(&mut ppu.pattern_tables[l-1][address]),
        _ => panic!("bad ppu address passed to nrom mapper: 0x{:04x}", address),
    }
}

pub fn get_mapper_funcs(mapper: u8) -> (super::CpuMapperFunc, super::PpuMapperFunc) {
    match mapper {
        0 => (nrom_cpu, nrom_ppu),
        _ => panic!("unimplemented mapper: {}", mapper),
    }
}
