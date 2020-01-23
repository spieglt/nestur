# nestur

Nestur is an NES emulator and a work in progress. There are still some minor bugs and the audio is kind of scratchy. I've mostly tested on Donkey Kong, Super Mario Bros., and Zelda so far. There are plenty of full-featured emulators out there; this is primarily an educational project but I do want it to run well. SDL2 is the only dependency, it's NTSC timing, and contains no `unsafe` code.

<img src="pics/smb.png" width=350>  <img src="pics/zelda_dungeon.png" width=350>

## Controls:
```
 Button  |   Key
___________________
|   A    |    D   |
|   B    |    F   |
| Start  |  Enter |
| Select | R-Shift|
|   Up   |   Up   |
|  Down  |  Down  |
|  Left  |  Left  |
|  Right |  Right |
-------------------
```
The code aims to follow the explanations from the [NES dev wiki](https://wiki.nesdev.com/w/index.php/NES_reference_guide) where possible, especially in the PPU, and the comments quote from it often. Thanks to everyone who contributes to that wiki/forum, and to Michael Fogleman's [NES](https://github.com/fogleman/nes) and Scott Ferguson's [Fergulator](https://github.com/scottferg/Fergulator) for getting me unstuck at several points.

## Compilation and Use

1. Install Rust: https://www.rust-lang.org/tools/install
2. Configure SDL2 for your platform:
    - Windows: `SDL2.dll` is already in the repo so you don't have to do anything.
    - macOS: Install [Homebrew](https://brew.sh/) and run `brew install sdl2`
    - Linux: `sudo apt-get install libsdl2-dev` (or whatever your package manager is)
3. `cd nestur/ && cargo build --release`
4. The `nestur` executable or `nestur.exe` will be in `nestur/target/release`.
5. Run with `$ ./nestur path/to/rom_filename.nes` or `> nestur.exe path\to\rom_filename.nes`.
6. If the game uses battery-backed RAM (if it can save data when turned off), a save file like `rom_filename.sav` will be created in the same folder as the ROM when the program is exited. When Nestur is run again, it will look for a file matching the ROM name, with a `.sav` extension instead of `.nes`.

## To do:

- Fix any bugs with mappers 0-3 and improve mapper 4 (MMC3)

- DMC audio channel, high- and low-pass filters

- Save state/load functionality

- Player 2 controller?


Please also check out [Cloaker](https://github.com/spieglt/cloaker) and [Flying Carpet](https://github.com/spieglt/flyingcarpet)!
