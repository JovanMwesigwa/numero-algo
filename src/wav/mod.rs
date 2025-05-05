// This is the audio file reader that supports both WAV and MP3 files
// Returns mono samples as a vector of i16 along with the sample rate.
// Ensures consistent mono, 16-bit format for fingerprinting.

use crate::utils;
use rodio::{Decoder, Source};
use std::fs::File;
use std::io::{self, BufReader};

pub fn read_audio_file(path: &str) -> Result<(Vec<i16>, u32), io::Error> {
    // Open the file
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Create decoder (supports both WAV and MP3)
    let decoder = Decoder::new(reader).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to decode audio: {}", e),
        )
    })?;

    // Get the sample rate and channels
    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();

    // Convert to mono i16 samples
    let samples: Vec<i16> = decoder.convert_samples().collect();
    let num_frames = samples.len() / channels as usize;

    let mut mono_samples: Vec<i16> = Vec::with_capacity(num_frames);

    // Downmix stereo to mono by averaging the channels
    for i in 0..num_frames {
        let mut sum: i32 = 0;
        for j in 0..channels as usize {
            let sample = samples[i * channels as usize + j] as i32;
            sum += sample;
        }
        let mono_sample = (sum / channels as i32) as i16;
        mono_samples.push(mono_sample);
    }

    // Validate the audio format
    if let Err(e) = utils::validate_audio_format(&mono_samples, sample_rate) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, e));
    }

    Ok((mono_samples, sample_rate))
}
