mod cpu;
mod ppu;
mod apu;
mod cartridge;
mod input;
mod screen;
mod audio;
mod state;

use cpu::Cpu;
use ppu::Ppu;
use apu::Apu;
use cartridge::get_mapper;
use input::poll_buttons;
use screen::{init_window, draw_pixel, draw_to_window};
use state::{save_state, load_state, change_file_extension, find_next_filename};

use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use std::path::Path;

// use cpuprofiler::PROFILER;

fn main() -> Result<(), String> {
    // Set up screen
    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let (mut canvas, texture_creator) = init_window(&sdl_context).expect("Could not create window");
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24, 256*screen::SCALE_FACTOR as u32, 240*screen::SCALE_FACTOR as u32)
        .map_err(|e| e.to_string())?;
    let byte_width = 256 * 3 * screen::SCALE_FACTOR; // 256 NES pixels, 3 bytes for each pixel (RGB 24-bit), and NES-to-SDL scale factor
    let byte_height = 240 * screen::SCALE_FACTOR; // NES image is 240 pixels tall, multiply by scale factor for total number of rows needed
    let mut screen_buffer = vec![0; byte_width * byte_height]; // contains raw RGB data for the screen

    // Set up audio
    let mut temp_buffer = vec![]; // receives one sample each time the APU ticks. this is a staging buffer so we don't have to lock the mutex too much.
    let apu_buffer = Arc::new(Mutex::new(Vec::<f32>::new())); // stays in this thread, receives raw samples between frames
    let sdl_buffer = Arc::clone(&apu_buffer); // used in audio device's callback to select the samples it needs
    let audio_device = audio::initialize(&sdl_context, sdl_buffer).expect("Could not create audio device");
    let mut half_cycle = false;
    audio_device.resume();

    // Initialize hardware components
    let filename = get_filename();
    let mapper = get_mapper(filename.clone());
    let ppu = Ppu::new(mapper.clone());
    let apu = Apu::new();
    let mut cpu = Cpu::new(mapper.clone(), ppu, apu);

    // For throttling to 60 FPS
    let mut timer = Instant::now();
    let mut fps_timer = Instant::now();
    let mut fps = 0;

    // PROFILER.lock().unwrap().start("./main.profile").unwrap();
    'running: loop {
        // step CPU: perform 1 cpu instruction, getting back number of clock cycles it took
        let cpu_cycles = cpu.step();
        // clock APU every other CPU cycle
        let mut apu_cycles = cpu_cycles / 2;
        if cpu_cycles & 1 == 1 {   // if cpu step took an odd number of cycles
            if half_cycle {        // and we have a half-cycle stored
                apu_cycles += 1;   // use it
                half_cycle = false;
            } else {
                half_cycle = true; // or save it for next odd cpu step
            }
        }
        for _ in 0..apu_cycles {
            temp_buffer.push(cpu.apu.clock());
        }
        // clock PPU three times for every CPU cycle
        for _ in 0..cpu_cycles * 3 {
            let (pixel, end_of_frame) = cpu.ppu.clock();
            match pixel {
                Some((x, y, color)) => draw_pixel(&mut screen_buffer, x, y, color),
                None => (),
            };
            if end_of_frame {
                fps += 1; // keep track of how many frames we've rendered this second
                draw_to_window(&mut texture, &mut canvas, &screen_buffer)?; // draw the buffer to the window with SDL
                let mut b = apu_buffer.lock().unwrap(); // unlock mutex to the real buffer
                b.append(&mut temp_buffer); // send this frame's audio data, emptying the temp buffer
                let now = Instant::now();
                // if we're running faster than 60Hz, kill time
                if now < timer + Duration::from_millis(1000/60) {
                    std::thread::sleep(timer + Duration::from_millis(1000/60) - now);
                }
                timer = Instant::now();
                if !process_events(&mut event_pump, &filename, &mut cpu) {
                    break 'running;
                }
            }
        }
        // handle keyboard events
        match poll_buttons(&cpu.strobe, &event_pump) {
            Some(button_states) => cpu.button_states = button_states,
            None => (),
        };
        // calculate fps
        let now = Instant::now();
        if now > fps_timer + Duration::from_secs(1) {
            println!("fps: {}", fps);
            fps = 0;
            fps_timer = now;
        }
    }
    // PROFILER.lock().unwrap().stop().unwrap();
    mapper.borrow().save_battery_backed_ram();
    Ok(())
}

fn get_filename() -> String {
    let argv: Vec<String> = std::env::args().collect();
    assert!(argv.len() > 1, "must include .nes ROM as argument");
    argv[1].clone()
}

fn process_events(event_pump: &mut EventPump, filename: &str, cpu: &mut Cpu) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                => return false,
            Event::KeyDown{ keycode: Some(Keycode::F5), .. } => {
                let dat_filename = change_file_extension(&filename, "dat").unwrap();
                println!("{:?}", find_next_filename(&dat_filename));
                let res: Result<(), String> = save_state(cpu, dat_filename)
                    .or_else(|e| {println!("{}", e); Ok(())});
                res.unwrap();
            },
            Event::KeyDown{ keycode: Some(Keycode::F9), .. } => {
                let dat_filename = change_file_extension(&filename, "dat").unwrap();
                let res: Result<(), String> = load_state(cpu, dat_filename)
                    .or_else(|e| {println!("{}", e); Ok(())});
                res.unwrap();
            },
            Event::DropFile{ timestamp: _t, window_id: _w, filename: f } => {
                let p = Path::new(&f).to_path_buf();
                let res: Result<(), String> = load_state(cpu, p)
                    .or_else(|e| {println!("{}", e); Ok(())});
                res.unwrap();
            },
            _ => (),
        }
    }
    true
}

/*

TODO:
- high- and low-pass audio filters
- DMC audio channel
- untangle CPU and APU/PPU?
- GUI? drag and drop ROMs?
- reset function
- save states: multiple, search/select, and generalized "find file by different extension" functionality


Timing notes:
The PPU is throttled to 60Hz by sleeping in the main loop. This locks the CPU to roughly its intended speed, 1.789773MHz NTSC. The APU runs at half that.
The APU gives all of its samples to the SDL audio device, which takes them 60 times per second in batches of 735 (44,100/60). It selects the ones
it needs at the proper interval and truncates its buffer.

Failed tests from instr_test-v5/rom_singles/:
3, immediate, Failed. Just unofficial instructions?
    0B AAC #n
    2B AAC #n
    4B ASR #n
    6B ARR #n
    AB ATX #n
    CB AXS #n
7, abs_xy, 'illegal opcode using abs x: 9c'

*/
