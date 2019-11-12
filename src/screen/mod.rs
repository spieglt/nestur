extern crate sdl2;

use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SCALE_FACTOR: usize = 4;

type RGBColor = (u8, u8, u8);

pub struct Screen {
	pub canvas: Canvas<Window>,
}

impl Screen {
	pub fn new(context: &Sdl) -> Result<Self, String> {
		let video_subsystem = context.video()?;
		let window = video_subsystem.window("NESTUR", (256 * SCALE_FACTOR) as u32, (240 * SCALE_FACTOR) as u32)
			.position_centered()
			.opengl()
			.build()
			.map_err(|e| e.to_string())?;
		let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
		canvas.set_draw_color(Color::RGB(0, 0, 0));
		canvas.clear();
		canvas.present();
		Ok(Screen{canvas})
	}

	pub fn draw_pixel(&mut self, x: usize, y: usize, color: RGBColor) -> Result<(), String> {
		let (r, g, b) = color;
		self.canvas.set_draw_color(Color::RGB(r, g, b));
		self.canvas.fill_rect(Rect::new((x * SCALE_FACTOR) as i32, (y * SCALE_FACTOR) as i32,
			SCALE_FACTOR as u32, SCALE_FACTOR as u32))?;
		Ok(())
	}
}