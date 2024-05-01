use crate::lfo::LFO;
use crate::ring_buffer::RingBuffer;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { name: String, value: f32 },
}
/*
   Vibrato Effect
   Implementation borrowed from comb_filter.rs from [https://github.com/manodrum/ase-2024/blob/assignment-1/src/comb_filter.rs]
   The struct (cnstructor) takes in the following parameters:
   - sample_rate_hz: f32
   - delay_secs: f32
   - width_secs: f32
   - mod_freq_hz: f32
   - num_channels: usize
   The struct has the following methods:
   - new: creates a new instance of the struct
   - reset: resets the delay buffer and phase
   - set_param: sets the parameters of the filter
   - get_param: gets the parameters of the filter
   - process: processes the input and writes the output to the output buffer
*/
pub struct VFilter {
    sample_rate_hz: f32,
    delay_secs: f32,
    width_secs: f32,
    mod_freq_hz: f32,
    delay_buffer: Vec<RingBuffer<f32>>,
    lfo: LFO, // Add LFO here
    num_channels: usize,
}

impl VFilter {
    /// Creates a new instance of the struct VFilter
    /// Example usage
    /// ```
    /// let vibrato = VFilter::new(44100.0, 0.01, 0.005, 5.0, 2);
    /// assert_eq!(vibrato.sample_rate_hz, 44100.0);
    /// assert_eq!(vibrato.delay_secs, 0.01);
    /// assert_eq!(vibrato.width_secs, 0.005);
    /// assert_eq!(vibrato.mod_freq_hz, 5.0);
    /// assert_eq!(vibrato.num_channels, 2);
    /// ```
    pub fn new(
        sample_rate_hz: f32,
        delay_secs: f32,
        width_secs: f32,
        mod_freq_hz: f32,
        num_channels: usize,
    ) -> Result<Self, Error> {
        let delay_samples = (delay_secs * sample_rate_hz) as usize;
        let width_samples = (width_secs * sample_rate_hz) as usize;
        let capacity = 1 + delay_samples + width_samples * 2;
        if width_samples > delay_samples {
            return Err(Error::InvalidValue {
                name: "width in seconds".to_string(),
                value: width_secs,
            });
        }
        if (mod_freq_hz - 0.0).abs() < 1e-6 || mod_freq_hz < 0.0 {
            return Err(Error::InvalidValue {
                name: "modulation frequency in Hz".to_string(),
                value: mod_freq_hz,
            });
        }
        let delay_buffer = vec![RingBuffer::<f32>::new(capacity); num_channels];
        let mut lfo = LFO::new(sample_rate_hz, 1024);
        lfo.set_frequency(mod_freq_hz);
        // I guess it is  ok to have the amplitude as the width in samples since witdth length is set to always
        // equal to delay in main.rs
        lfo.set_amplitude(width_samples as f32);
        Ok(Self {
            sample_rate_hz,
            delay_secs,
            width_secs,
            mod_freq_hz,
            delay_buffer,
            lfo,
            num_channels,
        })
    }

    /// `reset` resets the delay buffer to default values
    /// Example usage
    /// ```
    /// let mut vibrato = VFilter::new(44100.0, 0.01, 0.005, 5.0, 2).unwrap();
    /// vibrato.reset();
    /// assert_eq!(vibrato.delay_buffer[0].len(), 0);
    /// assert_eq!(vibrato.lfo.phase_index, 0.0);
    /// ```
    pub fn reset(&mut self) {
        for i in 0..self.num_channels {
            self.delay_buffer[i].reset();
        }
    }

