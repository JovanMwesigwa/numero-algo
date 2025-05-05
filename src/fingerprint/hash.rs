use super::peaks::Peak;

pub fn hash_fingerprint(peaks: &[Peak], target_zone: usize) -> Vec<u32> {
    let mut hashes = Vec::new();

    // For each peak (anchor)
    for i in 0..peaks.len() {
        let anchor = &peaks[i];

        // Look for peaks within the target zone
        for j in (i + 1)..peaks.len() {
            let target = &peaks[j];
            let dt = target.frame_index as isize - anchor.frame_index as isize;

            // Skip if time difference is negative or beyond target zone
            if dt < 0 || dt > target_zone as isize {
                continue;
            }

            // Encode the hash:
            // - 9 bits for anchor frequency (0-511)
            // - 9 bits for target frequency (0-511)
            // - 14 bits for time delta (0-16383)

            let mut f1 = anchor.freq_bin as u32;
            let mut f2 = target.freq_bin as u32;
            let mut dt_u = dt as u32;

            // Clamp values to their bit ranges
            if f1 > 0x1FF {
                f1 = 0x1FF;
            } // 9 bits max
            if f2 > 0x1FF {
                f2 = 0x1FF;
            } // 9 bits max
            if dt_u > 0x3FFF {
                dt_u = 0x3FFF;
            } // 14 bits max

            // Combine into 32-bit hash
            let hash = (f1 << 23) | (f2 << 14) | dt_u;
            hashes.push(hash);
        }
    }

    hashes
}
