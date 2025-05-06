pub mod fingerprint;
pub mod hash;
pub mod peaks;
pub mod spectogram;
pub mod utils;
// Re-export main functionality for easier access
pub use self::fingerprint::finger_print;
pub use self::hash::hash_fingerprint;
pub use self::utils::frame_signal;

/// Extract components from a hash
fn decode_hash(hash: u32) -> (u32, u32, u32) {
    let f1 = (hash >> 23) & 0x1FF; // First 9 bits
    let f2 = (hash >> 14) & 0x1FF; // Next 9 bits
    let dt = hash & 0x3FFF; // Last 14 bits
    (f1, f2, dt)
}

/// Compares two fingerprints and returns a confidence score
pub fn match_fingerprints(fp1: &[u32], fp2: &[u32]) -> f64 {
    let mut matches = Vec::new();
    const FREQ_TOLERANCE: f64 = 0.15; // 15% frequency tolerance for recording conditions
    const TIME_TOLERANCE: f64 = 0.15; // 150ms time tolerance
    const MIN_MATCHES: usize = 5; // Minimum matches needed for a valid sequence

    // For each hash in the sample fingerprint
    for (i, &hash1) in fp1.iter().enumerate() {
        let (f1_1, f2_1, dt1) = decode_hash(hash1);
        if f1_1 == 0 || f2_1 == 0 {
            continue;
        }

        // Convert frequencies to relative positions in the spectrum
        let freq_pos1 = f1_1 as f64 / 512.0;
        let freq_pos2 = f2_1 as f64 / 512.0;
        let time1 = i as f64 * 0.0464;

        // Search window size adapts to fingerprint lengths
        let window_size = (fp2.len() as f64 * 0.25) as usize;
        let start_idx = if i >= window_size { i - window_size } else { 0 };
        let end_idx = (i + window_size).min(fp2.len());

        for j in start_idx..end_idx {
            let hash2 = fp2[j];
            let (f1_2, f2_2, dt2) = decode_hash(hash2);
            if f1_2 == 0 || f2_2 == 0 {
                continue;
            }

            // Convert frequencies to relative positions
            let freq_pos1_2 = f1_2 as f64 / 512.0;
            let freq_pos2_2 = f2_2 as f64 / 512.0;

            // Compare relative frequency positions with tolerance
            let freq_diff1 = (freq_pos1 - freq_pos1_2).abs();
            let freq_diff2 = (freq_pos2 - freq_pos2_2).abs();

            if freq_diff1 <= FREQ_TOLERANCE && freq_diff2 <= FREQ_TOLERANCE {
                // Compare time deltas to handle speed variations
                let dt_ratio = dt1 as f64 / dt2.max(1) as f64;
                if (0.8..=1.2).contains(&dt_ratio) {
                    // Allow 20% speed variation
                    let time2 = j as f64 * 0.0464;
                    matches.push((time1, time2, (freq_diff1 + freq_diff2) / 2.0));
                }
            }
        }
    }

    if matches.len() < MIN_MATCHES {
        return 0.0;
    }

    // Sort matches by time in clip
    matches.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Find sequences of consistent matches
    let mut max_sequence = 0;
    let mut current_sequence = 1;
    let mut total_quality: f64 = 0.0;
    let mut best_quality: f64 = 0.0;

    for i in 1..matches.len() {
        let (prev_t1, prev_t2, prev_quality) = matches[i - 1];
        let (curr_t1, curr_t2, curr_quality) = matches[i];

        let dt1 = curr_t1 - prev_t1;
        let dt2 = curr_t2 - prev_t2;

        // Check if the time deltas are consistent (allowing for speed variations)
        let time_ratio = if dt2 != 0.0 { dt1 / dt2 } else { 0.0 };
        if dt1 > 0.0 && dt2 > 0.0 && (0.8..=1.2).contains(&time_ratio) {
            current_sequence += 1;
            total_quality += 1.0 - (curr_quality + prev_quality) / 2.0;
            best_quality = f64::max(best_quality, total_quality);
        } else {
            if current_sequence > max_sequence {
                max_sequence = current_sequence;
            }
            current_sequence = 1;
            total_quality = 0.0;
        }
    }

    max_sequence = max_sequence.max(current_sequence);

    if max_sequence < MIN_MATCHES {
        return 0.0;
    }

    // Calculate confidence score (0-100%)
    let sequence_score = (max_sequence as f64 / MIN_MATCHES as f64).min(1.0);
    let quality_score = (best_quality / max_sequence as f64).min(1.0);

    (sequence_score * quality_score * 100.0).min(100.0)
}
