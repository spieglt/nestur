# nestur

This is an NES emulator and a work in progress. The CPU and PPU work, though there are still at least a couple bugs. I've mostly tested on Donkey Kong and Super Mario Bros. so far. There are plenty of full-featured emulators out there; this is primarily an educational project but I do want it to run well.

- One dependency (SDL)

- One line of `unsafe` (`std::mem::transmute::<u8>() -> i8`)

- NTSC timing

<img src="pics/smb.png" width=600>

<sup>(Warning: this pipe currently takes you to an empty room, it's not the only one, and I don't know why.)</sup>

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
The code aims to follow the explanations from https://wiki.nesdev.com/w/index.php/NES_reference_guide where possible, especially in the PPU, and the comments quote from it often.

Thanks to Michael Fogleman's https://github.com/fogleman/nes for getting me unstuck at several points.

## To do:

- More mappers (only NROM/mapper 0 implemented so far)

- DMC audio channel, high- and low-pass filters, APU cleanup/timing fix

- Save/load functionality and battery-backed RAM solution

- Player 2 controller?
