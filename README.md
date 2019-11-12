# nestur
The NES you left outside in the rain but let dry and still kind of works.

This is an NES emulator and a work in progress. The CPU and PPU work, though there are still at least a couple bugs. I've mostly tested on Donkey Kong and Super Mario Bros. so far. There are plenty of full-featured emulators out there; this is primarily an educational project but I do want it to run well.

- One dependency (SDL)

- One line of `unsafe` (`std::mem::transmute::<u8>() -> i8`)

- No heap allocation

- NTSC timing

## Controls:
```
 Button  |   Key
___________________
|   A    |    D   |
|   B    |    F   |
| Start  |  Enter |
| Select | L-Shift|
|   Up   |   Up   |
|  Down  |  Down  |
|  Left  |  Left  |
|  Right |  Right |
-------------------
```
The code aims to follow the explanations from https://wiki.nesdev.com/w/index.php/NES_reference_guide where possible, especially in the PPU, and the comments quote from it often.

Thanks to Michael Fogleman's https://github.com/fogleman/nes for getting me unstuck at several points.

## To do:

- More mappers (only NROM/mapper 0 implemented so far)

- Audio

- Player 2 controller?

- Sprite bug when Goomba smashed in Mario

- Figure out performance issue on Windows
