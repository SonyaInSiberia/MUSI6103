use hound::{SampleFormat, WavReader, WavWriter};
use std::path::Path;

mod lfo;
mod ring_buffer;
mod vibrato;
use vibrato::VFilter;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
    show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <input wave filename> <output wave filename>",
            args[0]
        );
        return;
    }

    // Open the input wave file
    let input_path = Path::new(&args[1]);
    let mut reader = hound::WavReader::open(input_path).unwrap();
    let spec = reader.spec();
    let num_channels = spec.channels as usize;
    let sample_rate_hz = spec.sample_rate as f32;
    let delay_secs = 0.1;
    let width_secs = 0.1;
    let mod_freq_hz = 5.0;

    // Initialize the vibrato filter
    let mut vibrato_filter = VFilter::new(
        sample_rate_hz,
        delay_secs,
        width_secs,
        mod_freq_hz,
        num_channels,
    )
    .expect("Failed to create VFilter");

    // Prepare the output WAV file
    let output_path = Path::new(&args[2]);
    let mut writer = WavWriter::create(output_path, spec).expect("Failed to create WAV file");
    // Read all samples into a vector
    let samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();
    let num_samples = samples.len() / num_channels;

    // Convert samples to f32 and organize by channel
    let mut input_samples: Vec<Vec<f32>> = vec![Vec::with_capacity(num_samples); num_channels];
    for (i, &sample) in samples.iter().enumerate() {
        let channel_index = i % num_channels;
        input_samples[channel_index].push(sample as f32 / i16::MAX as f32);
    }

    // Prepare output samples container
    let mut output_samples: Vec<Vec<f32>> = vec![vec![0.0; num_samples]; num_channels];

    // Process samples through the vibrato filter
    let input_slices: Vec<&[f32]> = input_samples.iter().map(|v| v.as_slice()).collect();
    let mut output_slices: Vec<&mut [f32]> = output_samples
        .iter_mut()
        .map(|v| v.as_mut_slice())
        .collect();
    vibrato_filter.process(&input_slices, &mut output_slices);
    // Write processed samples back, interleaving channels
    for i in 0..num_samples {
        for channel in 0..num_channels {
            let sample = (output_samples[channel][i] * i16::MAX as f32) as i16;
            writer.write_sample(sample).expect("Failed to write sample");
        }
    }

    writer.finalize().expect("Failed to finalize WAV file");
}
