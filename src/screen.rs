extern crate sdl2;

use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

pub const SCALE_FACTOR: usize = 1;
//~ const BYTES_IN_COL: usize = SCALE_FACTOR * 3;   // 3 bytes per pixel in RGB24. This represents a thick, SCALE_FACTOR-pixel-wide column.
//~ const BYTES_IN_ROW: usize = BYTES_IN_COL * 256; // 256 = screen width in NES pixels. This represents a thin, one-SDL-pixel-tall row.

const BYTES_IN_ROW: usize = 3 * 256;

type RGBColor = [u8; 3];

pub fn init_window(context: &Sdl) -> Result<(Canvas<Window>, TextureCreator<WindowContext>), String> {
    let video_subsystem = context.video()?;
    let window = video_subsystem.window("NESTUR", (256 * SCALE_FACTOR) as u32, (240 * SCALE_FACTOR) as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    Ok((canvas, texture_creator))
}

pub fn draw_pixel(buffer: &mut Vec<u8>, x: usize, y: usize, color: RGBColor) {
    let offset = (y * BYTES_IN_ROW) + (x * 3);
    buffer.splice(offset..(offset+3), color);
}

pub fn draw_to_window(texture: &mut Texture, canvas: &mut Canvas<Window>, buffer: &Vec<u8>) -> Result<(), String> {
    texture.update(None, buffer, 256*3*SCALE_FACTOR)
        .map_err(|e| e.to_string())?;
    canvas.copy(&texture, None, None)?;
    canvas.present();
    Ok(())
}
