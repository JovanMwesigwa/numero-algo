// This is the main file for the audio file reader

mod utils;
mod wav;

use console::style;
use std::path::Path;
use tracing::info;

fn main() {
    // Initialize the tracing subscriber with a more compact format
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .init();

    let path = "samples/godown.wav";

    // Check if file exists
    if !Path::new(path).exists() {
        println!("{}", style("Error: File not found!").red().bold());
        println!("Path: {}", style(path).yellow());
        return;
    }

    match wav::read_audio_file(path) {
        Ok((samples, sample_rate)) => {
            // Print header
            println!("\n{}", style("🎵 Audio Analysis Report").cyan().bold());
            println!("{}", style("===================").cyan());

            // Format verification
            println!("\n{}", style("✅ Format Verification").green().bold());
            println!("{}", style("-------------------").green());
            info!("Format: {}", style("16-bit PCM").green());
            info!("Channels: {}", style("Mono").green());
            info!("Sample Rate: {} Hz", style(sample_rate).green());

            // Audio statistics
            println!("\n{}", style("📊 Audio Statistics").cyan().bold());
            println!("{}", style("-----------------").cyan());
            info!("Total Samples: {}", style(samples.len()).yellow());
            info!(
                "Duration: {} seconds",
                style(format!("{:.2}", samples.len() as f32 / sample_rate as f32)).yellow()
            );

            // Calculate and display audio statistics
            let stats = utils::calculate_audio_stats(&samples);

            println!("\n{}", style("📈 Sample Range").cyan().bold());
            println!("{}", style("-------------").cyan());
            info!("Minimum: {}", style(stats.min).blue());
            info!("Maximum: {}", style(stats.max).blue());
            info!(
                "Average Amplitude: {:.2}",
                style(stats.avg_amplitude).blue()
            );

            // Signal quality metrics
            println!("\n{}", style("📊 Signal Quality").cyan().bold());
            println!("{}", style("--------------").cyan());
            info!(
                "Zero Crossing Rate: {:.4}",
                style(stats.zero_crossing_rate).yellow()
            );

            // Sample preview
            println!("\n{}", style("👀 Sample Preview").cyan().bold());
            println!("{}", style("--------------").cyan());

            // Show first 5 samples
            println!("{}", style("First 5 samples:").dim());
            for (i, &sample) in samples.iter().take(5).enumerate() {
                info!("Sample {}: {}", style(i).dim(), style(sample).green());
            }

            // Show 5 samples from the middle of the file
            let mid = samples.len() / 2;
            println!("\n{}", style("Samples from middle of file:").dim());
            for i in mid..mid + 5 {
                if i < samples.len() {
                    info!("Sample {}: {}", style(i).dim(), style(samples[i]).green());
                }
            }
        }
        Err(e) => {
            println!("\n{}", style("❌ Error Reading Audio").red().bold());
            println!("{}", style("-----------------").red());
            println!("{}", style(format!("Error: {}", e)).red());
        }
    }
}
