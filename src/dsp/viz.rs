use crate::fingerprint::peaks::Peak;
use plotters::prelude::*;
use std::path::Path;

/// Visualize the kernel in ASCII format
pub fn visualize_kernel(kernel: &[f64], width: usize) -> String {
    let mut result = String::new();
    result.push_str("\nKernel Visualization:\n");
    result.push_str("-------------------\n");

    // Find min and max for scaling
    let max_abs = kernel.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);

    for (i, &value) in kernel.iter().enumerate() {
        // Add index marker every 10 samples
        if i % 10 == 0 {
            result.push_str(&format!("{:3} │", i));
        } else {
            result.push_str("    │");
        }

        // Scale the value to fit the width
        let scaled = (value / max_abs * width as f64).abs().round() as usize;
        let marker = if value >= 0.0 { "▄" } else { "▀" };

        // Add padding before the bar
        let center = width / 2;
        if value >= 0.0 {
            result.push_str(&" ".repeat(center));
            result.push_str(&marker.repeat(scaled));
        } else {
            result.push_str(&" ".repeat(center - scaled));
            result.push_str(&marker.repeat(scaled));
        }

        // Add the actual value
        result.push_str(&format!(" {:.6}\n", value));
    }

    result.push_str("    └");
    result.push_str(&"─".repeat(width));
    result.push_str("\n");

    result
}

/// Plot the kernel using plotters library and save to a file
pub fn plot_kernel(
    kernel: &[f64],
    sample_rate: u32,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path.as_ref(), (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_abs = kernel.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
    let min_y = -max_abs * 1.1; // Add 10% margin
    let max_y = max_abs * 1.1;

    let mut chart = ChartBuilder::on(&root)
        .caption("FIR Filter Kernel", ("sans-serif", 30).into_font())
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..kernel.len() as f64, min_y..max_y)?;

    chart
        .configure_mesh()
        .x_desc("Sample")
        .y_desc("Amplitude")
        .draw()?;

    // Plot the kernel values
    chart.draw_series(LineSeries::new(
        kernel.iter().enumerate().map(|(i, &y)| (i as f64, y)),
        &BLUE,
    ))?;

    // Add points at each kernel value
    chart.draw_series(PointSeries::of_element(
        kernel.iter().enumerate().map(|(i, &y)| (i as f64, y)),
        5,
        &BLUE,
        &|c, s, st| {
            return EmptyElement::at(c)    // We want the point to be at (x, y)
                + Circle::new((0, 0), s, st.filled()); // And a circle at its center
        },
    ))?;

    // Add zero line
    chart.draw_series(LineSeries::new(
        vec![(0.0, 0.0), (kernel.len() as f64, 0.0)],
        &RED.mix(0.3),
    ))?;

    // Add title with kernel info as a separate drawing element
    let info_text = format!(
        "Sample Rate: {} Hz, Length: {} taps",
        sample_rate,
        kernel.len()
    );
    root.draw(&Text::new(
        info_text,
        (70, 30),
        ("sans-serif", 20).into_font(),
    ))?;

    root.present()?;

    Ok(())
}

/// Plot both the kernel and its frequency response
pub fn plot_kernel_with_frequency_response(
    kernel: &[f64],
    sample_rate: u32,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path.as_ref(), (1000, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let areas = root.split_evenly((2, 1));

    // Plot time domain (kernel)
    let max_abs = kernel.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
    let min_y = -max_abs * 1.1;
    let max_y = max_abs * 1.1;

    let mut chart = ChartBuilder::on(&areas[0])
        .caption(
            "FIR Filter Kernel (Time Domain)",
            ("sans-serif", 25).into_font(),
        )
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..kernel.len() as f64, min_y..max_y)?;

    chart
        .configure_mesh()
        .x_desc("Sample")
        .y_desc("Amplitude")
        .draw()?;

    chart.draw_series(LineSeries::new(
        kernel.iter().enumerate().map(|(i, &y)| (i as f64, y)),
        &BLUE,
    ))?;

    // Plot frequency response
    let freq_response = calculate_frequency_response(kernel, 512);
    let mut chart = ChartBuilder::on(&areas[1])
        .caption("Frequency Response", ("sans-serif", 25).into_font())
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..sample_rate as f64 / 2.0, -100.0..10.0)?;

    chart
        .configure_mesh()
        .x_desc("Frequency (Hz)")
        .y_desc("Magnitude (dB)")
        .draw()?;

    chart.draw_series(LineSeries::new(
        freq_response.iter().enumerate().map(|(i, &y)| {
            (
                i as f64 * (sample_rate as f64 / 2.0) / freq_response.len() as f64,
                20.0 * y.log10(),
            )
        }),
        &BLUE,
    ))?;

    root.present()?;

    Ok(())
}

/// Calculate the frequency response of the filter
fn calculate_frequency_response(kernel: &[f64], n_points: usize) -> Vec<f64> {
    let mut response = Vec::with_capacity(n_points);

    for k in 0..n_points {
        let freq = k as f64 * std::f64::consts::PI / n_points as f64;
        let mut real = 0.0;
        let mut imag = 0.0;

        for (n, &h) in kernel.iter().enumerate() {
            real += h * (freq * n as f64).cos();
            imag -= h * (freq * n as f64).sin();
        }

        response.push((real * real + imag * imag).sqrt());
    }

    response
}

