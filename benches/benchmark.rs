use chip_8::cpu::CPU;
use chip_8::cpu::disassemble;
use chip_8::ram::Ram;
use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;

fn disassemble_benchmark(c: &mut Criterion) {
    let rom_path = Path::new("/home/nima/Downloads/1-chip8-logo.ch8");
    let rom_data = std::fs::read(rom_path).unwrap();

    c.bench_function("disassemble", |b| b.iter(|| disassemble(&rom_data)));
}

fn cpu_fetch_benchmark(c: &mut Criterion) {
    let rom_path = Path::new("/home/nima/Downloads/1-chip8-logo.ch8");
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut cpu = CPU::new();
    cpu.pc = 0x200;

    let mut ram = Ram::new();
    ram.load_rom(&rom_data).unwrap();
    let memory = ram.memory;

    c.bench_function("cpu_fetch", |b| {
        b.iter(|| {
            cpu.pc %= 4096;
            cpu.fetch(memory);
        })
    });
}

criterion_group!(benches, disassemble_benchmark, cpu_fetch_benchmark);
criterion_main!(benches);
