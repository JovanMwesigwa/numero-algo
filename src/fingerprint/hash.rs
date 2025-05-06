use super::peaks::Peak;

// HashFingerprint creates 32-bit hashes from pairs of audio peaks.
// Each hash combines:
// - 9 bits: anchor frequency
// - 9 bits: target frequency
// - 14 bits: time delta between peaks
pub fn hash_fingerprint(peaks: &[Peak], target_zone: usize) -> Vec<u32> {
    let mut hashes = Vec::new();
    for i in 0..peaks.len() {
        let anchor = &peaks[i];
        for j in (i + 1)..peaks.len() {
            let target = &peaks[j];
            let dt = target.frame_index as isize - anchor.frame_index as isize;

            if dt < 0 {
                continue;
            }
            if dt > target_zone as isize {
                break;
            }

            let mut f1 = anchor.freq_bin as u32;
            let mut f2 = target.freq_bin as u32;
            let mut dt_u = dt as u32;

            if f1 > 0x1FF {
                f1 = 0x1FF;
            }
            if f2 > 0x1FF {
                f2 = 0x1FF;
            }
            if dt_u > 0x3FFF {
                dt_u = 0x3FFF;
            }

            let hash = (f1 << 23) | (f2 << 14) | dt_u;
            hashes.push(hash);
        }
    }
    hashes
}
