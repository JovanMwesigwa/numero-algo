// Parallel Processing: spectogram.rs's computeSpectrogram function applies the FFT on each frame concurrently using goroutines, speeding up the process.

use std::sync::{Arc, Mutex};
use std::thread;

use crate::dsp::fft::compute_fft;

pub fn compute_spectrogram(frames: Vec<Vec<f32>>, window: Vec<f32>) -> Vec<Vec<f64>> {
    let num_frames = frames.len();
    let spectrogram = Arc::new(Mutex::new(vec![Vec::new(); num_frames]));
    let window_arc = Arc::new(window.to_vec());
    let frames_arc = Arc::new(frames.to_vec());
    let mut handles = Vec::new();

    for i in 0..num_frames {
        let spectrogram_clone = Arc::clone(&spectrogram);
        let window_clone = Arc::clone(&window_arc);
        let frames_clone = Arc::clone(&frames_arc);

        let handle = thread::spawn(move || {
            let mut frame = frames_clone[i].clone();
            for j in 0..frame.len() {
                frame[j] *= window_clone[j];
            }

            let magnitudes = compute_fft(frame);
            let mut spectrogram_lock = spectrogram_clone.lock().unwrap();
            spectrogram_lock[i] = magnitudes;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let spectrogram_lock = spectrogram.lock().unwrap();
    spectrogram_lock.clone()
}
