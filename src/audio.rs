use sdl2::audio::{AudioCallback, AudioSpecDesired};

use std::time::Duration;
use std::time::SystemTime;

fn gen_wave(bytes_to_write: i32) -> Vec<i16> {
    // Generate a square wave
    let tone_volume = 1_000i16;
    let period = 48_000 / 256;
    let sample_count = bytes_to_write;
    let mut result = Vec::new();

    for x in 0..sample_count {
        result.push(
            if (x / period) % 2 == 0 {
                tone_volume
            }
            else {
                -tone_volume
            }
        );
    }
    result
}

pub fn main(audio_subsystem: sdl2::AudioSubsystem) -> Result<(), String> {
    let desired_spec = AudioSpecDesired {
        freq: Some(48_000),
        channels: Some(2),
        // mono  -
        samples: Some(4)
        // default sample size
    };

    let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec)?;

    let target_bytes = 48_000 * 2;
    let wave = gen_wave(target_bytes);
    device.queue(&wave);
    // Start playback
    device.resume();

    // Play for 2 seconds
    std::thread::sleep(Duration::from_millis(2_000));

    // Device is automatically closed when dropped

    Ok(())
}


struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 { self.volume } else { -self.volume };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
        println!("{:?}", out.len());
    }
}

pub fn main2(audio_subsystem: sdl2::AudioSubsystem) -> Result<(), String> {
    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),  // mono
        samples: None       // default sample size
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // Show obtained AudioSpec
        println!("{:?}", spec);

        // initialize the audio callback
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.05
        }
    })?;

    // Start playback
    device.resume();

    // Play for 2 seconds
    std::thread::sleep(Duration::from_millis(200_000));

    // Device is automatically closed when dropped

    Ok(())
}
