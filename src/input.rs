use sdl2::keyboard::Scancode;
use std::collections::HashSet;

pub fn poll_buttons(strobe: &u8, event_pump: &sdl2::EventPump) -> Option<u8> {
    if *strobe & 1 == 1 {
        let mut button_states = 0;
        let pressed_keys: HashSet<Scancode> =
            event_pump.keyboard_state().pressed_scancodes().collect();
        for key in pressed_keys.iter() {
            match key {
                Scancode::D => button_states |= 1 << 0,      // A
                Scancode::F => button_states |= 1 << 1,      // B
                Scancode::RShift => button_states |= 1 << 2, // Select
                Scancode::Return => button_states |= 1 << 3, // Start
                Scancode::Up => button_states |= 1 << 4,     // Up
                Scancode::Down => button_states |= 1 << 5,   // Down
                Scancode::Left => button_states |= 1 << 6,   // Left
                Scancode::Right => button_states |= 1 << 7,  // Right
                _ => (),
            }
        }
        Some(button_states)
    } else {
        None
    }
}
