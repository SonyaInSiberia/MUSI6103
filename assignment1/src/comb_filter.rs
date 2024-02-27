pub struct CombFilter {
    // TODO: your code here
    max_delay_secs: f32,
    sample_rate_hz: f32,
    num_channels: usize,
    filter_type: FilterType,
    buffer: Vec<Vec<f32>>,
    gain: f32,
    delay_samples: usize,
    writer_idx: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    FIR,
    IIR,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterParam {
    Gain,
    Delay,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { param: FilterParam, value: f32 }
}

impl CombFilter {
    pub fn new(
        filter_type: FilterType, 
        max_delay_secs: f32, 
        sample_rate_hz: f32, 
        num_channels: usize, 
        gain: f32, 
        delay_secs: f32
    ) -> Result<Self, Error>{
        if gain < 0.0 {
            return Err(Error::InvalidValue{param: FilterParam::Gain, value: gain})
        }
        let delay_samples = (delay_secs * sample_rate_hz).round() as usize;
        if delay_samples > (max_delay_secs * sample_rate_hz).round() as usize {
            return Err(Error::InvalidValue{param: FilterParam::Delay, value: delay_secs})
        } else if delay_samples == 0 && filter_type == FilterType::IIR{
            return Err(Error::InvalidValue{param: FilterParam::Delay, value: delay_secs})
        }
        let buffer = vec![vec![0.0; delay_samples + 1]; num_channels];
        let writer_idx = vec![0; num_channels];
        Ok(Self{
            max_delay_secs,
            sample_rate_hz,
            num_channels,
            filter_type,
            buffer,
            gain,
            delay_samples,
            writer_idx,
        })
    }

    pub fn reset(&mut self) {
        for channel in &mut self.buffer{
            for sample in channel.iter_mut(){
                *sample = 0.0;
            }
        }
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        assert_eq!(input.len(), self.num_channels);
        assert_eq!(output.len(), self.num_channels);
        for channel in 0..input.len(){
            let in_channel = input[channel];
            let out_channel = &mut output[channel];
            if self.buffer[channel].len() > in_channel.len(){
                panic!("Buffer length is greater than input length");
            }
            for (sample_idx, &input_sample) in in_channel.iter().enumerate(){
                // comb filter based on filter type
                // handle ring buffer
                let delayed_index = (self.writer_idx[channel] + self.buffer[channel].len() - self.delay_samples) % self.buffer[channel].len();
                // dbg!(&self.writer_idx[channel], delayed_index);
                // Fetch the delayed sample from the buffer
                let delayed_sample = self.buffer[channel][delayed_index];
                // Calculate the output sample
                let out_sample = input_sample + self.gain * delayed_sample;
                // dbg!(input_sample, delayed_sample, out_sample);
                // dbg!(&out_channel);
                // Update the output buffer
                out_channel[sample_idx] = out_sample;


                // Update the delay buffer with the current input sample
                
                self.buffer[channel][self.writer_idx[channel]] = match self.filter_type {
                    FilterType::FIR => input_sample,
                    FilterType::IIR => out_sample,
                };
                self.writer_idx[channel] = (self.writer_idx[channel] + 1) % self.buffer[channel].len();
            }
        }
    }

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        match param {
            FilterParam::Gain => {
                if value < 0.0 {
                    return Err(Error::InvalidValue{param, value})
                } else {
                    self.gain = value;
                    Ok(())
                }
            },
            FilterParam::Delay => {
                let delay_samples = (value * self.sample_rate_hz).round() as usize;
                if delay_samples > (self.max_delay_secs * self.sample_rate_hz).round() as usize{
                    return Err(Error::InvalidValue{param, value})
                } else if delay_samples == 0 && self.filter_type == FilterType::IIR{
                    return Err(Error::InvalidValue{param, value})
                } else {
                    self.delay_samples = delay_samples;
                    Ok(())
                }
            }
        }
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        match param {
            FilterParam::Gain => self.gain,
            FilterParam::Delay => self.delay_samples as f32 / self.sample_rate_hz,
        }
    }
}

    // TODO: feel free to define other functions for your own use


// TODO: feel free to define other types (here or in other modules) for your own use

// #[cfg(test)]
// mod tests {
//     use super::*; // Import the CombFilter and other necessary items from the outer module

//     #[test]
//     fn test_fir_comb_filter() {
//         // Step 1: Setup - Create a CombFilter instance with known parameters
//         let mut filter = CombFilter::new(FilterType::FIR, 32.0, 1.0, 1, 1.0, 3.0).expect("IDK!!!!");
//         filter.set_param(FilterParam::Gain, 1.0).expect("Failed to set gain"); // Example gain
//         filter.set_param(FilterParam::Delay, 3.0).expect("Failed to set dely"); // Example delay of 1 sample at 44100 Hz

//         // Step 2: Create a test input signal - an impulse signal
//         let input = vec![vec![7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0]]; // Mono channel, impulse at the first sample
//         let mut output = vec![vec![0.0; 7]; 1]; // Output buffer for the processed signal

//         // Step 3: Process the input signal
//         filter.process(&input.iter().map(|x| &x[..]).collect::<Vec<_>>(), &mut output.iter_mut().map(|x| &mut x[..]).collect::<Vec<_>>());

//         // Step 4: Define the expected output signal
//         // Given the filter settings, the output should be an impulse followed by the delayed, attenuated impulse
//         let expected_output = vec![vec![7.0, 6.0, 5.0, 11.0, 9.0, 7.0, 5.0]]; // Expected output considering the gain and delay

//         // Step 5: Assert that the actual output matches the expected output
//         assert_eq!(output, expected_output, "The output signal did not match the expected output.");
//     }
//     #[test]
//     fn test_iir_comb_filter() {
//                 // Step 1: Setup - Create a CombFilter instance with known parameters
//         let mut filter = CombFilter::new(FilterType::IIR, 32.0, 1.0, 1, 1.0, 3.0).expect("IDK!!!!");
//         filter.set_param(FilterParam::Gain, 1.0).expect("Failed to set gain"); // Example gain
//         filter.set_param(FilterParam::Delay, 3.0).expect("Failed to set dely"); // Example delay of 1 sample at 44100 Hz

//         // Step 2: Create a test input signal - an impulse signal
//         let input = vec![vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]]; // Mono channel, impulse at the first sample
//         let mut output = vec![vec![0.0; 7]; 1]; // Output buffer for the processed signal

//         // Step 3: Process the input signal
//         filter.process(&input.iter().map(|x| &x[..]).collect::<Vec<_>>(), &mut output.iter_mut().map(|x| &mut x[..]).collect::<Vec<_>>());

//         // Step 4: Define the expected output signal
//         // Given the filter settings, the output should be an impulse followed by the delayed, attenuated impulse
//         let expected_output = vec![vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0]]; // Expected output considering the gain and delay

//         // Step 5: Assert that the actual output matches the expected output
//         assert_eq!(output, expected_output, "The output signal did not match the expected output.");
//     }
// }
