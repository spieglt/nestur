/*
First order low-pass filter equation: H(s)=1/(τs+1). H(s) is output, 

low pass filter:
y = (1 - gamma) * y + gamma * x

high pass filter:
y[i] := gamma * y[i−1] + gamma * (x[i] − x[i−1])

fc = 44100 = sample frequency
Ts = 1/44100 = sample period
fc = 14000 = cutoff frequency
gamma = 1 - (e ^ (-2pi * fc / fs))
*/

use std::f32::consts::{E, PI};

extern crate sdl2;

use std::sync::{Arc, Mutex};
use sdl2::Sdl;
use sdl2::audio::{AudioCallback, AudioSpecDesired};

const APU_SAMPLE_RATE: f32 = 894_886.5;
const SDL_SAMPLE_RATE: i32 = 44_100;
// Video runs at 60Hz, so console is clocked by doing enough work to create one frame of video, then sending the video and audio to their respective SDL
// devices and then sleeping. So the audio device is set to play 44,100 samples per second, and grab them in 60 intervals over the course of that second.
const SAMPLES_PER_FRAME: u16 = SDL_SAMPLE_RATE as u16/60;

// struct LowPass {
//     cutoff_freq: f32,
//     gamma: f32,
//     previous_input: f32,
//     previous_out: f32,
// }

// struct HighPass {
//     cutoff_freq: f32,
//     gamma: f32,
//     previous_input: f32,
//     previous_out: f32,
// }

// impl HighPass {
//     fn filter(&self, sample: f32) -> f32 
// }

fn get_gamma(cutoff_freq: f32) -> f32 {
    1. - (E.powf(-2. * PI * cutoff_freq / (SDL_SAMPLE_RATE as f32)))
}

pub struct ApuSampler {
    // This buffer receives all of the raw audio produced by the APU.
    // The callback will take what it needs when it needs it and truncate the buffer for smooth audio output.
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_ratio: f32,

    prev_input_90Hz: f32,
    prev_output_90Hz: f32,
    gamma_90Hz: f32,

    prev_input_440Hz: f32,
    prev_output_440Hz: f32,
    gamma_440Hz: f32,

    prev_input_14kHz: f32,
    prev_output_14kHz: f32,
    gamma_14kHz: f32,
}

impl ApuSampler {

    fn high_pass_90Hz(&self, sample: f32) -> f32 {
        // y[i] := α × y[i−1] + α × (x[i] − x[i−1])
        (self.gamma_90Hz * self.prev_output_90Hz) + (sample - self.prev_input_90Hz)
    }

    fn high_pass_440Hz(&self, sample: f32) -> f32 {
        (self.gamma_440Hz * self.prev_output_440Hz) + (sample - self.prev_input_440Hz)
    }

    fn low_pass_14kHz(&self, sample: f32) -> f32 {
        ((1. - self.gamma_14kHz) * self.prev_output_14kHz) + (self.gamma_14kHz * sample)
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

                    let filtered_90Hz = self.high_pass_90Hz(sample);
                    self.prev_input_90Hz = sample;
                    self.prev_output_90Hz = filtered_90Hz;
                    // *x = filtered_90Hz;

                    let filtered_440Hz = self.high_pass_440Hz(filtered_90Hz);
                    self.prev_input_440Hz = filtered_90Hz;
                    self.prev_output_440Hz = filtered_440Hz;
                    // *x = filtered_440Hz;

                    let filtered_14kHz = self.low_pass_14kHz(filtered_440Hz);
                    self.prev_input_14kHz = filtered_440Hz;
                    self.prev_output_14kHz = filtered_14kHz;
                    *x = filtered_14kHz;

                    // *x = sample;
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

            prev_input_90Hz: 0.,
            prev_output_90Hz: 0.,
            gamma_90Hz: 1.-get_gamma(90.),
            prev_input_440Hz: 0.,
            prev_output_440Hz: 0.,
            gamma_440Hz: 1.-get_gamma(440.),
            prev_input_14kHz: 0.,
            prev_output_14kHz: 0.,
            gamma_14kHz: get_gamma(14_000.),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::get_gamma;
    #[test]
    fn show_gamma_values() {
        for i in [0, 100, 1000, 10000, 100000].iter() {
            println!("gamma for cutoff frequency {}: {}", i, get_gamma(*i as f32));
        }
    }
}
