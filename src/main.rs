use std::time::{Instant, Duration};

mod cpu;
mod ppu;
mod apu;
mod cartridge;
mod input;
mod screen;

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

    // Initialize hardware components
    let cart = Cartridge::new();
    let ppu = Ppu::new(&cart);
    let apu = Apu::new();
    let mut cpu = Cpu::new(&cart, ppu, apu);

    // For throttling to 60 FPS
    let mut timer = Instant::now();
    let mut fps_timer = Instant::now();
    let mut fps = 0;

    // PROFILER.lock().unwrap().start("./main.profile").unwrap();
    'running: loop {
        // perform 1 cpu instruction, getting back number of clock cycles it took
        let num_cycles = cpu.step();
        // maintain ratio of 3 ppu cycles for 1 cpu step
        for _ in 0..num_cycles * 3 {
            let (pixel, end_of_frame) = cpu.ppu.step();
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
        }
    }
    // PROFILER.lock().unwrap().stop().unwrap();
    Ok(())
}

// TODO: reset function?
// TODO: save/load functionality
