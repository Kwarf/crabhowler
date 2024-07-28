use clack_plugin::events::{
    event_types::{NoteOffEvent, NoteOnEvent},
    Match,
};

pub trait Oscillator {
    fn handle_note_on(&mut self, event: &NoteOnEvent);
    fn handle_note_off(&mut self, event: &NoteOffEvent);
    fn process(&mut self, left: &mut [f32], right: &mut [f32]);
    fn is_active(&self) -> bool;
}

pub struct SineOscillator {
    sample_rate: f32,
    frequency: f32,
    phase: f32,
}

impl SineOscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            frequency: 0.0,
            phase: 0.0,
        }
    }
}

impl Oscillator for SineOscillator {
    fn handle_note_on(&mut self, event: &NoteOnEvent) {
        if let Match::Specific(key) = event.key() {
            self.frequency = 440.0 * 2.0f32.powf((key as f32 - 57.0) / 12.0);
        }
    }

    fn handle_note_off(&mut self, _event: &NoteOffEvent) {
        self.frequency = 0.0;
        self.phase = 0.0;
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        if self.frequency == 0.0 {
            left.fill(0.0);
            right.fill(0.0);
            return;
        }

        let increment = std::f32::consts::TAU * self.frequency / self.sample_rate;
        for (left, right) in left.iter_mut().zip(right.iter_mut()) {
            let sample = (self.phase * increment).sin();
            *left = sample;
            *right = sample;
            self.phase = (self.phase + 1.0) % self.sample_rate;
        }
    }

    fn is_active(&self) -> bool {
        self.frequency != 0.0
    }
}
