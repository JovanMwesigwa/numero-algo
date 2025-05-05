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

    // For each hash in the sample fingerprint
    for (i, &hash1) in fp1.iter().enumerate() {
        let (f1_1, f2_1, dt1) = decode_hash(hash1);
        let time1 = i as f64 * 0.0464; // Time offset based on hop size

        // Look for matching hashes in the reference fingerprint
        for (j, &hash2) in fp2.iter().enumerate() {
            let (f1_2, f2_2, dt2) = decode_hash(hash2);
            let time2 = j as f64 * 0.0464;

            // Allow for frequency bin differences due to sample rate variations
            let freq1_match = (f1_1 as i32 - f1_2 as i32).abs() <= 2;
            let freq2_match = (f2_1 as i32 - f2_2 as i32).abs() <= 2;

            // Allow for time delta differences
            let dt_match = (dt1 as i32 - dt2 as i32).abs() <= 3;

            if freq1_match && freq2_match && dt_match {
                matches.push((time1, time2));
            }
        }
    }

    if matches.is_empty() {
        return 0.0;
    }

    // Group matches by their time offset differences
    let mut time_deltas: Vec<f64> = matches.iter().map(|(t1, t2)| t1 - t2).collect();
    time_deltas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Find the largest cluster of matches with similar time offsets
    let mut max_cluster = 0;
    let mut current_cluster = 1;
    let time_tolerance = 0.5; // 500ms tolerance

    for i in 1..time_deltas.len() {
        if (time_deltas[i] - time_deltas[i - 1]).abs() < time_tolerance {
            current_cluster += 1;
        } else {
            max_cluster = max_cluster.max(current_cluster);
            current_cluster = 1;
        }
    }
    max_cluster = max_cluster.max(current_cluster);

    // Calculate minimum required matches based on clip length
    let min_matches = ((fp1.len().min(fp2.len()) as f64).sqrt() * 0.5) as usize;

    if max_cluster < min_matches {
        return 0.0;
    }

    // Calculate confidence score
    let confidence = (max_cluster as f64 / min_matches as f64).min(1.0) * 100.0;

    confidence
}