/// Plot the original and filtered signals for comparison
pub fn plot_filter_comparison(
    original: &[f64],
    filtered: &[f64],
    sample_rate: u32,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path.as_ref(), (1000, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let areas = root.split_evenly((2, 1));

    // Find global min and max for consistent y-axis scaling
    let max_abs = original
        .iter()
        .chain(filtered.iter())
        .map(|x| x.abs())
        .fold(0.0_f64, f64::max);
    let min_y = -max_abs * 1.1;
    let max_y = max_abs * 1.1;

    // Time axis in seconds
    let duration = original.len() as f64 / sample_rate as f64;

    // Plot original signal
    let mut chart = ChartBuilder::on(&areas[0])
        .caption("Original Signal", ("sans-serif", 25).into_font())
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..duration, min_y..max_y)?;

    chart
        .configure_mesh()
        .x_desc("Time (seconds)")
        .y_desc("Amplitude")
        .draw()?;

    // Plot first 1000 points with higher resolution
    let plot_len = original.len().min(1000);
    let step = original.len() / plot_len;

    chart.draw_series(LineSeries::new(
        (0..plot_len).map(|i| {
            let idx = i * step;
            (idx as f64 / sample_rate as f64, original[idx])
        }),
        &BLUE,
    ))?;

    // Plot filtered signal
    let mut chart = ChartBuilder::on(&areas[1])
        .caption("Filtered Signal", ("sans-serif", 25).into_font())
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..duration, min_y..max_y)?;

    chart
        .configure_mesh()
        .x_desc("Time (seconds)")
        .y_desc("Amplitude")
        .draw()?;

    chart.draw_series(LineSeries::new(
        (0..plot_len).map(|i| {
            let idx = i * step;
            (idx as f64 / sample_rate as f64, filtered[idx])
        }),
        &RED,
    ))?;

    // Add info text
    let info_text = format!(
        "Sample Rate: {} Hz, Duration: {:.2} seconds",
        sample_rate, duration
    );
    root.draw(&Text::new(
        info_text,
        (70, 30),
        ("sans-serif", 20).into_font(),
    ))?;

    root.present()?;

    Ok(())
}

/// Plot the spectrogram as a heatmap
pub fn plot_spectrogram(
    spectrogram: &[Vec<f64>],
    sample_rate: u32,
    frame_size: usize,
    hop_size: usize,
    peaks: &[Peak],
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path.as_ref(), (1000, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Calculate time and frequency axes
    let duration = (spectrogram.len() * hop_size) as f64 / sample_rate as f64;
    let max_freq = sample_rate as f64 / 2.0; // Nyquist frequency

    // Find the maximum magnitude for color scaling (in dB)
    let max_magnitude = spectrogram
        .iter()
        .flat_map(|frame| frame.iter())
        .fold(0.0_f64, |max, &x| max.max(x));

    // Create the chart
    let mut chart = ChartBuilder::on(&root)
        .caption("Spectrogram", ("sans-serif", 30).into_font())
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .right_y_label_area_size(60) // Add space for colorbar
        .build_cartesian_2d(0.0..duration, 0.0..max_freq)?;

    chart
        .configure_mesh()
        .x_desc("Time (seconds)")
        .y_desc("Frequency (Hz)")
        .label_style(("sans-serif", 15))
        .draw()?;

    // Create color gradient for the heatmap
    let color_gradient = colorous::VIRIDIS;

    // Calculate pixel dimensions
    let pixel_width = 900.0 / spectrogram.len() as f64; // Reduced width to accommodate colorbar
    let pixel_height = 600.0 / frame_size as f64;

    // Draw each time-frequency bin
    for (t, frame) in spectrogram.iter().enumerate() {
        for (f, &magnitude) in frame.iter().take(frame_size / 2).rev().enumerate() {
            // Reverse frequency order
            let time = t as f64 * hop_size as f64 / sample_rate as f64;
            let freq = f as f64 * max_freq / (frame_size / 2) as f64;

            // Convert magnitude to dB and normalize
            let db = 20.0 * (magnitude / max_magnitude).log10();
            let normalized = ((db + 100.0) / 100.0).max(0.0).min(1.0);

            let color = color_gradient.eval_continuous(normalized);
            let rgb = RGBColor(color.r, color.g, color.b);

            // Draw rectangle for this time-frequency bin
            chart.draw_series(std::iter::once(Rectangle::new(
                [
                    (time, freq),
                    (
                        time + hop_size as f64 / sample_rate as f64,
                        freq + max_freq / (frame_size / 2) as f64,
                    ),
                ],
                rgb.filled(),
            )))?;
        }
    }

    // Draw peaks as dark blue dots
    for peak in peaks {
        let time = peak.frame_index as f64 * hop_size as f64 / sample_rate as f64;
        let freq = peak.freq_bin as f64 * max_freq / (frame_size / 2) as f64;

        // Draw small solid dots
        chart.draw_series(std::iter::once(Circle::new(
            (time, freq),
            2,                                               // Reduced size
            ShapeStyle::from(&RGBColor(0, 0, 139)).filled(), // Dark blue filled dots
        )))?;
    }

    // Add colorbar with more width
    let (main_area, colorbar_area) = root.split_horizontally(920);
    let mut colorbar = ChartBuilder::on(&colorbar_area)
        .margin(5)
        .x_label_area_size(0)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..1.0, -100.0..0.0)?;

    colorbar
        .configure_mesh()
        .y_desc("Magnitude (dB)")
        .label_style(("sans-serif", 15))
        .draw()?;

    // Draw colorbar with thicker segments
    for i in 0..100 {
        let y = i as f64 / 100.0;
        let color = color_gradient.eval_continuous(y);
        let rgb = RGBColor(color.r, color.g, color.b);

        colorbar.draw_series(std::iter::once(Rectangle::new(
            [
                (0.0, -100.0 + y * 100.0),
                (1.0, -100.0 + (y + 0.01) * 100.0),
            ],
            rgb.filled(),
        )))?;
    }

    root.present()?;

    Ok(())
}
