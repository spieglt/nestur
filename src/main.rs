use std::time::{Instant, Duration};

mod cpu;
mod ppu;
mod cartridge;
mod screen;
mod input;

use cpu::Cpu;
use ppu::Ppu;
use cartridge::Cartridge;
use screen::Screen;
use input::poll_buttons;

use sdl2::keyboard::Keycode;
use sdl2::event::Event;

// use cpuprofiler::PROFILER;

fn main() -> Result<(), String> {
	let sdl_context = sdl2::init()?;
	let mut event_pump = sdl_context.event_pump()?;
    let mut screen = Screen::new(&sdl_context)?;

    let cart = Cartridge::new();

    let ppu = Ppu::new(&cart);
    let mut cpu = Cpu::new(&cart, ppu);

    let mut timer = Instant::now();
    let mut fps_timer = Instant::now();
    let mut fps = 0;

    // PROFILER.lock().unwrap().start("./main.profile").unwrap();
    'running: loop {
        // perform 1 cpu instruction, getting back number of clock cycles it took
        let num_cycles = cpu.step();
        // maintain ratio of 3 ppu cycles for 1 cpu step
        for _i in 0..num_cycles * 3 {
            let (pixel, end_of_frame) = cpu.ppu.step();
            match pixel {
                Some((x, y, color)) => screen.draw_pixel(x, y, color),
                None => Ok(()),
            }?;
            if end_of_frame {
                fps += 1;
                screen.canvas.present();
                let now = Instant::now();
                if now < timer + Duration::from_millis(1000/60) {
                    std::thread::sleep(timer + Duration::from_millis(1000/60) - now);
                }
                timer = Instant::now();
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                            break 'running
                        },
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
