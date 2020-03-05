use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use sdl2::Sdl;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::f32::consts::PI;

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

    prev_input_90_hz: f32,
    prev_output_90_hz: f32,
    gamma_90_hz: f32,

    prev_input_440_hz: f32,
    prev_output_440_hz: f32,
    gamma_440_hz: f32,

    prev_input_14_khz: f32,
    prev_output_14_khz: f32,
    gamma_14_khz: f32,
}

impl ApuSampler {
    fn high_pass_90_hz(&self, sample: f32) -> f32 {
        // y[i] := α × y[i−1] + α × (x[i] − x[i−1])
        (self.gamma_90_hz * self.prev_output_90_hz) + (sample - self.prev_input_90_hz)
    }

    fn high_pass_440_hz(&self, sample: f32) -> f32 {
        (self.gamma_440_hz * self.prev_output_440_hz) + (sample - self.prev_input_440_hz)
    }

    fn low_pass_14_khz(&self, sample: f32) -> f32 {
        ((1. - self.gamma_14_khz) * self.prev_output_14_khz) + (self.gamma_14_khz * sample)
    }

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
                    let sample = b[sample_idx];

                    let filtered_90_hz = self.high_pass_90_hz(sample);
                    self.prev_input_90_hz = sample;
                    self.prev_output_90_hz = filtered_90_hz;

                    let filtered_440_hz = self.high_pass_440_hz(filtered_90_hz);
                    self.prev_input_440_hz = filtered_90_hz;
                    self.prev_output_440_hz = filtered_440_hz;

                    let filtered_14_khz = self.low_pass_14_khz(filtered_440_hz);
                    self.prev_input_14_khz = filtered_440_hz;
                    self.prev_output_14_khz = filtered_14_khz;
                    *x = filtered_14_khz;
                }
            }
            let l = b.len();
            let target = (SAMPLES_PER_FRAME as f32 * self.sample_ratio) as usize;
            if l > target {
                *b = b.split_off(target);
            }
        } else {
            // println!("buffer empty!"); // happens when the callback fires twice between video frames
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
    audio_subsystem.open_playback(None, &desired_spec, |_spec| {
        // println!("{:?}", _spec);
        ApuSampler{
            buffer,
            sample_ratio: APU_SAMPLE_RATE / (SDL_SAMPLE_RATE as f32),
            prev_input_90_hz: 0.,
            prev_output_90_hz: 0.,
            gamma_90_hz: high_pass_coefficient(90.),
            prev_input_440_hz: 0.,
            prev_output_440_hz: 0.,
            gamma_440_hz: high_pass_coefficient(440.),
            prev_input_14_khz: 0.,
            prev_output_14_khz: 0.,
            gamma_14_khz: low_pass_coefficient(14_000.),
        }
    })
}

fn low_pass_coefficient(cutoff_freq: f32) -> f32 {
    (2.*PI*cutoff_freq/SDL_SAMPLE_RATE as f32) / ((2.*PI*cutoff_freq/SDL_SAMPLE_RATE as f32) + 1.)
}

fn high_pass_coefficient(cutoff_freq: f32) -> f32 {
    1. / ((2.*PI*cutoff_freq/SDL_SAMPLE_RATE as f32) + 1.)
}

/*

https://en.wikipedia.org/wiki/High-pass_filter
https://en.wikipedia.org/wiki/Low-pass_filter

low pass filter:
y = (1 - gamma) * y + gamma * x

high pass filter:
y[i] := gamma * y[i−1] + gamma * (x[i] − x[i−1])

*/
