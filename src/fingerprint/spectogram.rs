// Parallel Processing: spectogram.rs's computeSpectrogram function applies the FFT on each frame concurrently using goroutines, speeding up the process.

use crate::dsp::fft::compute_fft;
use rayon::prelude::*;

pub fn compute_spectrogram(frames: Vec<Vec<f32>>, window: Vec<f32>) -> Vec<Vec<f64>> {
    // Process frames in parallel using rayon
    frames
        .par_iter()
        .map(|frame| {
            // Apply window function and compute FFT
            let windowed_frame: Vec<f32> = frame
                .iter()
                .zip(window.iter())
                .map(|(&sample, &window_val)| sample * window_val)
                .collect();

            compute_fft(windowed_frame)
        })
        .collect()
}
