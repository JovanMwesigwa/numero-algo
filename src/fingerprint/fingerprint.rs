//  Main fingerprinting algorithm
// Normalization
// Inside this function (in fingerprint.go), the raw int16 samples are converted into float64 values scaled between –1 and 1:

use std::fs::File;
use std::path::Path;

use anyhow::Context;
use rusty_chromaprint::{Configuration, Fingerprinter};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::dsp::filter::{apply_fir_filter, generate_low_pass_kernel};
use crate::dsp::viz::plot_spectrogram;
use crate::fingerprint::hash::hash_fingerprint;
use crate::fingerprint::peaks::detect_peaks;
use crate::fingerprint::spectogram::compute_spectrogram;
use crate::fingerprint::utils::{frame_signal, hamming_window};

const TARGET_SAMPLE_RATE: u32 = 11025; // Downsampled rate.
const FILTER_TAPS: usize = 101; // Samples per frame
const FRAME_SIZE: usize = 1024; // Samples per frame
const HOP_SIZE: usize = 512; // Hop size for overlapping frames
const NUM_BANDS: usize = 6; // Number of frequency bands for peak detection
const TARGET_ZONE_FRAMES: usize = 20; // Maximum frame difference for pairing peaks

pub fn finger_print(samples: &[i16], sample_rate: u32) -> Result<Vec<u32>, String> {
    // Check if samples are empty or sample rate is lower that the target sample rate
    if samples.is_empty() || sample_rate < TARGET_SAMPLE_RATE {
        return Err("Invalid input: samples are empty or sample rate is too low".to_string());
    }

    // Find the maximum absolute value of the samples, handling i16::MIN specially
    let max_abs = samples
        .iter()
        .map(|&s| {
            if s == i16::MIN {
                32768_i32
            } else {
                s.abs() as i32
            }
        })
        .max()
        .unwrap_or(1) as f64;

    // Convert the raw i16 samples to f64 values scaled between -1.0 and 1.0
    let normalized_samples: Vec<f64> = samples
        .iter()
        .map(|&s| {
            if s == i16::MIN {
                -1.0 // i16::MIN maps to -1.0
            } else {
                (s as f64) / max_abs
            }
        })
        .collect();

    // Get the cutoff frequency for downsampling
    let cutoff_freq = TARGET_SAMPLE_RATE as f64 / 2.0;

    // Generate the low-pass filter
    let kernel = generate_low_pass_kernel(cutoff_freq, sample_rate, FILTER_TAPS);

    // Apply the filter
    let filtered = apply_fir_filter(&normalized_samples, &kernel);

    // Downsample the filtered signal
    let decimation_factor = sample_rate as f64 / TARGET_SAMPLE_RATE as f64;
    let mut downsampled = vec![0.0; filtered.len() / decimation_factor as usize];
    for i in 0..downsampled.len() {
        downsampled[i] = filtered[i * decimation_factor as usize];
    }

    // Framing the Signal
    let frames = frame_signal(&downsampled, FRAME_SIZE, HOP_SIZE);

    // Windowing
    let window = hamming_window(FRAME_SIZE);

    // Compute the spectrogram
    let spectrogram = compute_spectrogram(
        frames
            .iter()
            .map(|f| f.iter().map(|&x| x as f32).collect())
            .collect(),
        window.iter().map(|&x| x as f32).collect(),
    );

    // Detect Peaks
    let peaks = detect_peaks(&spectrogram, NUM_BANDS);

    if let Err(e) = plot_spectrogram(
        &spectrogram,
        TARGET_SAMPLE_RATE,
        FRAME_SIZE,
        HOP_SIZE,
        &peaks,
        "spectrogram.png",
    ) {
        eprintln!("Warning: Failed to plot spectrogram: {}", e);
    }

    // Generate and return the fingerprint hashes
    let hashes = hash_fingerprint(&peaks, TARGET_ZONE_FRAMES);
    Ok(hashes)
}

pub fn plot_file_spectrogram(
    samples: &[i16],
    sample_rate: u32,
    out_path: &str,
) -> Result<(), String> {
    // Normalize samples
    let max_abs = samples
        .iter()
        .map(|&s| {
            if s == i16::MIN {
                32768_i32
            } else {
                s.abs() as i32
            }
        })
        .max()
        .unwrap_or(1) as f64;

    let normalized_samples: Vec<f64> = samples
        .iter()
        .map(|&s| {
            if s == i16::MIN {
                -1.0
            } else {
                (s as f64) / max_abs
            }
        })
        .collect();

    // Frame the signal
    let frames = frame_signal(&normalized_samples, FRAME_SIZE, HOP_SIZE);
    let window = hamming_window(FRAME_SIZE);

    // Compute spectrogram
    let spectrogram = compute_spectrogram(
        frames
            .iter()
            .map(|f| f.iter().map(|&x| x as f32).collect())
            .collect(),
        window.iter().map(|&x| x as f32).collect(),
    );

    // Detect peaks
    let peaks = detect_peaks(&spectrogram, NUM_BANDS);

    // Plot spectrogram with peaks
    if let Err(e) = plot_spectrogram(
        &spectrogram,
        sample_rate,
        FRAME_SIZE,
        HOP_SIZE,
        &peaks,
        out_path,
    ) {
        eprintln!("Warning: Failed to plot spectrogram: {}", e);
    }

    Ok(())
}

pub fn calc_fingerprint(
    path: impl AsRef<Path>,
    config: &Configuration,
    plot_path: Option<&str>,
) -> anyhow::Result<Vec<u32>> {
    let path = path.as_ref();
    let src = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .context("unsupported format")?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .context("no supported audio tracks")?;

    let dec_opts: DecoderOptions = Default::default();

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .context("unsupported codec")?;

    let track_id = track.id;

    let mut printer = Fingerprinter::new(config);
    let sample_rate = track
        .codec_params
        .sample_rate
        .context("missing sample rate")?;
    let channels = track
        .codec_params
        .channels
        .context("missing audio channels")?
        .count() as u32;

    // start the printer
    printer
        .start(sample_rate, channels)
        .context("initializing fingerprinter")?;

    let mut sample_buf = None;
    let mut all_samples = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                if sample_buf.is_none() {
                    let spec = *audio_buf.spec();
                    let duration = audio_buf.capacity() as u64;
                    sample_buf = Some(SampleBuffer::<i16>::new(duration, spec));
                }

                if let Some(buf) = &mut sample_buf {
                    buf.copy_interleaved_ref(audio_buf);
                    printer.consume(buf.samples());
                    all_samples.extend_from_slice(buf.samples());
                }
            }
            Err(Error::DecodeError(_)) => (),
            Err(_) => break,
        }
    }

    // Plot spectrogram if requested
    // if let Some(out_path) = plot_path {
    //     if let Err(e) = plot_file_spectrogram(&all_samples, sample_rate, out_path) {
    //         eprintln!("Warning: Failed to plot spectrogram: {}", e);
    //     }
    // }

    printer.finish();
    Ok(printer.fingerprint().to_vec())
}
