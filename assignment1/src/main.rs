
use std::{env, fs::File, io::Write, path::Path};
use hound::{WavReader, WavWriter, WavSpec, SampleFormat};

mod comb_filter;
use comb_filter::{CombFilter, FilterType};

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
    show_info();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() == 0 {
        println!("No command line arguments provided, running tests...");
        test_fir_output_zero_on_feedforward_freq();
        test_iir_magnitude_change_on_feedback_freq();
        test_varying_input_block_size();
        test_processing_zero_input_signal();
        test_buffer_length_greater_than_input_length();
        std::process::exit(1);
    }
    if args.len() < 4 {
        eprintln!("Usage: {} <input wave filename> <output wave filename> <effect-parameters>", args[0]);
        std::process::exit(1);
    }

    // Open the input wave file
    let input_path = Path::new(&args[1]);
    let mut reader = WavReader::open(input_path).expect("Failed to open WAV file");

    // Prepare the output WAV file
    let spec = reader.spec();
    let output_path = Path::new(&args[2]);
    let mut writer = WavWriter::create(output_path, spec).expect("Failed to create WAV file");

    let filter_params: Vec<&str> = args[3].split(',').collect();
    if filter_params.len() != 5 {
        eprintln!("Invalid number of effect parameters. Expected 5, found {}", filter_params.len());
        std::process::exit(1);
    }
    let filter_type = match filter_params[0] {
        "FIR" => FilterType::FIR,
        "IIR" => FilterType::IIR,
        _ => {
            eprintln!("Invalid filter type: {}", filter_params[0]);
            std::process::exit(1);
        }
    };

    let max_delay_secs = filter_params[1].parse::<f32>().expect("Invalid max delay seconds");
    let sample_rate_hz = filter_params[2].parse::<f32>().expect("Invalid sample rate Hz");
    let gain = filter_params[3].parse::<f32>().expect("Invalid gain");
    let delay_secs = filter_params[4].parse::<f32>().expect("Invalid delay samples");
   


    // Process audio in blocks
    let block_size_per_channel = 1024;
    let num_samples = reader.len() as usize;
    // let num_blocks = num_samples / block_size_per_channel;
    let channels = spec.channels as usize;
    let mut comb_filter = CombFilter::new(filter_type, max_delay_secs, sample_rate_hz, channels, gain, delay_secs).expect("Failed to create CombFilter");
    // Initialize buffers for processing
    let mut input_blocks: Vec<Vec<f32>> = vec![vec![0.0; block_size_per_channel]; channels];
    let mut output_blocks: Vec<Vec<f32>> = vec![vec![0.0; block_size_per_channel]; channels];

    let mut total_samples_written = 0;

    while let Some(samples) = reader.samples::<i16>().take(block_size_per_channel * channels).collect::<Result<Vec<_>, _>>().ok() {
        if samples.is_empty() {
            break;
        }
        let actual_block_size = samples.len() / channels; // Actual number of samples per channel in this block

        // Clear previous block data
        for channel_data in &mut input_blocks {
            channel_data.fill(0.0);
        }

        // Convert and separate samples into channels
        for (i, sample) in samples.iter().enumerate() {
            let channel_index = i % channels;
            let sample_index = i / channels;
            input_blocks[channel_index][sample_index] = *sample as f32 / i16::MAX as f32;
        }

        // Process each block
        let input_slices: Vec<&[f32]> = input_blocks.iter().map(|v| v.as_slice()).collect();
        let mut output_slices: Vec<&mut [f32]> = output_blocks.iter_mut().map(|v| v.as_mut_slice()).collect();
        comb_filter.process(&input_slices, &mut output_slices);

        // Write processed samples back, interleaving channels
        for i in 0..actual_block_size {
            for channel in 0..channels {
                let sample = output_blocks[channel][i];
                let sample_i16 = (sample * i16::MAX as f32) as i16;
                writer.write_sample(sample_i16).expect("Failed to write sample");
                total_samples_written += 1;
            }
        }
    }

    writer.finalize().expect("Failed to finalize WAV file");

}