    /// `set_param` sets the parameters (delay, width, LFO frequency) of the filter
    /// Example usage
    /// ```
    /// let mut vibrato = VFilter::new(44100.0, 0.01, 0.005, 5.0, 2).unwrap();
    /// vibrato.set_param(0.02, 0.01, 10.0);
    /// assert_eq!(vibrato.delay_secs, 0.02);
    /// assert_eq!(vibrato.width_secs, 0.01);
    /// assert_eq!(vibrato.mod_freq_hz, 10.0);
    pub fn set_param(
        &mut self,
        delay_secs: f32,
        width_secs: f32,
        mod_freq_hz: f32,
    ) -> Result<(), Error> {
        let delay_samples = (delay_secs * self.sample_rate_hz) as usize;
        let width_samples = (width_secs * self.sample_rate_hz) as usize;
        let capacity = 1 + delay_samples + width_samples * 2;
        if width_samples > delay_samples {
            return Err(Error::InvalidValue {
                name: "width in seconds".to_string(),
                value: width_secs,
            });
        }
        if width_samples > delay_samples {
            return Err(Error::InvalidValue {
                name: "width in seconds".to_string(),
                value: width_secs,
            });
        }
        if (mod_freq_hz - 0.0).abs() < 1e-6 || mod_freq_hz < 0.0 {
            return Err(Error::InvalidValue {
                name: "modulation frequency in Hz".to_string(),
                value: mod_freq_hz,
            });
        }
        self.delay_secs = delay_secs;
        self.width_secs = width_secs;
        self.mod_freq_hz = mod_freq_hz;
        // since the width samples might chage, we need to make another buffer with new size
        for i in 0..self.num_channels {
            self.delay_buffer[i] = RingBuffer::<f32>::new(capacity);
        }
        Ok(())
    }

    /// `get_param` gets the parameters (delay, width, LFO frequency) of the filter
    /// Example usage
    /// ```
    /// let vibrato = VFilter::new(44100.0, 0.01, 0.005, 5.0, 2).unwrap();
    /// let (delay_secs, width_secs, mod_freq_hz) = vibrato.get_param();
    /// assert_eq!(delay_secs, 0.01);
    /// assert_eq!(width_secs, 0.005);
    /// assert_eq!(mod_freq_hz, 5.0);
    pub fn get_param(&self) -> (f32, f32, f32) {
        (self.delay_secs, self.width_secs, self.mod_freq_hz)
    }

