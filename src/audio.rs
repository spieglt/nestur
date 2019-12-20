extern crate sdl2;

use sdl2::audio::{AudioCallback, AudioSpecDesired};

pub struct Speaker {
    buffer: [f32; 4096*4],
    head: usize,
}

impl AudioCallback for Speaker {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for (i, x) in out.iter_mut().enumerate() {
            *x = self.buffer[i+self.head]; // get data from apu
        }
        self.head = (self.head + 4096) % (4096*4)
    }
}

impl Speaker {
    pub fn append(&mut self, sample: f32) {
        self.buffer[self.head] = sample;
        self.head = (self.head + 1) % (4096*4);
    }
}

pub fn initialize(context: &sdl2::Sdl) -> Result<sdl2::audio::AudioDevice<Speaker>, String> {
    let audio_subsystem = context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),   // mono
        samples: Some(4096),       // default sample size
    };

    audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // Show obtained AudioSpec
        println!("{:?}", spec);

        // initialize the audio callback
        Speaker{buffer: [0_f32; 4096*4], head: 0}
    })
}