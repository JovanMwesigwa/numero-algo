//  Main fingerprinting algorithm
// Normalization
// Inside this function (in fingerprint.go), the raw int16 samples are converted into float64 values scaled between –1 and 1:

const TARGET_SAMPLE_RATE: u32 = 11025; // Downsampled rate.
const FILTER_TAPS: usize = 101; // Samples per frame
const FRAME_SIZE: usize = 1024; // Samples per frame
const HOP_SIZE: usize = 512; // Hop size for overlapping frames
const NUM_BANDS: usize = 6; // Number of frequency bands for peak detection
const TARGET_ZONE_FRAMES: usize = 20; // Maximum frame difference for pairing peaks

pub fn finger_print(samples: &[i16], sample_rate: u32) -> Result<Vec<f64>, String> {
    // Check if samples are empty or smaple rate is lower that the target sample rate
    if samples.is_empty() || sample_rate < TARGET_SAMPLE_RATE {
        return Err("Invalid input: samples are empty or sample rate is too low".to_string());
    }

    // The the raw int16 samples are converted into float64 values scaled between –1 and 1:

    // Find the maximum absolute value of the i16 samples
    let max_abs = samples.iter().map(|s| s.abs()).max().unwrap_or(1);

    // Convert the raw i16 samples to f64 values scaled between -1.0 and 1.0
    let normalized_samples: Vec<f64> = samples
        .iter()
        .map(|&s| (s as f64) / max_abs as f64)
        .collect();

    // Return downsampled samples
    Ok(normalized_samples)
}
