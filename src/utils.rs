/// Audio processing utility functions

/// Safely convert i16 to absolute value as f32, handling MIN_VALUE case
pub fn safe_abs(x: i16) -> f32 {
    if x == i16::MIN {
        32768.0
    } else {
        x.abs() as f32
    }
}

/// Validates that the audio data meets our format requirements
pub fn validate_audio_format(samples: &[i16], sample_rate: u32) -> Result<(), String> {
    // Check if we have any samples
    if samples.is_empty() {
        return Err("No audio samples found".to_string());
    }

    // Verify sample rate is standard
    if ![44100, 48000, 96000].contains(&sample_rate) {
        return Err(format!("Unusual sample rate: {} Hz", sample_rate));
    }

    // Verify samples are within 16-bit range
    if let (Some(&min), Some(&max)) = (samples.iter().min(), samples.iter().max()) {
        if min < -32768 || max > 32767 {
            return Err(format!(
                "Samples outside 16-bit range: min={}, max={}",
                min, max
            ));
        }
    }

    // Calculate average absolute difference between consecutive samples
    // This can help detect if we're truly mono (should have smooth transitions)
    let avg_diff: f32 = samples
        .windows(2)
        .map(|w| {
            let a = safe_abs(w[0]);
            let b = safe_abs(w[1]);
            (b - a).abs()
        })
        .sum::<f32>()
        / (samples.len() - 1) as f32;

    // If average difference is too high, might indicate incorrect channel mixing
    if avg_diff > 10000.0 {
        return Err(format!(
            "Unusually high sample variation (avg_diff={}), possible stereo mixing issue",
            avg_diff
        ));
    }

    Ok(())
}

/// Calculate audio statistics for a given sample buffer
pub fn calculate_audio_stats(samples: &[i16]) -> AudioStats {
    let (min, max) = samples.iter().fold((i16::MAX, i16::MIN), |(min, max), &x| {
        (min.min(x), max.max(x))
    });

    let avg_amplitude = samples.iter().map(|&x| safe_abs(x)).sum::<f32>() / samples.len() as f32;

    let zero_crossings = samples
        .windows(2)
        .filter(|w| w[0].signum() != w[1].signum())
        .count();
    let zero_crossing_rate = zero_crossings as f32 / samples.len() as f32;

    AudioStats {
        min,
        max,
        avg_amplitude,
        zero_crossing_rate,
    }
}

/// Structure to hold audio statistics
#[derive(Debug)]
pub struct AudioStats {
    pub min: i16,
    pub max: i16,
    pub avg_amplitude: f32,
    pub zero_crossing_rate: f32,
}
