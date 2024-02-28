use crate::ring_buffer::RingBuffer;
use std::f32::consts::PI;

pub struct LFO {
    wave_table: RingBuffer<f32>,
    sample_rate_hz: f32,
    phase_index: f32,
    freq_hz: f32,
    amplitude: f32,
}

impl LFO {
    pub fn new(sample_rate_hz: f32, size: usize) -> Self {
        let mut size_ = size;

        let mut wave_table = RingBuffer::<f32>::new(size_);
        for i in 0..size_ {
            let phase = (i as f32 / size_ as f32) * 2.0 * PI;
            wave_table.push(phase.sin());
        }
        wave_table.push(0.0);
        Self {
            wave_table,
            sample_rate_hz,
            freq_hz: 0.0,
            amplitude: 1.0,
            phase_index: 0.0,
        }
    }
    /// Set the frequency of the LFO in Hz.
    /// Example usage
    /// ```
    /// let mut lfo = LFO::new(44100.0, 1024);
    /// lfo.set_frequency(1.0);
    /// assert_eq!(lfo.freq_hz, 1.0);
    /// ```
    pub fn set_frequency(&mut self, freq_hz: f32) {
        self.freq_hz = freq_hz;
    }

    /// Set the Amplitude of LFO wavetable.
    /// Example usage
    /// ```
    /// let mut lfo = LFO::new(44100.0, 1024);
    /// lfo.set_amplitude(2.0);
    /// assert_eq!(lfo.amplitude, 2.0);
    /// ```
    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.phase_index = phase;
    }

    pub fn get_params(&self) -> (f32, f32, f32) {
        (self.freq_hz, self.amplitude, self.phase_index)
    }
    /// Reset the phase index of the LFO.
    /// Example usage
    /// ```
    /// let mut lfo = LFO::new(44100.0, 1024);
    /// lfo.reset();
    /// assert_eq!(lfo.phase_index, 0.0);
    pub fn reset(&mut self) {
        self.phase_index = 0.0;
    }

    pub fn get_table_step(&self) -> f32 {
        let table_capacity = self.wave_table.capacity() as f32;
        let phase_increment = self.freq_hz / self.sample_rate_hz;
        phase_increment * table_capacity
    }

    pub fn next_mod(&mut self, table_step: f32) -> f32 {
        let table_capacity = self.wave_table.capacity() as f32;
        self.phase_index += table_step;
        self.phase_index = if self.phase_index > table_capacity {
            self.phase_index - table_capacity
        } else {
            self.phase_index
        };
        self.wave_table.get_frac(self.phase_index) * self.amplitude
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    // This one is not realistic (wave table size > sample_rate, but easy for understanding)
    fn test_next_mod() {
        let mut lfo = LFO::new(2.0, 4);
        lfo.set_frequency(1.0);
        lfo.set_amplitude(1.0);
        let table_step = lfo.get_table_step();
        let val = lfo.next_mod(table_step);
        // Be careful with float comparison
        assert!((val - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_next_mod_frac() {
        let mut lfo = LFO::new(5.0, 3);
        lfo.set_frequency(1.0);
        lfo.set_amplitude(1.0);
        let table_step = lfo.get_table_step();
        let val = lfo.next_mod(table_step);
        assert!((val - 0.6 * (2.0 * PI / 3.0).sin()).abs() < 0.0001);
    }
}
