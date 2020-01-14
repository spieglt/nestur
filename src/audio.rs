extern crate sdl2;

use std::sync::{Arc, Mutex};
use sdl2::Sdl;
use sdl2::audio::{AudioCallback, AudioSpecDesired};

const APU_SAMPLE_RATE: f32 = 894_886.5;
const SDL_SAMPLE_RATE: i32 = 44_100;
// Video runs at 60Hz, so console is clocked by doing enough work to create one frame of video, then sending the video and audio to their respective SDL
// devices and then sleeping. So the audio device is set to play 44,100 samples per second, and grab them in 60 intervals over the course of that second.
const SAMPLES_PER_FRAME: u16 = SDL_SAMPLE_RATE as u16/60;

pub struct ApuSampler {
    // This buffer receives all of the raw audio produced by the APU.
    // The callback will take what it needs when it needs it and truncate the buffer for smooth audio output.
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_ratio: f32,
}

impl AudioCallback for ApuSampler {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut b = self.buffer.lock().unwrap();
        // if we have data in the buffer
        if b.len() > 0 {
            // copy samples at the appropriate interval from the raw APU buffer to the output device
            for (i, x) in out.iter_mut().enumerate() {
                let sample_idx = ((i as f32) * self.sample_ratio) as usize;
                if sample_idx < b.len() {
                    *x = b[sample_idx];
                }
            }
            let l = b.len();
            // how many samples we would hope to have consumed
            let target = (SAMPLES_PER_FRAME as f32 * self.sample_ratio) as usize;
            // if we had more data than we needed, truncate what we used and keep the rest in case
            // the callback is called twice before the buffer is refilled,
            // but raise the ratio so we get closer to the speed at which the APU is working.
            // if we didn't have enough, decrease the ratio so we take more samples from the APU
            if l > target {
                *b = b.split_off(target);
                self.sample_ratio += 0.005;
                // println!("raised ratio to {}", self.sample_ratio);
            } else {
                b.clear();
                self.sample_ratio -= 0.05;
                // println!("lowered ratio to {}", self.sample_ratio);
            }
        } else {
            println!("buffer empty!"); // happens when the callback fires twice between video frames
        }
    }
}

pub fn initialize(sdl_context: &Sdl, buffer: Arc<Mutex<Vec<f32>>>) 
    -> Result<sdl2::audio::AudioDevice<ApuSampler>, String> 
{
    let audio_subsystem = sdl_context.audio()?;
    let desired_spec = AudioSpecDesired {
        freq: Some(SDL_SAMPLE_RATE),
        channels: Some(1), // mono
        samples: Some(SAMPLES_PER_FRAME)
    };
    audio_subsystem.open_playback(None, &desired_spec, |spec| {
        println!("{:?}", spec);
        ApuSampler{buffer, sample_ratio: APU_SAMPLE_RATE / (SDL_SAMPLE_RATE as f32)}
    })
}