    /// `process` processes the input and writes the output to the output buffer
    /// Example usage
    /// ```
    /// let vibrato = VFilter::new(1.0, 2.0, 1.0, 1.0, 2).unwrap();
    /// let input = vec![vec![1.0; 44100]; 2];
    /// let mut output = vec![vec![0.0; 44100]; 2];
    /// let input_slices: Vec<&[f32]> = input.iter().map(|v| v.as_slice()).collect();
    /// let input: &[&[f32]] = &input_slices;
    /// let mut output_slices: Vec<&mut [f32]> = output.iter_mut().map(|v| v.as_mut_slice()).collect();
    /// let mut output: &mut [&mut [f32]] = &mut output_slices;
    /// vibrato.process(&input, &mut output);
    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        for (channel_idx, &channel_input) in input.iter().enumerate() {
            let channel_delay_buffer = &mut self.delay_buffer[channel_idx];

            for (sample_idx, &sample) in channel_input.iter().enumerate() {
                let mod_depth_samples = self.lfo.next_mod();

                let total_delay_samples = self.delay_secs * self.sample_rate_hz + mod_depth_samples;

                channel_delay_buffer.push(sample);

                let delayed_sample = channel_delay_buffer.get_frac(total_delay_samples);

                output[channel_idx][sample_idx] = delayed_sample;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vibrato() {
        let sample_rate_hz = 44100.0;
        let delay_secs = 0.01;
        let width_secs = 0.005;
        let mod_freq_hz = 5.0;
        let num_channels = 2;
        let vibrato = VFilter::new(
            sample_rate_hz,
            delay_secs,
            width_secs,
            mod_freq_hz,
            num_channels,
        )
        .unwrap();
        let (delay_secs_check, width_secs_check, mod_freq_hz_check) = vibrato.get_param();
        assert_eq!(delay_secs_check, 0.01);
        assert_eq!(width_secs_check, 0.005);
        assert_eq!(mod_freq_hz_check, 5.0);
    }

    #[test]
    fn test_set_param() {
        let sample_rate_hz = 44100.0;
        let delay_secs = 0.01;
        let width_secs = 0.005;
        let mod_freq_hz = 5.0;
        let num_channels = 2;
        let mut vibrato = VFilter::new(
            sample_rate_hz,
            delay_secs,
            width_secs,
            mod_freq_hz,
            num_channels,
        )
        .unwrap();
        let (delay_secs_check_1, width_secs_check_1, mod_freq_hz_check_1) = vibrato.get_param();
        vibrato.set_param(0.02, 0.01, 10.0).unwrap();
        let (delay_secs_check_2, width_secs_check_2, mod_freq_hz_check_2) = vibrato.get_param();
        assert_eq!(delay_secs_check_1 * 2.0, delay_secs_check_2);
        assert_eq!(width_secs_check_1 * 2.0, width_secs_check_2);
        assert_eq!(mod_freq_hz_check_1 * 2.0, mod_freq_hz_check_2);
    }

    #[test]
    fn test_reset() {
        let sample_rate_hz = 44100.0;
        let delay_secs = 0.01;
        let width_secs = 0.005;
        let mod_freq_hz = 5.0;
        let num_channels = 1;
        let mut vibrato = VFilter::new(
            sample_rate_hz,
            delay_secs,
            width_secs,
            mod_freq_hz,
            num_channels,
        )
        .unwrap();
        let input = vec![vec![1.0; 44100]; 1];
        let mut output = vec![vec![0.0; 44100]; 1];
        // Convert Vec<Vec<f32>> to &[&[f32]] for input and &mut [&mut [f32]] for output
        let input_slices: Vec<&[f32]> = input.iter().map(|v| v.as_slice()).collect();
        let input: &[&[f32]] = &input_slices;

        let mut output_slices: Vec<&mut [f32]> =
            output.iter_mut().map(|v| v.as_mut_slice()).collect();
        let mut output: &mut [&mut [f32]] = &mut output_slices;
        vibrato.process(&input, &mut output);
        vibrato.reset();
        assert_eq!(output[0][0], 0.0);
    }

    #[test]
    fn output_equals_delayed_input_with_zero_modulation() {
        let sample_rate_hz = 3.0;
        let delay_secs = 1.0; // 10 samples of delay
        let width_secs = 0.0; // No modulation
        let mod_freq_hz = 5.0; // Modulation frequency, irrelevant here due to zero width
        let num_channels = 1;
        let mut vibrato = VFilter::new(
            sample_rate_hz,
            delay_secs,
            width_secs,
            mod_freq_hz,
            num_channels,
        )
        .unwrap();

        let input_len = 5;
        let input = vec![vec![1.0; input_len]; num_channels];
        let mut output = vec![vec![0.0; input_len]; num_channels];

        let input_slices: Vec<&[f32]> = input.iter().map(|v| v.as_slice()).collect();
        let mut output_slices: Vec<&mut [f32]> =
            output.iter_mut().map(|v| v.as_mut_slice()).collect();

        vibrato.process(&input_slices, &mut output_slices);

        // Expect the output to be delayed by delay_samples (not exactly the first few samples due to initial buffer state)
        let delay_samples = (sample_rate_hz * delay_secs) as usize;
        // dbg!(delay_samples);
        for i in 0..input_len {
            if i < delay_samples {
                assert_eq!(
                    output[0][i], 0.0,
                    "Output should be 0 for initial delay period"
                );
            } else {
                assert_eq!(
                    output[0][i], 1.0,
                    "Output should match delayed input after initial delay period"
                );
            }
        }
    }

    #[test]
    fn dc_input_results_in_dc_output() {
        let sample_rate_hz = 44100.0;
        let delay_secs = 0.01; // Example delay
        let width_secs = 0.005; // Example modulation width
        let mod_freq_hz = 5.0; // Example modulation frequency
        let num_channels = 1;
        let mut vibrato = VFilter::new(
            sample_rate_hz,
            delay_secs,
            width_secs,
            mod_freq_hz,
            num_channels,
        )
        .unwrap();

        let input_len = 441; // Example input length
        let dc_value = 0.5; // Example DC value for input
        let input = vec![vec![dc_value; input_len]; num_channels];
        let mut output = vec![vec![0.0; input_len]; num_channels];

        let input_slices: Vec<&[f32]> = input.iter().map(|v| v.as_slice()).collect();
        let mut output_slices: Vec<&mut [f32]> =
            output.iter_mut().map(|v| v.as_mut_slice()).collect();

        vibrato.process(&input_slices, &mut output_slices);

        // Allow some initial transient samples for the effect to stabilize
        let transient_samples = (sample_rate_hz * delay_secs) as usize;

        for channel in output.iter() {
            for &sample in channel.iter().skip(transient_samples) {
                assert!(
                    (sample - dc_value).abs() < 0.001,
                    "Output should remain constant (DC) after initial transient"
                );
            }
        }
    }

    #[test]
    fn varying_input_block_size() {
        let sample_rate_hz = 44100.0;
        let delay_secs = 0.01;
        let width_secs = 0.005;
        let mod_freq_hz = 5.0;
        let num_channels = 1;
        let mut vibrato = VFilter::new(
            sample_rate_hz,
            delay_secs,
            width_secs,
            mod_freq_hz,
            num_channels,
        )
        .unwrap();

        // Test with different input sizes
        for &input_len in &[100, 500, 1000] {
            let input = vec![vec![1.0; input_len]; num_channels];
            let mut output = vec![vec![0.0; input_len]; num_channels];

            let input_slices: Vec<&[f32]> = input.iter().map(|v| v.as_slice()).collect();
            let mut output_slices: Vec<&mut [f32]> =
                output.iter_mut().map(|v| v.as_mut_slice()).collect();

            vibrato.process(&input_slices, &mut output_slices);

            let transient_samples = (sample_rate_hz * delay_secs) as usize;

            for channel in output.iter() {
                for &sample in channel.iter().skip(transient_samples) {
                    assert!(
                        (sample - 1.0).abs() < 0.001,
                        "Output should remain constant (DC) after initial transient"
                    );
                }
            }
        }
    }

    #[test]
    fn zero_input_signal() {
        let sample_rate_hz = 44100.0;
        let delay_secs = 0.01;
        let width_secs = 0.005;
        let mod_freq_hz = 5.0;
        let num_channels = 1;
        let mut vibrato = VFilter::new(
            sample_rate_hz,
            delay_secs,
            width_secs,
            mod_freq_hz,
            num_channels,
        )
        .unwrap();

        let input_len = 441;
        let input = vec![vec![0.0; input_len]; num_channels]; // Zero input
        let mut output = vec![vec![0.0; input_len]; num_channels];

        let input_slices: Vec<&[f32]> = input.iter().map(|v| v.as_slice()).collect();
        let mut output_slices: Vec<&mut [f32]> =
            output.iter_mut().map(|v| v.as_mut_slice()).collect();

        vibrato.process(&input_slices, &mut output_slices);

        for channel in output.iter() {
            for &sample in channel.iter() {
                assert_eq!(sample, 0.0, "Output should be 0 for zero input signal");
            }
        }
    }

    #[test]
    fn vibrato_with_zero_delay() {
        let sample_rate_hz = 3.0;
        let delay_secs = 0.0; // No delay
        let width_secs = 0.0; // Example modulation width, should be irrelevant here
        let mod_freq_hz = 5.0; // Example modulation frequency, should also be irrelevant
        let num_channels = 1;
        let mut vibrato = VFilter::new(
            sample_rate_hz,
            delay_secs,
            width_secs,
            mod_freq_hz,
            num_channels,
        )
        .unwrap();

        // Define a test input signal
        let input_len = 5; // Arbitrary length for test
        let test_signal = 0.5; // Example signal value
        let input = vec![vec![test_signal; input_len]; num_channels];
        let mut output = vec![vec![0.0; input_len]; num_channels];

        // Convert input and output vectors to slices for processing
        let input_slices: Vec<&[f32]> = input.iter().map(|v| v.as_slice()).collect();
        let mut output_slices: Vec<&mut [f32]> =
            output.iter_mut().map(|v| v.as_mut_slice()).collect();

        // Process the input signal through the vibrato effect
        vibrato.process(&input_slices, &mut output_slices);

        // Verify that the output signal matches the input signal exactly
        for (input_channel, output_channel) in input.iter().zip(output.iter()) {
            for (input_sample, output_sample) in input_channel.iter().zip(output_channel.iter()) {
                assert_eq!(
                    input_sample, output_sample,
                    "The output should match the input exactly when delay is 0."
                );
            }
        }
    }
}
