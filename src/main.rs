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

    // TODO: remove
    // check for location of VerticalPipeEntry
    // println!("verticalPipeEntry: {:02X?}", cpu.memory_at(0xB225, 512));
    // why not just dump all memory?
    // let mut mem = cpu.memory_at(0, 0x4020);
    // let mut mem2 = cpu.memory_at(0x8000, 0xFFFF-0x8000);
    // mem.append(&mut mem2);
    // let mut line = 0;
    // for i in 0..0x4020 {
    //     if i % 0x10 == 0 {
    //         print!("\n0x{:04X}:  ", i);
    //     }
    //     print!("{:02X} ", mem[i]);
    // }
    // println!("\n=========================");
    // for i in 0x8000..=0xFFFF {
    //     if i % 0x10 == 0 {
    //         print!("\n0x{:04X}:  ", i);
    //     }
    //     print!("{:02X} ", mem[i-0x4020]);
    // }

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
            // println!("fps: {}", fps);
            fps = 0;
            fps_timer = now;

            // println!("samples per second: {}", sps);
            sps = 0;

        }
    }
    // PROFILER.lock().unwrap().stop().unwrap();
    Ok(())
}

/*
TODO:
- DMC audio channel, high- and low-pass filters, refactor envelope
- name audio variables (dividers, counters, etc.) more consistently
- common mappers
- battery-backed RAM solution
- fix mysterious Mario pipe non-locations
- GUI? drag and drop ROMs?
- reset function
- save/load/pause functionality


Timing notes:
The PPU is throttled to 60Hz by sleeping in the main loop. This locks the CPU to roughly its intended speed, 1.789773MHz NTSC. The APU runs at half that.
The SDL audio device samples/outputs at 44,100Hz, so as long as the APU queues up 44,100 samples per second, it works.
But it's not doing so evenly. If PPU runs faster than 60Hz, audio will get skipped, and if slower, audio will pop/have gaps.
Need to probably lock everything to the APU but worried about checking time that often. Can do for some division of 44_100.

Nowhere room debugging:
Do we want to detect every time WarpZoneControl is accessed and log a buffer before and after it?
Or is the problem not with loading WZC but writing it? Good and bad logs match when entering the pipe.
The subroutine that accesses $06D6 is HandlePipeEntry. That's only called by ChkFootMTile->DoFootCheck->ChkCollSize->PlayerBGCollision->PlayerCtrlRoutine.
PlayerCtrlRoutine is called by PlayerInjuryBlink and PlayerDeath, and all three of those are called by GameRoutines engine.
So the normal physics loop checks for pipe entry every so often. So need to find out how HandlePipeEntry determines where to send you,
and what puts you in the room.

Functions that write to WarpZoneControl:
WarpZoneObject<-RunEnemyObjectsCore<-EnemiesAndLoopsCore<-VictoryMode/GameEngine
ScrollLockObject_Warp<-DecodeAreaData<-ProcessAreaData<-

Is ParseRow0e a clue?

I think L_UndergroundArea3 is the data for the coin rooms. Need to verify that it's loaded properly.
It's at 0x2D89 in the ROM, so 0x2D79 without header. Which means it's in PRG ROM, because it's within the first 0x4000,
in the first PRG chunk/vec given to CPU by cartridge. Because it's NROM, that will be mapped starting at $8000,
so its position in memory should be 0x8000 + 0x2D79 = 0xAD79.

L_UndergroundArea3 is indeed at 0xAD79 and correct in both good emulator and mine. So need to detect its use? Verified that
it's not changed, neither is E_UndergroundArea3 which is at $A133. WarpZoneControl is also set properly: 0 for a while, then
1 when running over exit in 2-1 to Warp Zone, then 4 once dropped down into the WarpZone. 0 when going into any coin rooms.

HandlePipeEntry queues VerticalPipeEntry:
         sta GameEngineSubroutine  ;set to run vertical pipe entry routine on next frame
Then it checks WarpZoneControl and branches to rts if :
        lda WarpZoneControl       ;check warp zone control
        beq ExPipeE               ;branch to leave if none found
        [...]
        ExPipeE: rts                       ;leave!!!
So the problem may be in VerticalPipeEntry. Need to hook it. It starts with lda #$01, so looking for lda in immediate mode, which is 0xA9
followed by jsr then followed by a two byte absolute address we don't know, so 0x20 ?? ??, then jsr another function, so same thing,
then ldy #$00, which is 0xA0 0x00... so now we can grep the rom file for its address and compare to good emulator.
    grep -A10 "a9 *01 *20 *.. *.. *20 *.. *.. *a0"
    000031f0  52 07 4c 13 b2 a9 01 20  00 b2 20 93 af a0 00 ad  |R.L.... .. .....|
VerticalPipeEntry is at $31F5 in the ROM, so at $B205 in the running emulator. Now need to confirm that and then log starting there.
No, had to do a full memory dump to find out that it's at $B225... Anyway, can now hook there. But hook was wrong. And hooking for address == $06D6
shows the program counter at 0xB1EF, meaning I was right that the routine's address is 0xB1E5... So my dump was wrong? Or routines move around? Doesn't make sense.
Anyway, hook PC == $B1E5.

Ok, so, comparing logs with the good emulator down the WORKING pipe in 1-1 shows a divergence in behavior based on loading value 0x6E from $0755 into the accumulator, 
and comparing that to 0x50. What's at $0755? Player_Pos_ForScroll. 
*/
