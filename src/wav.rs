use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

#[derive(Debug)]
pub struct WavHeader {
    pub sample_rate: u32,
    pub num_channels: u16,
    pub bits_per_sample: u16,
    pub data_size: u32,
}

#[derive(Debug)]
pub enum WavError {
    IoError(std::io::Error),
    InvalidFormat(String),
    UnsupportedFormat(String),
}

impl From<std::io::Error> for WavError {
    fn from(err: std::io::Error) -> Self {
        WavError::IoError(err)
    }
}

pub fn read_wav_file(path: &str) -> Result<WavHeader, WavError> {
    let file = File::open(path);
    let file_result = match file {
        Ok(file) => file,
        Err(e) => panic!("Failed to open file: {}", e),
    };

    let mut reader = BufReader::new(file_result);

    // Read and validate RIFF header
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    if &buffer != b"RIFF" {
        return Err(WavError::InvalidFormat("Not a RIFF file".to_string()));
    }

    // Read WAVE identifier
    // reader.read_exact(&mut buffer)?;
    // if &buffer != b"WAVE" {
    //     return Err(WavError::InvalidFormat("Not a WAVE file".to_string()));
    // }

    // Read format data
    // let mut format_buffer = [0u8; 2];
    // reader.read_exact(&mut format_buffer)?;
    // let audio_format = u16::from_le_bytes(format_buffer);

    // if audio_format != 1 {
    //     return Err(WavError::UnsupportedFormat(
    //         "Only PCM format supported".to_string(),
    //     ));
    // }

    // Read bits per sample
    // reader.read_exact(&mut format_buffer)?;
    // let bits_per_sample = u16::from_le_bytes(format_buffer);

    // if bits_per_sample != 16 {
    //     return Err(WavError::UnsupportedFormat(
    //         "Only 16-bit PCM supported".to_string(),
    //     ));
    // }

    // println!("{:?}", format_buffer);

    // Decoding the file to a buffer...
    let mut decoded_buffer = Vec::new();
    reader.read_to_end(&mut decoded_buffer);

    let downmixed = downmix_stereo_to_mono(decoded_buffer);

    // println!("First 20 bytes: {:?}", downmixed);

    let wave_header = WavHeader {
        bits_per_sample: 0,
        data_size: 0,
        num_channels: 1,
        sample_rate: 1,
    };

    Ok(wave_header)
}

fn downmix_stereo_to_mono(stereo_buffer: Vec<u8>) -> Vec<i16> {
    if stereo_buffer.len() % 2 != 0 {
        panic!("Stereo buffer must have an even number of samples (left + right)");
    }

    let mut mono_buffer = Vec::new();
    for i in (0..stereo_buffer.len()).step_by(2) {
        let left_sample = stereo_buffer[i];
        let right_sample = stereo_buffer[i + 1];
        let mono_sample = (left_sample + right_sample) as f32 / 2.0; // W're f32 for better precision
        mono_buffer.push(mono_sample as i16);
    }

    println!("MONO: {:?}", mono_buffer);

    mono_buffer
}
