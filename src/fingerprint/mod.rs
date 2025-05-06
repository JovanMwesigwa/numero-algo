pub mod fingerprint;
pub mod hash;
pub mod peaks;
pub mod spectogram;
pub mod utils;
// Re-export main functionality for easier access
pub use self::fingerprint::finger_print;
pub use self::hash::hash_fingerprint;
pub use self::utils::frame_signal;

use rayon::prelude::*;

/// Extract components from a hash
fn decode_hash(hash: u32) -> (u32, u32, u32) {
    let f1 = (hash >> 23) & 0x1FF; // First 9 bits
    let f2 = (hash >> 14) & 0x1FF; // Next 9 bits
    let dt = hash & 0x3FFF; // Last 14 bits
    (f1, f2, dt)
}

/// Represents a match between two fingerprints
#[derive(Debug)]
struct FingerprintMatch {
    time1: f64,
    time2: f64,
    freq_confidence: f64,
    time_confidence: f64,
}

/// Compares two fingerprints and returns a confidence score
pub fn match_fingerprints(fp1: &[u32], fp2: &[u32]) -> f64 {
    const CHUNK_SIZE: usize = 1000; // Process in chunks of 1000 hashes
    const MIN_FREQ_RATIO: f64 = 0.9;
    const MAX_FREQ_RATIO: f64 = 1.1;

    let mut matches = Vec::new();
    let mut freq_distribution = vec![0; 512];

    // Process in chunks to avoid memory pressure
    for chunk in fp1.chunks(CHUNK_SIZE) {
        let chunk_matches: Vec<_> = chunk
            .par_iter()
            .enumerate()
            .flat_map(|(chunk_offset, &hash1)| {
                let mut local_matches = Vec::new();
                let (f1_1, f2_1, dt1) = decode_hash(hash1);
                let time1 = (chunk_offset as f64 * 0.0464)
                    + ((chunk.as_ptr() as usize - fp1.as_ptr() as usize)
                        / std::mem::size_of::<u32>()) as f64
                        * 0.0464;

                // Early exit if we've found enough matches
                if matches.len() > fp1.len() / 4 {
                    return local_matches;
                }

                // Look for matching hashes using relative frequency relationships
                for (j, &hash2) in fp2.iter().enumerate() {
                    let (f1_2, f2_2, dt2) = decode_hash(hash2);
                    let time2 = j as f64 * 0.0464;

                    // Calculate frequency ratios instead of absolute differences
                    let f1_ratio = f1_1 as f64 / f1_2 as f64;
                    let f2_ratio = f2_1 as f64 / f2_2 as f64;

                    // Check if ratios are within acceptable range
                    let freq_match = f1_ratio >= MIN_FREQ_RATIO
                        && f1_ratio <= MAX_FREQ_RATIO
                        && f2_ratio >= MIN_FREQ_RATIO
                        && f2_ratio <= MAX_FREQ_RATIO
                        && (f1_ratio - f2_ratio).abs() < 0.1;

                    // Calculate frequency confidence
                    let freq_confidence = if freq_match {
                        1.0 - ((f1_ratio - 1.0).abs() + (f2_ratio - 1.0).abs()) / 2.0
                    } else {
                        0.0
                    };

                    // Time delta matching with adaptive tolerance
                    let dt_diff = (dt1 as i32 - dt2 as i32).abs();
                    let dt_tolerance = (dt1.max(dt2) as f64 * 0.05) as i32;
                    let dt_match = dt_diff <= dt_tolerance;

                    // Calculate time confidence
                    let time_confidence = if dt_match {
                        1.0 - dt_diff as f64 / dt_tolerance as f64
                    } else {
                        0.0
                    };

                    if freq_match && dt_match {
                        local_matches.push(FingerprintMatch {
                            time1,
                            time2,
                            freq_confidence,
                            time_confidence,
                        });
                    }
                }
                local_matches
            })
            .collect();

        // Update frequency distribution
        for match_info in &chunk_matches {
            let hash = chunk[(match_info.time1 / 0.0464) as usize % chunk.len()];
            let (f1, f2, _) = decode_hash(hash);
            freq_distribution[f1 as usize] += 1;
            freq_distribution[f2 as usize] += 1;
        }

        matches.extend(chunk_matches);

        if matches.len() > fp1.len() / 4 {
            break;
        }
    }

    if matches.is_empty() {
        return 0.0;
    }

    // Verify frequency distribution
    let active_bands = freq_distribution.iter().filter(|&&count| count > 0).count();
    if active_bands < 20 {
        return 0.0;
    }

    // Group matches by time offset and validate sequence consistency
    let mut time_deltas: Vec<(f64, f64, f64)> = matches
        .iter()
        .map(|m| (m.time1 - m.time2, m.freq_confidence, m.time_confidence))
        .collect();
    time_deltas.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Find the best cluster using weighted confidence scores
    let mut max_cluster_score = 0.0;
    let mut current_cluster_score = time_deltas[0].1 * time_deltas[0].2;
    let mut current_cluster_size = 1;
    let time_tolerance = 0.25;

    for i in 1..time_deltas.len() {
        if (time_deltas[i].0 - time_deltas[i - 1].0).abs() < time_tolerance {
            current_cluster_size += 1;
            current_cluster_score += time_deltas[i].1 * time_deltas[i].2;
        } else {
            max_cluster_score = f64::max(max_cluster_score, current_cluster_score);
            current_cluster_score = time_deltas[i].1 * time_deltas[i].2;
            current_cluster_size = 1;
        }
    }
    max_cluster_score = f64::max(max_cluster_score, current_cluster_score);

    // Calculate minimum required matches with adaptive threshold
    let min_matches = ((fp1.len().min(fp2.len()) as f64).sqrt() * 0.6) as f64;

    if current_cluster_size as f64 * max_cluster_score < min_matches {
        return 0.0;
    }

    // Calculate final confidence score
    let freq_weight = (active_bands as f64 / 50.0).min(1.0);
    let cluster_score = (current_cluster_size as f64 * max_cluster_score / min_matches).min(1.0);
    let confidence = (cluster_score * freq_weight) * 100.0;

    confidence
}
