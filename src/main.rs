use wav::read_wav_file;

mod wav;

fn main() -> anyhow::Result<()> {
    let song_path = "samples/song1.wav";
    let test_path = "samples/clip1.wav";

    let (samples, sample_rate) = read_wav_file(song_path)?;
    println!("Read {} samples at {} Hz", samples.len(), sample_rate);

    Ok(())
}
