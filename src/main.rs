// This is the main file for the audio file reader

mod dsp;
mod fingerprint;
mod utils;
mod wav;

use console::style;
use std::path::Path;

use fingerprint::{finger_print, match_fingerprints};

fn process_file(path: &str) -> Result<(Vec<u32>, u32, f64), String> {
    if !Path::new(path).exists() {
        return Err(format!("File not found: {}", path));
    }

    let (samples, sample_rate) = wav::read_audio_file(path).map_err(|e| e.to_string())?;
    let duration = samples.len() as f64 / sample_rate as f64;
    let fingerprint = finger_print(&samples, sample_rate)?;

    Ok((fingerprint, sample_rate, duration))
}

fn main() {
    // Process the full song
    println!("\n{}", style("Processing full song...").blue().bold());
    let song_path = "samples/song1.wav";
    let (song_fp, song_rate, song_duration) = match process_file(song_path) {
        Ok(result) => result,
        Err(e) => {
            println!("{}: {}", style("Error").red().bold(), e);
            return;
        }
    };

    println!(
        "{} Processed song: {} ({:.2} seconds, {} Hz)",
        style("✓").green().bold(),
        style(song_path).yellow(),
        song_duration,
        song_rate
    );

    // Process the test file
    println!("\n{}", style("Processing test file...").blue().bold());
    let test_path = "samples/recording.wav";
    let (test_fp, test_rate, test_duration) = match process_file(test_path) {
        Ok(result) => result,
        Err(e) => {
            println!("{}: {}", style("Error").red().bold(), e);
            return;
        }
    };

    println!(
        "{} Processed file: {} ({:.2} seconds, {} Hz)",
        style("✓").green().bold(),
        style(test_path).yellow(),
        test_duration,
        test_rate
    );

    // Compare fingerprints
    println!("\n{}", style("Comparing fingerprints...").blue().bold());
    let confidence = match_fingerprints(&test_fp, &song_fp);

    println!("\nResults:");
    println!(
        "• Song fingerprint length: {}",
        style(song_fp.len()).yellow()
    );
    println!(
        "• Test fingerprint length: {}",
        style(test_fp.len()).yellow()
    );
    println!(
        "• Match confidence: {}%",
        style(format!("{:.1}", confidence)).cyan().bold()
    );

    // Interpret results
    println!("\nInterpretation:");
    let interpretation = if confidence > 70.0 {
        "Very strong match - Audio files are very likely the same"
    } else if confidence > 40.0 {
        "Strong match - Audio files are probably related"
    } else if confidence > 20.0 {
        "Moderate match - Some similarity detected"
    } else if confidence > 5.0 {
        "Weak match - Slight similarity detected"
    } else {
        "No significant match - Audio files are different"
    };

    println!("{}", style(interpretation).yellow().bold());
}
