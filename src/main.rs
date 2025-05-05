// This is the main file for the audio file reader

mod fingerprint;
mod utils;
mod wav;

use console::style;
use std::path::Path;
use tracing::info;

use fingerprint::{finger_print, hash};

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

    let (samples, sample_rate) = wav::read_audio_file(path).unwrap();

    let fingerprint = finger_print(&samples, sample_rate).unwrap();

    println!("Fingerprint: {:?}", fingerprint);
}
