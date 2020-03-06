# nestur

Nestur is an NES emulator. There are plenty of full-featured emulators out there; this is primarily an educational project but it is usable. There may still be many bugs, but I'm probably not aware of them so please submit issues.
- no use of `unsafe`
- NTSC timing
- supports mappers 0-4 which cover ~85% of [games](http://tuxnes.sourceforge.net/nesmapper.txt)

<img src="pics/smb.png" width=250> <img src="pics/zelda_dungeon.png" width=250> <img src="pics/kirby.png" width=250> <img src="pics/dk.png" width=250> <img src="pics/smb3.png" width=250> <img src="pics/excitebike.png" width=250>

The code aims to follow the explanations from the [NES dev wiki](https://wiki.nesdev.com/w/index.php/NES_reference_guide) where possible, especially in the PPU, and the comments quote from it often. Thanks to everyone who contributes to that wiki/forum, and to Michael Fogleman's [NES](https://github.com/fogleman/nes) and Scott Ferguson's [Fergulator](https://github.com/scottferg/Fergulator) for getting me unstuck at several points.

## Controls
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

F2: reset console
F5: save game state
F9: load most recent save state
```
If the game is called `mygame.nes`, the save state files will be called `mygame-#.dat`. To load any previous save state, drag and drop a `.dat` file onto the window.

## Use

Double-click or run the executable from a terminal by itself to launch with instructions. Then click Ok and drag a (iNES/`.nes`) ROM file onto the window. Or, drag and drop a ROM file onto the executable to run it directly, or use the path to the ROM file as the first argument to the terminal command.

If the game uses battery-backed RAM (if it can save data when the console is turned off), a save file like `rom_filename.sav` will be created in the same folder as the ROM when the program is exited. When Nestur is run again, it will look for a file matching the ROM name, with a `.sav` extension instead of `.nes`.

## Compilation

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Have a C compiler
    - Linux: `sudo apt install build-essential`
    - Mac: [XCode](https://apps.apple.com/us/app/xcode/id497799835)
    - Windows: install the [Visual Studio Build Tools](https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16) (or [Visual Studio](https://docs.microsoft.com/en-us/cpp/build/vscpp-step-0-installation?view=vs-2019) with the "Desktop development with C++" workload).
3. Install CMake
    - Linux: `sudo apt install cmake`
    - Mac: install [Homebrew](https://brew.sh/) and run `brew install cmake`
    - [Windows](https://cmake.org/download/)
4. `cd nestur/ && cargo build --release` (be sure to build/run with the release flag or it will run very slowly)
5. The `nestur` executable or `nestur.exe` will be in `nestur/target/release`.

## To do

- DMC audio channel

- support other controllers?

## Known problem games

- None currently, please report any issues


Please also check out [Cloaker](https://github.com/spieglt/cloaker) and [Flying Carpet](https://github.com/spieglt/flyingcarpet)!
