//  Main fingerprinting algorithm
// Normalization
// Inside this function (in fingerprint.go), the raw int16 samples are converted into float64 values scaled between –1 and 1:

use crate::dsp::filter::{apply_fir_filter, generate_low_pass_kernel};
use crate::dsp::viz::plot_spectrogram;
use crate::fingerprint::hash::hash_fingerprint;
use crate::fingerprint::peaks::detect_peaks;
use crate::fingerprint::spectogram::compute_spectrogram;
use crate::fingerprint::utils::{frame_signal, hamming_window};

const TARGET_SAMPLE_RATE: u32 = 11025; // Downsampled rate.
const FILTER_TAPS: usize = 101; // Samples per frame
const FRAME_SIZE: usize = 1024; // Samples per frame
const HOP_SIZE: usize = 512; // Hop size for overlapping frames
const NUM_BANDS: usize = 6; // Number of frequency bands for peak detection
const TARGET_ZONE_FRAMES: usize = 20; // Maximum frame difference for pairing peaks
const THRESHOLD_MULTIPLIER: f64 = 0.1; // Threshold multiplier for peak detection

pub fn finger_print(samples: &[i16], sample_rate: u32) -> Result<Vec<u32>, String> {
    // Check if samples are empty or sample rate is lower that the target sample rate
    if samples.is_empty() || sample_rate < TARGET_SAMPLE_RATE {
        return Err("Invalid input: samples are empty or sample rate is too low".to_string());
    }

    // Find the maximum absolute value of the samples, handling i16::MIN specially
    let max_abs = samples
        .iter()
        .map(|&s| {
            if s == i16::MIN {
                32768_i32
            } else {
                s.abs() as i32
            }
        })
        .max()
        .unwrap_or(1) as f64;

    // Convert the raw i16 samples to f64 values scaled between -1.0 and 1.0
    let normalized_samples: Vec<f64> = samples
        .iter()
        .map(|&s| {
            if s == i16::MIN {
                -1.0 // i16::MIN maps to -1.0
            } else {
                (s as f64) / max_abs
            }
        })
        .collect();

    // Get the cutoff frequency for downsampling
    let cutoff_freq = TARGET_SAMPLE_RATE as f64 / 2.0;

    // Generate the low-pass filter
    let kernel = generate_low_pass_kernel(cutoff_freq, sample_rate, FILTER_TAPS);

    // Apply the filter
    let filtered = apply_fir_filter(&normalized_samples, &kernel);

    // Downsample the filtered signal
    let decimation_factor = sample_rate as f64 / TARGET_SAMPLE_RATE as f64;
    let mut downsampled = vec![0.0; filtered.len() / decimation_factor as usize];
    for i in 0..downsampled.len() {
        downsampled[i] = filtered[i * decimation_factor as usize];
    }

    // Framing the Signal
    let frames = frame_signal(&downsampled, FRAME_SIZE, HOP_SIZE);

    // Windowing
    let window = hamming_window(FRAME_SIZE);

    // Compute the spectrogram
    let spectrogram = compute_spectrogram(
        frames
            .iter()
            .map(|f| f.iter().map(|&x| x as f32).collect())
            .collect(),
        window.iter().map(|&x| x as f32).collect(),
    );

    // Detect Peaks
    let peaks = detect_peaks(&spectrogram, NUM_BANDS, THRESHOLD_MULTIPLIER);

    if let Err(e) = plot_spectrogram(
        &spectrogram,
        TARGET_SAMPLE_RATE,
        FRAME_SIZE,
        HOP_SIZE,
        &peaks,
        "spectrogram.png",
    ) {
        eprintln!("Warning: Failed to plot spectrogram: {}", e);
    }

    // Generate and return the fingerprint hashes
    let hashes = hash_fingerprint(&peaks, TARGET_ZONE_FRAMES);
    Ok(hashes)
}
