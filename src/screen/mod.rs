extern crate sdl2;

use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

pub const SCALE_FACTOR: usize = 3;
const BYTES_IN_COL: usize = SCALE_FACTOR * 3;   // 3 bytes per pixel in RGB24. This represents a thick, SCALE_FACTOR-pixel-wide column.
const BYTES_IN_ROW: usize = BYTES_IN_COL * 256; // 256 = screen width in NES pixels. This represents a thin, one-SDL-pixel-tall row.

type RGBColor = (u8, u8, u8);

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
	let (r, g, b) = color;
	let nes_y_offset = y * BYTES_IN_ROW * SCALE_FACTOR; // find offset for thick, SCALE_FACTOR-pixel tall row
	for sdl_row_num in 0..SCALE_FACTOR { // looping over one-pixel tall rows up to SCALE_FACTOR
		let row_offset = nes_y_offset + (sdl_row_num * BYTES_IN_ROW); // row_offset is the offset within buffer of the thin row we're on
		let nes_x_offset = x * BYTES_IN_COL; // find horizontal offset within row (in byte terms) of NES x-coordinate
		for sdl_col_num in 0..SCALE_FACTOR { // for pixels up to SCALE_FACTOR, moving horizontally
			let col_offset = nes_x_offset + (sdl_col_num * 3); // skip 3 bytes at a time, R/G/B for each pixel
			let offset = row_offset + col_offset;
			buffer[offset + 0] = r;
			buffer[offset + 1] = g;
			buffer[offset + 2] = b;
		}
	}
}

pub fn draw_to_window(texture: &mut Texture, canvas: &mut Canvas<Window>, buffer: &Vec<u8>) -> Result<(), String> {
	texture.update(None, buffer, 256*3*SCALE_FACTOR)
		.map_err(|e| e.to_string())?;
    canvas.copy(&texture, None, None)?;
	canvas.present();
	Ok(())
}
