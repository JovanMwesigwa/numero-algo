mod dsp;
mod fingerprint;
mod utils;
mod wav;

use console::style;
use std::collections::HashMap;
use std::path::Path;

use fingerprint::finger_print;
use wav::read_audio_file;

fn main() {
    // --- Process the full song ---
    println!("\n{}", style("Processing full song...").blue().bold());
    let song_path = "samples/song1.wav";

    let (song_samples, song_sample_rate) = read_audio_file(song_path)
        .map_err(|e| e.to_string())
        .unwrap();

    let song_fingerprint = finger_print(&song_samples, song_sample_rate).unwrap();
    let song_duration = song_samples.len() as f64 / song_sample_rate as f64;

    println!(
        "{} Generated fingerprint for song ({:.2} seconds, {} fingerprints)",
        style("✓").green().bold(),
        song_duration,
        song_fingerprint.len()
    );

    // --- Process the test clip ---
    println!("\n{}", style("Processing test clip...").green().bold());
    let test_path = "samples/clip1.wav";

    let (clip_samples, clip_sample_rate) = read_audio_file(test_path)
        .map_err(|e| e.to_string())
        .unwrap();

    let clip_fingerprint = finger_print(&clip_samples, clip_sample_rate).unwrap();
    let clip_duration = clip_samples.len() as f64 / clip_sample_rate as f64;

    println!(
        "{} Generated fingerprint for clip ({:.2} seconds, {} fingerprints)",
        style("✓").green().bold(),
        clip_duration,
        clip_fingerprint.len()
    );

    // --- Matching the fingerprint ---
    println!("\n{}", style("Matching fingerprints...").yellow().bold());
    if let Some(offset) = find_match(&song_fingerprint, &clip_fingerprint) {
        let time_offset = offset as f64 * 0.0464;

        if time_offset >= song_duration {
            println!("{}", style("Invalid match time detected.").bold().red());
            return;
        }

        let percent_through = (time_offset / song_duration) * 100.0;

        println!(
            "{} Match found at {:.2} seconds into the song ({:.1}% through)",
            style("✓").green().bold(),
            time_offset,
            percent_through
        );

        // Verify the match by checking surrounding context
        let window_start = offset.saturating_sub(50);
        let window_end = (offset + 50).min(song_fingerprint.len());
        let match_length = count_matching_frames(
            &song_fingerprint[window_start..window_end],
            &clip_fingerprint,
            0,
        );

        println!(
            "Match verification: {} matching fingerprints in surrounding context",
            style(match_length).cyan().bold(),
        );
    } else {
        println!("{}", style("No match found.").bold().red());
    }
}

fn decode_hash(hash: u32) -> (u32, u32, u32) {
    let f1 = (hash >> 23) & 0x1FF;
    let f2 = (hash >> 14) & 0x1FF;
    let dt = hash & 0x3FFF;
    (f1, f2, dt)
}

fn get_freq_signature(f1: u32, f2: u32) -> i32 {
    (f2 as i32) - (f1 as i32)
}

fn find_match(song_fingerprint: &[u32], clip_fingerprint: &[u32]) -> Option<usize> {
    if clip_fingerprint.is_empty() || song_fingerprint.is_empty() {
        return None;
    }

    let mut matches = Vec::new();
    let window_size = 5; // Look at groups of 5 fingerprints

    // Process clip in windows
    for clip_start in (0..clip_fingerprint.len()).step_by(window_size) {
        let clip_end = (clip_start + window_size).min(clip_fingerprint.len());
        let clip_window = &clip_fingerprint[clip_start..clip_end];

        // Extract frequency pattern from clip window
        let clip_pattern: Vec<_> = clip_window
            .iter()
            .map(|&hash| {
                let (f1, f2, _) = decode_hash(hash);
                (f1, f2)
            })
            .collect();

        // Search for this pattern in song
        'outer: for i in 0..song_fingerprint.len().saturating_sub(window_size) {
            let mut matches_in_window = 0;

            // Compare each fingerprint in the window
            for (j, &(clip_f1, clip_f2)) in clip_pattern.iter().enumerate() {
                if i + j >= song_fingerprint.len() {
                    break;
                }

                let (song_f1, song_f2, _) = decode_hash(song_fingerprint[i + j]);

                // Allow small frequency differences
                if (song_f1 as i32 - clip_f1 as i32).abs() <= 2
                    && (song_f2 as i32 - clip_f2 as i32).abs() <= 2
                {
                    matches_in_window += 1;
                } else if matches_in_window < 2 {
                    // If we don't have at least 2 matches, try next position
                    continue 'outer;
                }
            }

            if matches_in_window >= 3 {
                // At least 3 matches in window
                matches.push((i, matches_in_window));
            }
        }
    }

    // Find best sequence of matching windows
    if matches.is_empty() {
        return None;
    }

    matches.sort_by_key(|&(pos, _)| pos);

    let mut best_start = matches[0].0;
    let mut best_length = matches[0].1;
    let mut current_start = matches[0].0;
    let mut current_length = matches[0].1;

    for i in 1..matches.len() {
        let (pos, length) = matches[i];
        let gap = pos - (matches[i - 1].0 + window_size);

        if gap <= window_size {
            // Windows are close enough to be part of same sequence
            current_length += length;
        } else {
            if current_length > best_length {
                best_length = current_length;
                best_start = current_start;
            }
            current_start = pos;
            current_length = length;
        }
    }

    if current_length > best_length {
        best_length = current_length;
        best_start = current_start;
    }

    println!(
        "\nFound sequence with {} matching fingerprints",
        best_length
    );

    if best_length >= 15 {
        // Require substantial matching sequence
        Some(best_start)
    } else {
        None
    }
}

fn count_matching_frames(song_fp: &[u32], clip_fp: &[u32], offset: usize) -> usize {
    let mut matches = 0;
    for (i, &clip_hash) in clip_fp.iter().enumerate() {
        let song_idx = offset + i;
        if song_idx < song_fp.len() {
            let (f1_clip, f2_clip, _) = decode_hash(clip_hash);
            let (f1_song, f2_song, _) = decode_hash(song_fp[song_idx]);
            if f1_clip == f1_song && f2_clip == f2_song {
                matches += 1;
            }
        }
    }
    matches
}
