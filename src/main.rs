// This is the main file for the audio file reader

mod dsp;
mod fingerprint;
mod utils;
mod wav;

use fingerprint::fingerprint::calc_fingerprint;
use rusty_chromaprint::{match_fingerprints, Configuration};

fn main() -> anyhow::Result<()> {
    let config = Configuration::preset_test1();
    // let song_path = "samples/song1.wav";
    // let test_path = "samples/recording.wav";

    let args: Vec<_> = std::env::args_os().collect();

    if args.len() != 3 {
        eprintln!("missing paths to files");
        return Ok(());
    }

    let song_path = &args[1];
    let test_path = &args[2];

    std::println!("Processing original song...");
    let fp1 = calc_fingerprint(song_path, &config, Some("original.png"))?;

    println!("Processing test recording...");
    let fp2 = calc_fingerprint(test_path, &config, Some("reference.png"))?;

    let segments = match_fingerprints(&fp1, &fp2, &config)?;

    println!(
        "\n  #  |          File 1          |          File 2          |  Duration  |  Score  "
    );
    println!("-----+--------------------------+--------------------------+------------+---------");
    for (idx, segment) in segments.iter().enumerate() {
        // println!(
        //     "{:>4} | {} -- {} | {} -- {} | {} | {:>6.02}",
        //     idx + 1,
        //     segment.start1(&config),
        //     segment.end1(&config),
        //     segment.start2(&config),
        //     segment.end2(&config),
        //     segment.duration(&config),
        //     segment.score,
        // );

        println!("Analysis: {:?}", segment)
    }

    Ok(())
}