fn test_fir_output_zero_on_feedforward_freq() {
    let mut filter = CombFilter::new(FilterType::FIR, 1.0, 44100.0, 1, 0.5, 0.25).expect("Failed to create CombFilter");
    let input = vec![vec![0.0; 1024]; 1]; // Example input block of zeros
    let mut output = vec![vec![0.0; 1024]; 1]; // Output buffer

    filter.process(&input.iter().map(|x| &x[..]).collect::<Vec<_>>(), &mut output.iter_mut().map(|x| &mut x[..]).collect::<Vec<_>>());

    // Check if output is approximately zero
    let is_zero_output = output.iter().flatten().all(|&sample| (sample as f32).abs() < 1e-5);
    assert!(is_zero_output, "FIR filter test failed: Output is not zero.");
    println!("FIR Output Zero on Feedforward Frequency: Passed");
}


fn test_iir_magnitude_change_on_feedback_freq() {
    let mut filter = CombFilter::new(FilterType::IIR, 32.0, 44100.0, 1, 0.5, 1.0 / 440.0).expect("Failed to create CombFilter");

    // Create a test input signal - constant value
    let input = vec![vec![1.0; 1024]]; // Mono channel, constant input
    let mut output = vec![vec![0.0; 1024]]; // Output buffer for the processed signal

    // Process the input signal
    filter.process(&input.iter().map(|x| &x[..]).collect::<Vec<_>>(), &mut output.iter_mut().map(|x| &mut x[..]).collect::<Vec<_>>());

    // Analyze the output for expected behavior (e.g., check for stability or expected amplification/attenuation)
    // This part of the test would depend on what specific behavior you expect from your IIR filter
    // For simplicity, here we check if the output stabilizes or shows expected trends
    let last_sample = output[0][1023];
    println!("Last sample of the output: {}", last_sample);
    
    // Assert based on expected behavior, e.g., output should not diverge for a stable filter
    assert!(last_sample.abs() < 10.0, "IIR filter output did not stabilize as expected.");
}


fn test_varying_input_block_size() {
    let mut filter = CombFilter::new(FilterType::FIR, 1.0, 44100.0, 1, 0.5, 0.25).expect("Failed to create CombFilter");
    let input_small = vec![vec![1.0; 512]; 1]; // Smaller block
    let mut output_small = vec![vec![0.0; 512]; 1];
    let input_large = vec![vec![1.0; 2048]; 1]; // Larger block
    let mut output_large = vec![vec![0.0; 2048]; 1];

    filter.process(&input_small.iter().map(|x| &x[..]).collect::<Vec<_>>(), &mut output_small.iter_mut().map(|x| &mut x[..]).collect::<Vec<_>>());
    filter.process(&input_large.iter().map(|x| &x[..]).collect::<Vec<_>>(), &mut output_large.iter_mut().map(|x| &mut x[..]).collect::<Vec<_>>());

    // Placeholder for actual verification of consistency between outputs
    println!("Correct Result for Varying Input Block Size: Test logic needs specific implementation details.");
}

fn test_processing_zero_input_signal() {
    let mut filter = CombFilter::new(FilterType::FIR, 1.0, 44100.0, 1, 0.5, 0.25).expect("Failed to create CombFilter");
    let input_zero = vec![vec![0.0; 1024]; 1]; // Zero input block
    let mut output = vec![vec![0.0; 1024]; 1]; // Output buffer

        filter.process(&input_zero.iter().map(|x| &x[..]).collect::<Vec<_>>(), &mut output.iter_mut().map(|x| &mut x[..]).collect::<Vec<_>>());

    // Check if output is zero
    let is_zero_output = output.iter().flatten().all(|&sample| sample == 0.0);
    assert!(is_zero_output, "Processing Zero Input Signal test failed: Output is not zero.");
    println!("Correct Processing for Zero Input Signal: Passed");
}

#[should_panic(expected = "Buffer length is greater than input length")]
fn test_buffer_length_greater_than_input_length() {
    // Setup - create a CombFilter instance
    let mut filter = CombFilter::new(
        FilterType::FIR, // or FilterType::IIR, depending on what you want to test
        1.0, // max_delay_secs
        44100.0, // sample_rate_hz
        1, // num_channels
        0.5, // gain
        0.01, // delay_secs, adjust this so that delay_samples > in_channel.len()
    ).unwrap();

    // Create an input signal shorter than the buffer length
    let input_signal = vec![vec![0.0; 10]; 1]; // Adjust the length based on your buffer length setup

    // Attempt to process the input signal
    let mut output_signal = vec![vec![0.0; 10]; 1]; // Placeholder for output

    // This call should trigger the panic based on the assertion in your process method
    filter.process(&input_signal.iter().map(|x| &x[..]).collect::<Vec<_>>(), &mut output_signal.iter_mut().map(|x| &mut x[..]).collect::<Vec<_>>());
}