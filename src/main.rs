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
use cartridge::{check_signature, get_mapper};
use input::poll_buttons;
use screen::{init_window, draw_to_window};
use state::{save_state, load_state, find_next_filename, find_last_save_state};

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

use sdl2::Sdl;
use sdl2::render::{Canvas, Texture};
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::Window;
use sdl2::messagebox::*;

use cpuprofiler::PROFILER;

enum GameExitMode {
    QuitApplication,
    NewGame(String),
    Reset,
    Nothing,
}

fn main() -> Result<(), String> {
    // Set up screen
    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let (mut canvas, texture_creator) = init_window(&sdl_context).expect("Could not create window");
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24, 256 as u32, 240 as u32)
        .map_err(|e| e.to_string())?;
    // let mut screen_buffer = vec![0; 256 * 240 * 3]; // contains raw RGB data for the screen

    let argv = std::env::args().collect::<Vec<String>>();
    let mut filename = if argv.len() > 1 {
        argv[1].to_string()
    } else {
        show_simple_message_box(
            MessageBoxFlag::INFORMATION, "Welcome to Nestur!", INSTRUCTIONS, canvas.window()
        ).map_err(|e| e.to_string())?;
        let name;
        'waiting: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                        => return Ok(()),
                    Event::DropFile{ filename: f, .. } => {
                        match check_signature(&f) {
                            Ok(()) => {
                                name = f;
                                break 'waiting;
                            },
                            Err(e) => println!("{}", e),
                        }
                    },
                    _ => (),
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        name
    };
    loop {
        let res = run_game(&sdl_context, &mut event_pump, &mut canvas, &mut texture, &filename);
        match res {
            Ok(Some(GameExitMode::Reset)) => (),
            Ok(Some(GameExitMode::NewGame(next_file))) => filename = next_file,
            Ok(None) | Ok(Some(GameExitMode::QuitApplication)) => return Ok(()),
            Err(e) => return Err(e),
            Ok(Some(GameExitMode::Nothing)) => panic!("shouldn't have returned exit mode Nothing to main()"),
        }
    }
}

fn run_game(
        sdl_context: &Sdl,
        event_pump: &mut EventPump,
        canvas: &mut Canvas<Window>,
        texture: &mut Texture,
        filename: &str
    ) -> Result<Option<GameExitMode>, String> {

    println!("loading game {}", filename);

    // Set up audio
    // let mut temp_buffer = vec![]; // receives one sample each time the APU ticks. this is a staging buffer so we don't have to lock the mutex too much.
    let apu_buffer = Arc::new(Mutex::new(Vec::<f32>::new())); // stays in this thread, receives raw samples between frames
    let sdl_buffer = Arc::clone(&apu_buffer); // used in audio device's callback to select the samples it needs
    let audio_device = audio::initialize(sdl_context, sdl_buffer).expect("Could not create audio device");
    let mut half_cycle = false;
    let mut audio_started = false;

    // Initialize hardware components
    let filepath = Path::new(filename).to_path_buf();
    let mapper = get_mapper(filename.to_string());
    let ppu = Ppu::new(mapper);
    let apu = Apu::new();
    let mut cpu = Cpu::new(ppu, apu);

    // For throttling to 60 FPS
    // let mut timer = Instant::now();
    let mut fps_timer = Instant::now();
    let mut fps = 0;
    let mut timer_counter = 0; // used to only check time every so many cycles

    // if cpu budget > 0, clock cpu and add triple cycles to ppu and half cycles to apu
    // if not, clock ppu 3 times, and add 1 to cpu budget, half cycle to apu
    // cpu budget += cpu.ppu.clock()
    let mut cpu_budget: i32 = 1;

    PROFILER.lock().unwrap().start("./main.profile").unwrap();
    'running: loop {




        if cpu_budget > 0 {
            // println!("cpu, budget before: {}", cpu_budget);
            let cpu_cycles = cpu.step();
            cpu_budget -= cpu_cycles as i32; // should be safe because no 1 cpu instruction takes more than a few clock cycles
            // clock APU every other CPU cycle
            // let mut apu_cycles = cpu_cycles / 2;
            // if cpu_cycles & 1 == 1 {   // if cpu step took an odd number of cycles
            //     if half_cycle {        // and we have a half-cycle stored
            //         apu_cycles += 1;   // use it
            //         half_cycle = false;
            //     } else {
            //         half_cycle = true; // or save it for next odd cpu step
            //     }
            // }
            // for _ in 0..apu_cycles {
            //     // can't read CPU from APU so have to pass byte in here
            //     let sample_byte = cpu.read(cpu.apu.dmc.current_address);
            //     temp_buffer.push(cpu.apu.clock(sample_byte));
            // }
            // println!("cpu, budget after: {}", cpu_budget);
        } else {
            // println!("ppu, budget before: {}", cpu_budget);
            for _ in 0..3 {
                let (end_of_frame, rendered_scanline) = cpu.ppu.new_clock();
                if rendered_scanline {
                    cpu_budget += 3;
                    // println!("ppu, budget after: {}", cpu_budget);
                }
                if end_of_frame {
                    fps += 1; // keep track of how many frames we've rendered this second
                    draw_to_window(texture, canvas, &cpu.ppu.screen_buffer)?; // draw the buffer to the window with SDL
                    // let mut b = apu_buffer.lock().unwrap(); // unlock mutex to the real buffer
                    // b.append(&mut temp_buffer); // send this frame's audio data, emptying the temp buffer
                    // if !audio_started {
                    //     audio_started = true;
                    //     audio_device.resume();
                    // }
                    /*let now = Instant::now();
                    // if we're running faster than 60Hz, kill time
                    if now < timer + Duration::from_millis(1000/60) {
                        std::thread::sleep(timer + Duration::from_millis(1000/60) - now);
                    }
                    timer = Instant::now();*/
                    let outcome = process_events(event_pump, &filepath, &mut cpu);
                    match outcome {
                        GameExitMode::QuitApplication => break 'running,
                        GameExitMode::Reset => return Ok(Some(GameExitMode::Reset)),
                        GameExitMode::NewGame(g) => return Ok(Some(GameExitMode::NewGame(g))),
                        GameExitMode::Nothing => (),
                    }
                }
            }
            cpu_budget += 1;
            // println!("ppu, budget after: {}", cpu_budget);
        }
        // println!("===============");


/*
        // step CPU: perform 1 cpu instruction, getting back number of clock cycles it took
        let cpu_cycles = cpu.step();
        // clock APU every other CPU cycle
        // let mut apu_cycles = cpu_cycles / 2;
        // if cpu_cycles & 1 == 1 {   // if cpu step took an odd number of cycles
        //     if half_cycle {        // and we have a half-cycle stored
        //         apu_cycles += 1;   // use it
        //         half_cycle = false;
        //     } else {
        //         half_cycle = true; // or save it for next odd cpu step
        //     }
        // }
        // for _ in 0..apu_cycles {
        //     // can't read CPU from APU so have to pass byte in here
        //     let sample_byte = cpu.read(cpu.apu.dmc.current_address);
        //     temp_buffer.push(cpu.apu.clock(sample_byte));
        // }
        // clock PPU three times for every CPU cycle
        for _ in 0..cpu_cycles * 3 {
            let end_of_frame = cpu.ppu.clock();
            if end_of_frame {
                fps += 1; // keep track of how many frames we've rendered this second
                draw_to_window(texture, canvas, &cpu.ppu.screen_buffer)?; // draw the buffer to the window with SDL
                // let mut b = apu_buffer.lock().unwrap(); // unlock mutex to the real buffer
                // b.append(&mut temp_buffer); // send this frame's audio data, emptying the temp buffer
                // if !audio_started {
                //     audio_started = true;
                //     audio_device.resume();
                // }
                /*let now = Instant::now();
                // if we're running faster than 60Hz, kill time
                if now < timer + Duration::from_millis(1000/60) {
                    std::thread::sleep(timer + Duration::from_millis(1000/60) - now);
                }
                timer = Instant::now();*/
                let outcome = process_events(event_pump, &filepath, &mut cpu);
                match outcome {
                    GameExitMode::QuitApplication => break 'running,
                    GameExitMode::Reset => return Ok(Some(GameExitMode::Reset)),
                    GameExitMode::NewGame(g) => return Ok(Some(GameExitMode::NewGame(g))),
                    GameExitMode::Nothing => (),
                }
            }
        }
