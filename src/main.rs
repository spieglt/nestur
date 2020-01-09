use std::time::{Instant, Duration};

mod cpu;
mod ppu;
mod apu;
mod cartridge;
mod input;
mod screen;
mod audio;

use cpu::Cpu;
use ppu::Ppu;
use apu::Apu;
use cartridge::Cartridge;
use input::poll_buttons;
use screen::{init_window, draw_pixel, draw_to_window};

use sdl2::keyboard::Keycode;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;

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
    let audio_device = audio::initialize(&sdl_context).expect("Could not create audio device");
    let mut half_cycle = false;
    audio_device.resume();

    // Initialize hardware components
    let cart = Cartridge::new();
    let ppu = Ppu::new(&cart);
    let apu = Apu::new();
    let mut cpu = Cpu::new(&cart, ppu, apu);

    // For throttling to 60 FPS
    let mut timer = Instant::now();
    let mut fps_timer = Instant::now();
    let mut fps = 0;
    let mut sps = 0;

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
            match cpu.apu.clock() {
                Some(sample) => {
                    sps += 1;
                    if sps < 44_100 {audio_device.queue(&vec![sample]);} // TODO: fix this
                    // audio_device.queue(&vec![sample]);
                },
                None => (),
            };
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
                let now = Instant::now();
                // if we're running faster than 60Hz, kill time
                if now < timer + Duration::from_millis(1000/60) {
                    std::thread::sleep(timer + Duration::from_millis(1000/60) - now);
                }
                timer = Instant::now();
                // listen for Esc or window close. TODO: does this prevent keyboard events from being handled?
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                            => { break 'running },
                        _ => (),
                    }
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

            println!("samples per second: {}", sps);
            sps = 0;

        }
    }
    // PROFILER.lock().unwrap().stop().unwrap();
    Ok(())
}

/*

TODO:
- common mappers
- DMC audio channel, high- and low-pass filters, refactor envelope
- name audio variables (dividers, counters, etc.) more consistently
- battery-backed RAM solution
- GUI? drag and drop ROMs?
- reset function
- save/load/pause functionality


Timing notes:
The PPU is throttled to 60Hz by sleeping in the main loop. This locks the CPU to roughly its intended speed, 1.789773MHz NTSC. The APU runs at half that.
The SDL audio device samples/outputs at 44,100Hz, so as long as the APU queues up 44,100 samples per second, it works.
But it's not doing so evenly. If PPU runs faster than 60Hz, audio will get skipped, and if slower, audio will pop/have gaps.
Need to probably lock everything to the APU but worried about checking time that often. Can do for some division of 44_100.

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
