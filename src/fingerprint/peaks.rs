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
pub fn detect_peaks(spectrogram: &[Vec<f64>], num_of_bands: usize) -> Vec<Peak> {
    let mut peaks = Vec::new();

    // First find the global maximum magnitude for thresholding
    let max_magnitude = spectrogram
        .iter()
        .flat_map(|frame| frame.iter())
        .fold(0.0_f64, |max, &x| max.max(x));

    // Set threshold to keep only strong peaks (e.g. top 10% of max magnitude)
    let threshold = max_magnitude * 0.1;

    for (i, frame) in spectrogram.iter().enumerate() {
        let num_bins = frame.len();
        let band_size = num_bins / num_of_bands;

        for band in 0..num_of_bands {
            let start = band * band_size;
            let mut end = start + band_size;
            if band == num_of_bands - 1 {
                end = num_bins;
            }

            let mut max_val = -1.0;
            let mut max_bin = None;

            for j in start..end {
                if frame[j] > max_val {
                    max_val = frame[j];
                    max_bin = Some(j);
                }
            }

            // Only keep peaks above the threshold
            if let Some(bin) = max_bin {
                if max_val > threshold {
                    peaks.push(Peak {
                        frame_index: i,
                        freq_bin: bin,
                        magnitude: max_val,
                    });
                }
            }
        }
    }

    peaks
}
