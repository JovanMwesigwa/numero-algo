// This is the main file for the audio file reader

mod dsp;
mod fingerprint;
mod utils;
mod wav;

use console::style;
use std::path::Path;

use fingerprint::finger_print;

fn main() {
    let path = "samples/godown.wav";

    // Check if file exists
    if !Path::new(path).exists() {
        println!("{}", style("Error: File not found!").red().bold());
        println!("Path: {}", style(path).yellow());
        return;
    }

    let (samples, sample_rate) = wav::read_audio_file(path).unwrap();

    let fingerprint = finger_print(&samples, sample_rate).unwrap();

    // println!("Fingerprint: {:?}", fingerprint);
}
