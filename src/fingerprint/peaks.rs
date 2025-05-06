//  Detecting Peaks in the Spectrogram

// With the spectrogram in hand, the next task is to pinpoint the most significant frequencies:
// Dividing into Bands: Each frame's frequency bins are divided into a fixed number of bands (in our case, 6). This ensures that we capture distinct frequency components across the spectrum.
// Selecting the Maximum: Within each band, the algorithm finds the frequency bin with the highest magnitude (the peak) and records its frame index, frequency bin, and amplitude.

#[derive(Debug, Clone, Copy)]
pub struct Peak {
    pub frame_index: usize,
    pub freq_bin: usize,
    pub magnitude: f64,
}

// DetectPeaks finds the strongest frequency peaks in each band of the spectrogram
pub fn detect_peaks(
    spectrogram: &[Vec<f64>],
    num_of_bands: usize,
    threshold_multiplier: f64,
) -> Vec<Peak> {
    let mut peaks = Vec::new();

    for (i, frame) in spectrogram.iter().enumerate() {
        let num_bins = frame.len();
        let band_size = num_bins / num_of_bands;

        for band in 0..num_of_bands {
            let start = band * band_size;
            let mut end = start + band_size;
            if band == num_of_bands - 1 {
                end = num_bins;
            }

            // Calculate average magnitude in the current band
            let mut sum_magnitude = 0.0;
            for j in start..end {
                sum_magnitude += frame[j];
            }
            let average_magnitude = sum_magnitude / (end - start) as f64;
            let local_threshold = average_magnitude * threshold_multiplier; // Tunable parameter

            let mut max_val = -1.0;
            let mut max_bin = None;

            for j in start..end {
                if frame[j] > max_val && frame[j] > local_threshold {
                    // Apply local threshold
                    max_val = frame[j];
                    max_bin = Some(j);
                }
            }

            if let Some(bin) = max_bin {
                peaks.push(Peak {
                    frame_index: i,
                    freq_bin: bin,
                    magnitude: max_val,
                });
            }
        }
    }

    peaks
}