*/

        // handle keyboard events
        match poll_buttons(&cpu.strobe, &event_pump) {
            Some(button_states) => cpu.button_states = button_states,
            None => (),
        };
        // calculate fps
        if timer_counter == 999 {
            let now = Instant::now();
            if now > fps_timer + Duration::from_secs(1) {
                println!("frames per second: {}", fps);
                fps = 0;
                fps_timer = now;
            }
            timer_counter = 0;
        } else {
            timer_counter += 1;
        }
    }
    PROFILER.lock().unwrap().stop().unwrap();
    cpu.ppu.mapper.save_battery_backed_ram();
    Ok(None)
}

fn process_events(event_pump: &mut EventPump, filepath: &PathBuf, cpu: &mut Cpu) -> GameExitMode {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                => return GameExitMode::QuitApplication,
            Event::KeyDown{ keycode: Some(Keycode::F2), .. }
                => return GameExitMode::Reset,
            Event::KeyDown{ keycode: Some(Keycode::F5), .. } => {
                let save_file = find_next_filename(filepath, Some("dat"))
                    .expect("could not generate save state filename");
                let res: Result<(), String> = save_state(cpu, &save_file)
                    .or_else(|e| {println!("{}", e); Ok(())});
                res.unwrap();
            },
            Event::KeyDown{ keycode: Some(Keycode::F9), .. } => {
                match find_last_save_state(filepath, Some("dat")) {
                    Some(p) => {
                        let res: Result<(), String> = load_state(cpu, &p)
                            .or_else(|e| { println!("{}", e); Ok(()) } );
                        res.unwrap();
                    },
                    None => println!("no save state found for {:?}", filepath)
                }
            },
            Event::DropFile{ timestamp: _t, window_id: _w, filename: f } => {
                if f.len() > 4 && &f[f.len()-4..] == ".dat" {
                    let p = Path::new(&f).to_path_buf();
                    let res: Result<(), String> = load_state(cpu, &p)
                        .or_else(|e| {println!("{}", e); Ok(())});
                    res.unwrap();
                // } else if f.len() > 4 && &f[f.len()-4..] == ".nes" {
                } else {
                    match check_signature(&f) {
                        Ok(()) => return GameExitMode::NewGame(f),
                        Err(e) => println!("{}", e),
                    }
                }
            },
            _ => (),
        }
    }
    return GameExitMode::Nothing
}

const INSTRUCTIONS: &str = "To play a game, drag an INES file (extension .nes) onto the main window.
To save the game state, press F5. To load the most recent save state, press F9.
To load another save state file, drag a .dat file onto the window while the game is running.
Battery-backed RAM saves (what the NES cartridges have) will be written to a .sav file if used.
To reset the console/current game, press F2.

Controls
------------
A: D
B: F
Start: enter
Select: (right) shift
Up/Down/Left/Right: arrow keys
";

/*

TODO:
- untangle CPU and APU/PPU?
- better save file organization?


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
