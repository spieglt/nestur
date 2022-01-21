extern crate sdl2;

use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

pub const SCALE_FACTOR: usize = 2;

pub fn init_window(context: &Sdl) -> Result<(Canvas<Window>, TextureCreator<WindowContext>), String> {
    let video_subsystem = context.video()?;
    let window = video_subsystem.window("NESTUR", (256 * SCALE_FACTOR) as u32, (240 * SCALE_FACTOR) as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_logical_size(256, 240)
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    Ok((canvas, texture_creator))
}
/*
pub fn draw_pixel(buffer: &mut Vec<u8>, x: usize, y: usize, color: RGBColor) {
    let offset = (y * 3 * 256) + (x * 3);
    // buffer.splice(offset..(offset+3), color);
    buffer[offset] = color[0];
    buffer[offset+1] = color[1];
    buffer[offset+2] = color[2];
}
*/
pub fn draw_to_window(texture: &mut Texture, canvas: &mut Canvas<Window>, buffer: &Vec<u8>) -> Result<(), String> {
    texture.update(None, &buffer, 256*3)
        .map_err(|e| e.to_string())?;
    canvas.copy(&texture, None, None)?;
    canvas.present();
    Ok(())
}
