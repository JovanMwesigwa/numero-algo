use wav::read_wav_file;

mod wav;

fn main() -> anyhow::Result<()> {
    let song_path = "samples/song1.wav";
    let test_path = "samples/clip1.wav";

    read_wav_file(song_path);

    Ok(())
}
