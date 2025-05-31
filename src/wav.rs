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

impl std::error::Error for WavError {}

impl std::fmt::Display for WavError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WavError::IoError(e) => write!(f, "IO error: {}", e),
            WavError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            WavError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
        }
    }
}

impl From<std::io::Error> for WavError {
    fn from(err: std::io::Error) -> Self {
        WavError::IoError(err)
    }
}

pub fn read_wav_file(path: &str) -> Result<(Vec<i16>, u32), WavError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read and validate RIFF header
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    if &buffer != b"RIFF" {
        return Err(WavError::InvalidFormat("Not a RIFF file".to_string()));
    }

    // Skip file size
    reader.read_exact(&mut buffer)?;

    // Read WAVE identifier
    reader.read_exact(&mut buffer)?;
    if &buffer != b"WAVE" {
        return Err(WavError::InvalidFormat("Not a WAVE file".to_string()));
    }

    // Read "fmt " chunk
    reader.read_exact(&mut buffer)?;
    if &buffer != b"fmt " {
        return Err(WavError::InvalidFormat("Expected fmt chunk".to_string()));
    }

    // Read format chunk size
    reader.read_exact(&mut buffer)?;
    let fmt_size = u32::from_le_bytes(buffer);
    if fmt_size != 16 {
        return Err(WavError::UnsupportedFormat(
            "Expected PCM format".to_string(),
        ));
    }

    // Read format type
    let mut format_buffer = [0u8; 2];
    reader.read_exact(&mut format_buffer)?;
    let audio_format = u16::from_le_bytes(format_buffer);
    if audio_format != 1 {
        return Err(WavError::UnsupportedFormat(
            "Only PCM format supported".to_string(),
        ));
    }

    // Read number of channels
    reader.read_exact(&mut format_buffer)?;
    let num_channels = u16::from_le_bytes(format_buffer);

    // Read sample rate
    reader.read_exact(&mut buffer)?;
    let sample_rate = u32::from_le_bytes(buffer);

    // Skip byte rate and block align
    let mut skip_buffer = [0u8; 6];
    reader.read_exact(&mut skip_buffer)?;

    // Read bits per sample
    reader.read_exact(&mut format_buffer)?;
    let bits_per_sample = u16::from_le_bytes(format_buffer);

    // Find data chunk
    loop {
        reader.read_exact(&mut buffer)?;
        if &buffer == b"data" {
            break;
        }
        // Skip other chunks
        reader.read_exact(&mut buffer)?;
        let chunk_size = u32::from_le_bytes(buffer);
        let mut skip_buffer = vec![0u8; chunk_size as usize];
        reader.read_exact(&mut skip_buffer)?;
    }

    // Read data size
    reader.read_exact(&mut buffer)?;
    let data_size = u32::from_le_bytes(buffer);

    // Now read the actual audio data
    let mut decoded_buffer = vec![0u8; data_size as usize];
    reader.read_exact(&mut decoded_buffer)?;

    let mono_samples = downmix_stereo_to_mono(decoded_buffer);

    // Return the mono samples as a vector of i16 along with the sample rate
    Ok((mono_samples, sample_rate))
}

fn downmix_stereo_to_mono(stereo_buffer: Vec<u8>) -> Vec<i16> {
    if stereo_buffer.len() % 4 != 0 {
        panic!("Stereo buffer must have samples in pairs of 16-bit values");
    }

    let mut mono_buffer = Vec::new();
    for i in (0..stereo_buffer.len()).step_by(4) {
        // Convert two bytes to a 16-bit sample for left channel
        let left_sample = i16::from_le_bytes([stereo_buffer[i], stereo_buffer[i + 1]]);
        // Convert two bytes to a 16-bit sample for right channel
        let right_sample = i16::from_le_bytes([stereo_buffer[i + 2], stereo_buffer[i + 3]]);

        // Average the samples, using i32 to prevent overflow
        let mono_sample = ((left_sample as i32 + right_sample as i32) / 2) as i16;
        mono_buffer.push(mono_sample);
    }

    mono_buffer
}
