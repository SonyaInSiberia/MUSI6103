use std::f32::consts::PI;

use crate::ring_buffer::RingBuffer;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue{name: String, value: f32},
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
    phase: f32,
    num_channels: usize,
}

impl VFilter {
    pub fn new(
        sample_rate_hz: f32,
        delay_secs: f32,
        width_secs: f32,
        mod_freq_hz: f32,
        num_channels: usize,
    ) -> Result<Self, Error> {
        let delay_samples = (delay_secs * sample_rate_hz) as usize;
        let width_samples = (width_secs * sample_rate_hz) as usize;
        let capacity = delay_samples + width_samples * 2;
        if width_samples > delay_samples {
            return Err(Error::InvalidValue{name: "width in seconds".to_string(), value: width_secs})
        }
        if (mod_freq_hz - 0.0).abs() < 1e-6 || mod_freq_hz < 0.0{
            return Err(Error::InvalidValue{name: "modulation frequency in Hz".to_string(), value: mod_freq_hz})
        }
        let delay_buffer = vec![RingBuffer::<f32>::new(capacity); num_channels];
        let phase = 0.0;
        Ok(Self {
            sample_rate_hz,
            delay_secs,
            width_secs,
            mod_freq_hz,
            delay_buffer,
            phase,
            num_channels,
        })
    }

    pub fn reset(&mut self) {
        for i in 0..self.num_channels {
            self.delay_buffer[i].reset();
        }
        self.phase = 0.0;
    }

    pub fn set_param(&mut self, delay_secs: f32, width_secs: f32, mod_freq_hz: f32) -> Result<(), Error> {
        let delay_samples = (delay_secs * self.sample_rate_hz) as usize;
        let width_samples = (width_secs * self.sample_rate_hz) as usize;
        let capacity = delay_samples + width_samples * 2;
        if width_samples > delay_samples {
            return Err(Error::InvalidValue{name: "width in seconds".to_string(), value: width_secs})
        }
        if width_samples > delay_samples {
            return Err(Error::InvalidValue{name: "width in seconds".to_string(), value: width_secs})
        }
        if (mod_freq_hz - 0.0).abs() < 1e-6 || mod_freq_hz < 0.0{
            return Err(Error::InvalidValue{name: "modulation frequency in Hz".to_string(), value: mod_freq_hz})
        }
        self.delay_secs = delay_secs;
        self.width_secs = width_secs;
        self.mod_freq_hz = mod_freq_hz;
        // since the width samples might chage, we need to make another buffer with new size
        for i in 0..self.num_channels {
            self.delay_buffer[i] = RingBuffer::<f32>::new(capacity);
        }
        self.phase = 0.0;
        Ok(())
    }

    pub fn get_param(&self) -> (f32, f32, f32) {
        (self.delay_secs, self.width_secs, self.mod_freq_hz)
    }
    
    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        let mod_freq_rad_per_sample = self.mod_freq_hz * 2.0 * PI / self.sample_rate_hz;
        for (channel_idx, &channel_input) in input.iter().enumerate() {
            let channel_delay_buffer = &mut self.delay_buffer[channel_idx];

            for (sample_idx, &sample) in channel_input.iter().enumerate(){
                self.phase = (self.phase + mod_freq_rad_per_sample) % (2.0 * PI);

                let mod_value = self.phase.sin();
                let mod_depth_samples = self.width_secs * self.sample_rate_hz * mod_value;
                let total_delay_samples = self.delay_secs * self.sample_rate_hz + mod_depth_samples;
                let delayed_sample = channel_delay_buffer.get_frac(total_delay_samples);
                output[channel_idx][sample_idx] = delayed_sample;
                channel_delay_buffer.push(sample);
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
        let mut vibrato = VFilter::new(sample_rate_hz, delay_secs, width_secs, mod_freq_hz, num_channels).unwrap();
        let (delay_secs, width_secs
        , mod_freq_hz) = vibrato.get_param();
        assert_eq!(delay_secs, 0.01);
        assert_eq!(width_secs, 0.005);
        assert_eq!(mod_freq_hz, 5.0);
    }

}