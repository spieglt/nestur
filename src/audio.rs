extern crate sdl2;

use sdl2::audio::{AudioCallback, AudioSpecDesired};

pub struct Speaker {
    buffer: [u8; 4096],
}

impl AudioCallback for Speaker {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for (i, x) in out.iter_mut().enumerate() {
            *x = self.buffer[i]; // get data from apu
        }
    }
}

pub fn initialize(context: &sdl2::Sdl) -> Result<sdl2::audio::AudioDevice<SquareWave>, String> {
    let audio_subsystem = context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),   // mono
        samples: 4096,       // default sample size
    };

    audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // Show obtained AudioSpec
        println!("{:?}", spec);

        // initialize the audio callback
        Speaker{buffer: [0; 4096]}
    })
}