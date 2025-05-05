// frameSignal divides the signal into overlapping frames.

// Framing the Signal
// Using frameSignal from utils.go, the downsampled signal is split into overlapping frames (each 1024 samples long, with a hop size of 512).

pub fn frame_signal(signal: &[f64], frame_size: usize, hop_size: usize) -> Vec<Vec<f64>> {
    let mut frames: Vec<Vec<f64>> = Vec::new();
    let mut n = signal.len();

    for start in (0..n).step_by(hop_size) {
        let end = start + frame_size;
        if start + frame_size > n {
            // Check if enough samples are left for a full frame
            break;
        }
        frames.push(signal[start..end].to_vec());
    }

    frames
}

// Windowing
// To minimize abrupt discontinuities at the edges of each frame, a Hamming window is applied (via hammingWindow). This function creates a smooth taper that reduces spectral leakage when performing the FFT.

pub fn hamming_window(n: usize) -> Vec<f64> {
    let mut window = vec![0.0; n];
    if n > 1 {
        // Avoid division by zero if n is 1
        for i in 0..n {
            window[i] =
                0.54 - 0.46 * (2.0 * std::f64::consts::PI * i as f64 / (n as f64 - 1.0)).cos();
        }
    } else if n == 1 {
        window[0] = 1.0;
    }

    window
}
