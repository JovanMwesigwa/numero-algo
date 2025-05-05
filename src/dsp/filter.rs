// GenerateLowPassKernel creates a low-pass FIR filter kernel.

use std::f64::consts::PI;

// Parameters:
// - cutoffFreq: cutoff frequency in Hz
// - sampleRate: sample rate in Hz
// - numTaps: filter length (must be odd)
pub fn generate_low_pass_kernel(
    cutoff_freq: f64,
    sample_rate: u32,
    filter_length: usize,
) -> Vec<f64> {
    // Create a kernel with the number of taps
    let mut kernel = vec![0.0; filter_length];
    let fc = cutoff_freq / sample_rate as f64;
    let m = (filter_length - 1) as f64;
    let half_m = m / 2.0;

    for i in 0..filter_length {
        let n = i as f64;
        if (n - half_m).abs() < 1e-9 {
            // handle division by zero at the center
            kernel[i] = 2.0 * fc;
        } else {
            kernel[i] = (2.0 * PI * fc * (n - half_m)).sin() / (PI * (n - half_m));
        }
        // Apply Hamming window
        kernel[i] *= 0.54 - 0.46 * (2.0 * PI * n / m).cos();
    }

    // Normalize the kernel
    let sum: f64 = kernel.iter().sum();
    if sum != 0.0 {
        for i in 0..filter_length {
            kernel[i] /= sum;
        }
    }

    kernel
}

// ApplyFIRFilter applies an FIR filter to the input signal using the provided kernel.
// Filtering: ApplyFIRFilter convolves the generated kernel with the audio signal, effectively removing frequencies above the desired cutoff (half of the target sample rate).
pub fn apply_fir_filter(input: &[f64], kernel: &[f64]) -> Vec<f64> {
    let mut n = input.len();
    let mut k = kernel.len();
    let mut output = vec![0.0; n];
    let half = k / 2;

    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..k {
            let index = i as isize - j as isize + half as isize;
            if index >= 0 && index < n as isize {
                sum += input[index as usize] * kernel[j];
            }
        }
        output[i] = sum; // Assign the calculated sum to the output vector
    }

    output
}
