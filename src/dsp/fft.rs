// FFT and Spectrogram Creation
// Each windowed frame is transformed into the frequency domain using the Fast Fourier Transform (FFT):
// FFT Computation: In fft.rs, ComputeFFT computes the FFT of a real-valued frame using the Gonum DSP library. Since the FFT of a real signal is symmetric, only the first half of the magnitude spectrum is kept.

use rustfft::{num_complex::Complex, FftPlanner};

// computes the FFT of a real valued frame. returns the magnitude spectrum
// (only the first half is returned since the input is real-valued).
pub fn compute_fft(frame: Vec<f32>) -> Vec<f64> {
    let n = frame.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);

    let mut complex_frame: Vec<Complex<f64>> =
        frame.iter().map(|&x| Complex::new(x as f64, 0.0)).collect();
    fft.process(&mut complex_frame);

    let half = n / 2 + 1;
    let mut magnitudes = Vec::with_capacity(half);

    for i in 0..half {
        magnitudes.push(complex_frame[i].norm());
    }

    magnitudes
}
